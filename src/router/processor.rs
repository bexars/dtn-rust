use log::*;
use bp7::Bundle;
use crate::cla::cla_handle::{ClaHandle, HandleId};
use crate::bus::ModuleMsgEnum;
use crate::router::RouterModule;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
use msg_bus::{MsgBusHandle, Message};

pub struct Processor {
    // pub adapters: Arc<RwLock<HashMap<HandleId, Arc<Mutex<ClaHandle>>>>>,
    // conf: Arc<RwLock<Configuration>>,
    bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>,
    rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,

}

impl Processor {
    pub async fn new(mut bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>) -> Self {
        let rx = bus_handle.register(RouterModule::Processing).await.unwrap();

        Self {
            rx:  Arc::new(Mutex::new(rx)),
            bus_handle,
        }
    }

    pub async fn start(&mut self) {
        let rx = self.rx.clone();

        let bus_handle = self.bus_handle.clone();

        while let Some(msg) = rx.lock().await.recv().await {
                // Listen for updates from CLAs
                match msg {
                    Message::Shutdown => { break; },
                    _ => { debug!("Unknown msg: {:?}", msg); },
                };
            
                // tokio::spawn(async move {
                //     // Do something with the msg
                // });
        }; // end While

 


    }
    pub async fn process_bundle(&self, _bundle: Bundle, _handle_id: HandleId) {
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
