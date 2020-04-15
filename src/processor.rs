use log::*;
use bp7::Bundle;
use crate::cla::cla_handle::{HandleId};
use crate::bus::ModuleMsgEnum;
use crate::system::SystemModules;
use tokio::sync::mpsc::*;
use tokio::sync::{Mutex};
use std::sync::Arc;
use msg_bus::{MsgBusHandle, Message};

pub struct Processor {
    // pub adapters: Arc<RwLock<HashMap<HandleId, Arc<Mutex<ClaHandle>>>>>,
    // conf: Arc<RwLock<Configuration>>,
    bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>,
    rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,

}

impl Processor {
    pub async fn new(mut bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>) -> Self {
        let rx = bus_handle.register(SystemModules::Processing).await.unwrap();

        Self {
            rx:  Arc::new(Mutex::new(rx)),
            bus_handle,
        }
    }

    pub async fn start(&mut self) {
        let rx = self.rx.clone();

        let _bus_handle = self.bus_handle.clone();

        while let Some(msg) = rx.lock().await.recv().await {
                // Listen for updates from CLAs
                match msg {
                    Message::Shutdown => { break; },
                    _ => {},
                };
            
                // tokio::spawn(async move {
                //     // Do something with the msg
                // });
        }; // end While
        debug!("Exited process loop");
 


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

