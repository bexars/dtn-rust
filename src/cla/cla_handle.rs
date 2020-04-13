use std::sync::Arc;
use crate::processor::Processor;
use super::ClaRW;
use super::ClaType;
use crate::system::SystemModules;
use crate::routing::MetaBundle;
use tokio::sync::mpsc::{Sender,Receiver};
use msg_bus::{MsgBus, MsgBusHandle};
use crate::bus::ModuleMsgEnum;


pub struct ClaHandle {
    pub rw: ClaRW,
    pub id: HandleId,
    pub in_bytes: usize,
    pub out_bytes: usize,
    pub in_bundles: usize,
    pub out_bundles: usize,
    pub name: String,
    bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>,
    router_handle: Option<Sender<MetaBundle>>,
    tx: Sender<MetaBundle>,
    rx: Receiver<MetaBundle>,

}

pub type HandleId = usize;

// impl Default for ClaHandle {
//     fn default() -> ClaHandle {
//         Self {
//             id: 0,
//             name: String::from(""),
//             rw: ClaRW::R,
//             in_bundles: 0,
//             in_bytes: 0,
//             out_bundles: 0,
//             out_bytes: 0,
            
//         }
//     }
// }



impl ClaHandle {
    pub async fn new( id: HandleId, name: String, rw: ClaRW, bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>) -> ClaHandle {
        let (tx, rx) = tokio::sync::mpsc::channel(50);
        Self {
            id,
            name,
            rw,
            bus_handle,
            router_handle: None,
            in_bundles: 0,
            in_bytes: 0,
            out_bundles: 0,
            out_bytes: 0,
            tx,
            rx,

        }
    }

    pub async fn start(&mut self) {
        let routing_handle = crate::routing::router::add_cla_handle(&mut self.bus_handle, self.id, self.tx.clone()).await;
        self.router_handle = Some(routing_handle);
    }

    pub async fn process_bundle(metabun: Arc<MetaBundle>) {
        // update stats
        // send to the router
    }
}