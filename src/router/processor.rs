use super::super::router;
use bp7::Bundle;
use crate::cla::cla_handle::{ClaHandle, HandleId};
use crate::cla::cla_manager::ClaManager;
use tokio::prelude::*;
use crate::bus::{ModuleMsgEnum, BusMessage};
use crate::router::RouterModule;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;

pub struct Processor {
    // pub adapters: Arc<RwLock<HashMap<HandleId, Arc<Mutex<ClaHandle>>>>>,
    // conf: Arc<RwLock<Configuration>>,
    bus_tx: Option<Sender<ModuleMsgEnum>>,
    tx: Sender<ModuleMsgEnum>,
    rx: Arc<Mutex<Receiver<ModuleMsgEnum>>>,

}

impl Processor {
    pub fn new() -> Self {
        let (tx,rx) = channel::<ModuleMsgEnum>(50);

        Self {
            tx,
            rx:  Arc::new(Mutex::new(rx)),
            bus_tx:   None,
        }
    }

    pub fn start(&mut self, bus_tx: Sender<ModuleMsgEnum>) {
        self.bus_tx = Some(bus_tx.clone());
        let rx = self.rx.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            while let Some(msg) = rx.lock().await.recv().await {
                // Listen for updates from CLAs
                tokio::spawn(async move {
                    // Do something with the msg
                });
            } // end While
        });  // end spawn

        let mut bus_tx = bus_tx.clone();
 
        tokio::spawn( {
            async move {   
                let res = bus_tx.send(ModuleMsgEnum::MsgBus(
                                BusMessage::SetTx(
                                    tx.clone(), RouterModule::Processing))).await;
            }});


    }
    pub async fn process_bundle(&self, bundle: Bundle, handle_id: HandleId) {
        // Process flags

        // update CLA stats

        // Lookup next hop

        // Send to destination

        // else store

        // else drop

        // cleanup dropped bundle, notify
    }
}



// pub struct Processor {
//     node: String,
//     cla_manager: Arc<ClaManager>,
// }

//impl Processor {
    // pub fn new() -> Self {
    //     Self {
    //         node: conf.local_eid.node_id().unwrap(),
    //         cla_manager: Arc::new(ClaManager::new(conf)),
    //     }
    // }


//     pub async fn start(&mut self) {
//         println!("Starting processor for node: {}", self.node);

//         println!("Building bundle loop");
//         let cla_manager = self.cla_manager.clone();
//         let (tx, rx) : (Sender<(HandleId,Bundle)>, Receiver< (HandleId, Bundle) >) = channel();

//         // cla_manager.start(tx);

//         let process_loop = async move {
//             loop {
//                 let (id, bun) = rx.recv().unwrap();

//                 // println!("Received bundle on: {}", &cla_manager.adapters.read().unwrap().get(&id).unwrap().lock().unwrap().name);
//                 // TODO Update stats on Cla Handle
//                 // &self.process_bundle(bun, id);
//             }
//         };
//         process_loop.await;
// //        tokio::spawn(process_loop);
//         println!("Started bundle processing loop");




//         // TODO Timer loop to check if bundles can be sent
//     }
