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


pub struct Router {
    bus_handle:     BusHandle,
    rx:             Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,
    router_handle:   Sender<MetaBundle>,
    route_receiver: Arc<Mutex<Receiver<MetaBundle>>>,
    cla_handles:    Arc<RwLock<HashMap<HandleId, Sender<MetaBundle>>>>,
}

impl Router {

    pub async fn new(mut bus_handle: BusHandle) -> Self {
        let rx = bus_handle.register(SystemModules::Routing).await.unwrap();
        let (rt_tx, rt_rx) = channel::<MetaBundle>(100);


        Self {
            rx:             Arc::new(Mutex::new(rx)),
            bus_handle,
            router_handle:   rt_tx,
            route_receiver: Arc::new(Mutex::new(rt_rx)),
            cla_handles:    Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&mut self) {
        let rx = self.rx.clone();
        

        let bus_handle = self.bus_handle.clone();



        while let Some(msg) = rx.lock().await.recv().await {
                // Listen for updates from CLAs
                match msg {
                    Message::Shutdown => { break; },
                    Message::Rpc(ModuleMsgEnum::MsgRouting(msg), callback) => {
                        match msg {
                            AddClaHandle(handle_id, sender) => {
                                self.cla_handles.write().await.insert(handle_id, sender).unwrap();
                                callback.send(ModuleMsgEnum::MsgRouting(DataRouterHandle(self.router_handle.clone()))).unwrap();
                            }
                            DropClaHandle(handle_id) => {
                                self.cla_handles.write().await.remove(&handle_id).unwrap();
                                callback.send(ModuleMsgEnum::MsgOk("".to_string())).unwrap();
                            }
                            _ => { debug!("Uknown RPC"); }
                        }
                    }
                    _ => {},
                };
            
                // tokio::spawn(async move {
                //     // Do something with the msg
                // });
        }; // end While
    }

    async fn route_loop(&self, rx: Arc<Mutex<Receiver<bp7::Bundle>>> ) {
        let mut rx = rx.lock().await;  // take the lock forever!!
        while let Some(bundle) = rx.recv().await {
            debug!("Got a bundle");
        }
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