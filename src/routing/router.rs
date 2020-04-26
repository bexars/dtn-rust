use log::*;
use crate::system::BusHandle;
use crate::bus::{ModuleMsgEnum};
use crate::system::SystemModules;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
use std::collections::HashMap;
use msgbus::{Message};
use super::RoutingMessage::*;
use crate::cla::{ClaMessage, HandleId};
use super::*;



pub struct Router {
    bus_handle:     BusHandle,
    rx:             Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,
    router_handle:   Sender<MetaBundle>,
    route_receiver: Arc<Mutex<Receiver<MetaBundle>>>,
    cla_handles:    Arc<RwLock<HashMap<HandleId, Sender<MetaBundle>>>>,
    // route_table:    Arc<RwLock<RouteTableEntry>>,

}

impl Router {

    pub async fn new(mut bus_handle: BusHandle) -> Self  {
        let rx = bus_handle.register(SystemModules::Routing).await.unwrap();
        let (rt_tx, rt_rx) = channel::<MetaBundle>(100);

        let table_entry: RouteTableEntry = Default::default();
        
        Self {
            rx:             Arc::new(Mutex::new(rx)),
            bus_handle,
            router_handle:   rt_tx,
            route_receiver: Arc::new(Mutex::new(rt_rx)),
            cla_handles:    Arc::new(RwLock::new(HashMap::new())),
            // route_table:    Arc::new(RwLock::new(table_entry)),
        }
    }

    pub async fn start(&self) {
        let rx = self.rx.clone();
        

        let _bus_handle = self.bus_handle.clone();
        let (mut tx_control, rx_control) = channel::<()>(1);

        // tokio::task::spawn(Router::route_loop(self.route_receiver.clone(), self.cla_handles.clone(), rx_control));

        info!("Starting route control loop");
        while let Some(msg) = rx.lock().await.recv().await {
                // Listen for updates from CLAs
                match msg {
                    Message::Shutdown => { 
                        info!("Shutting down route control");
                        tx_control.send(()).await.unwrap();
                        break; 
                    },

                    Message::Rpc(ModuleMsgEnum::MsgRouting(msg), callback) => {
                        match msg {
                            AddClaHandle(handle_id, sender) => {
                                self.cla_handles.write().await.insert(handle_id, sender);
                                callback.send(ModuleMsgEnum::MsgRouting(DataRouterHandle(self.router_handle.clone()))).unwrap();
                            }
                            DropClaHandle(handle_id) => {
                                self.cla_handles.write().await.remove(&handle_id);
                                callback.send(ModuleMsgEnum::MsgOk("".to_string())).unwrap();
                            }
                            GetRoutesString => {
                                let res = super::ROUTING_TABLE.load().format_routes().await;
                                callback.send(ModuleMsgEnum::MsgOk(res)).unwrap();
                            }
                            _ => { debug!("Uknown RPC {:?}", msg); }
                        }
                    }
                    Message::Message(ModuleMsgEnum::MsgRouting(msg)) => {
                        match msg {
                            ForwardBundle(metabun) => {
                                let s = self.clone();
                                s.forward_bundle(metabun).await;
                            }
                            _ => { debug!("Unknown Message {:?}", msg)},
                        }
                    }

                    Message::Broadcast(ModuleMsgEnum::MsgRouting(msg)) => {
                        match msg {
                            AddRoute(route) => { 
                                debug!("Adding route {:?}", route);
                                let new_table = Arc::new(super::ROUTING_TABLE.load().add(route));
                                super::ROUTING_TABLE.swap(new_table);
                                // self.route_table.write().await.add(route); 
                            },
                            _ => {},
                        }
                    }
                    _ => { },
                };
            
                // tokio::spawn(async move {
                //     // Do something with the msg
                // });
        }; // end While
    }

    async fn forward_bundle(&self, metabun: MetaBundle) {
        debug!("Received Bundle");

        let cla_id = super::ROUTING_TABLE.load().lookup(&metabun.dest);
        let cla_id = match cla_id {
            None => { 
                debug!("Only Null route found. Dropping");  // TODO bounce back to processing for storing
                return;
            },
            Some(cla_id) => { cla_id },
        };
        let mut bh = self.bus_handle.clone();
        bh.send(SystemModules::Cla(cla_id), ModuleMsgEnum::MsgCla(ClaMessage::TransmitBundle(metabun))).await.unwrap();    

       
    }

}


///////////////  Helper functions /////////////////////////
//
pub async fn add_cla_handle(bus_handle: &mut BusHandle, handle_id: HandleId, cla_sender: Sender<MetaBundle>) -> Sender<MetaBundle> {
    let res = bus_handle.rpc(SystemModules::Routing, ModuleMsgEnum::MsgRouting(AddClaHandle(handle_id, cla_sender))).await.unwrap();
    if let ModuleMsgEnum::MsgRouting(DataRouterHandle(router_handle)) = res {
        return router_handle;
    } else { panic!("Not a router_handle from messaging"); }

}

pub async fn add_route(bus_handle: &mut BusHandle, route: Route) {
    bus_handle.broadcast(ModuleMsgEnum::MsgRouting(AddRoute(route))).await.unwrap();
}

pub async fn get_routes_string(bus_handle: &mut BusHandle) -> String {
    let res = bus_handle.rpc(SystemModules::Routing, ModuleMsgEnum::MsgRouting(GetRoutesString)).await.unwrap();
    if let ModuleMsgEnum::MsgOk(routes) = res {
        return routes;
    }
    return "Route printing is unavailable".to_string();
}

pub async fn forward_bundle(bh: &mut BusHandle, metabun: MetaBundle) {
    bh.send(SystemModules::Routing, ModuleMsgEnum::MsgRouting(ForwardBundle(metabun))).await.unwrap();
}