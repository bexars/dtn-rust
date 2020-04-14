use log::*;
use crate::system::BusHandle;
use crate::bus::{ModuleMsgEnum};
use crate::system::SystemModules;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
use std::collections::HashMap;
use msg_bus::{MsgBusHandle, Message};
use super::RoutingMessage::*;
use crate::cla::cla_handle::{ClaHandle, HandleId};
use super::*;


pub struct RouteTableEntry {
    route: Route,
    kids: Vec<RouteTableEntry>,
}

impl RouteTableEntry {
    pub fn add(&mut self, new_route: Route) {
        let mut new_rte = RouteTableEntry {
            route: new_route.clone(),
            kids: Vec::new(),
        };
        if self.kids.len() > 0 {
            let og_kid_len = self.kids.len();
            for i in 0..self.kids.len() {
                let j = (og_kid_len - 1) -i;
                if  new_route.dest.contains(&self.kids[j].route.dest) {
                    new_rte.kids.push(self.kids.remove(j));
                }
                
            }
        }
        // if !self.route.dest.contains(&route.dest) { return };
        let i = self.kids.iter_mut().filter(|kid| kid.route.dest.contains(&new_route.dest)).next();
        if let Some(kid) = i {
            kid.add(new_route);            
        } else { 
            self.kids.push(new_rte); 
        }
    }
}

pub struct Router {
    bus_handle:     BusHandle,
    rx:             Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,
    router_handle:   Sender<MetaBundle>,
    route_receiver: Arc<Mutex<Receiver<MetaBundle>>>,
    cla_handles:    Arc<RwLock<HashMap<HandleId, Sender<MetaBundle>>>>,
    route_table:    RouteTableEntry,

}

impl Router {

    pub async fn new(mut bus_handle: BusHandle) -> Self  {
        let rx = bus_handle.register(SystemModules::Routing).await.unwrap();
        let (rt_tx, rt_rx) = channel::<MetaBundle>(100);

        let route = Route {
            dest: NodeRoute::from(""),
            nexthop: RouteType::Null,
        };

        let table_entry = RouteTableEntry{
            route: route,
            kids: Vec::new(),
        };

        Self {
            rx:             Arc::new(Mutex::new(rx)),
            bus_handle,
            router_handle:   rt_tx,
            route_receiver: Arc::new(Mutex::new(rt_rx)),
            cla_handles:    Arc::new(RwLock::new(HashMap::new())),
            route_table:    table_entry,
        }
    }

    pub async fn start(&mut self) {
        let rx = self.rx.clone();
        

        let bus_handle = self.bus_handle.clone();
        let (mut tx_control, rx_control) = channel::<()>(1);

        tokio::task::spawn(Router::route_loop(self.route_receiver.clone(), rx_control));

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
                                let res = self.format_routes().await;
                                callback.send(ModuleMsgEnum::MsgOk(res)).unwrap();
                            }
                            _ => { debug!("Uknown RPC"); }
                        }
                    }

                    Message::Broadcast(ModuleMsgEnum::MsgRouting(msg)) => {
                        match msg {
                            AddRoute(route) => { 
                                debug!("Adding route {:?}", route);
                                self.route_table.add(route); }
                            _ => {}
                        }
                    }
                    _ => { },
                };
            
                // tokio::spawn(async move {
                //     // Do something with the msg
                // });
        }; // end While
    }

    async fn route_loop(rx: Arc<Mutex<Receiver<MetaBundle>>>, mut rx_control: Receiver<()> ) {
        let mut rx = rx.lock().await;  // take the lock forever!!
        info!("Starting routing");
        loop {
            let bundle = tokio::select! {
                b = rx.recv() => b,
                _ = rx_control.recv() => {
                    info!("Received shutdown in routing loop");
                    break;
                },
            };
            

        

        }
        debug!("Exited routing loop");
    }

    // let route = Route {
    //     dest: NodeRoute::from(""),
    //     nexthop: RouteType::Null,
    // };

    // let table_entry = RouteTableEntry{
    //     route: route,
    //     kids: Vec::new(),
    // };

    async fn format_routes(&self) -> String {
        fn fmt_parent(parent: &RouteTableEntry, out: &mut String, indent:String) {
            let o = format!( "{}{}   Nexthop:   {}\n", indent, parent.route.dest, parent.route.nexthop);
            out.push_str(&o);
            let indent = format!("{}    ", indent);
            for k in &parent.kids {
                fmt_parent(&k, &mut *out, indent.clone());
            };
        };
        
        let rt = &self.route_table;
        let mut out = String::new();
        let mut indent = String::new();
        fmt_parent(&rt, &mut out, indent);
        // println!("{}", out);
        out
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