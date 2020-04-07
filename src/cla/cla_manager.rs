use log::*;

use crate::router::processor::Processor;
use crate::cla::cla_handle::*;
use crate::conf::Configuration;
use super::stcp_server::StcpServer;
use std::collections::HashMap;
use bp7::Bundle;
use tokio::prelude::*;
use crate::bus::{ModuleMsgEnum, BusMessage};
use crate::router::RouterModule;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
use msg_bus::{MsgBusHandle, Message};





pub struct ClaManager {
    pub adapters: Arc<RwLock<HashMap<HandleId, Arc<Mutex<ClaHandle>>>>>,
    // conf: Arc<RwLock<Configuration>>,
    bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>,
    clam_rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,

}

impl ClaManager {

    pub async fn new(mut bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>) -> Self {
        let rx = bus_handle.register(RouterModule::ClaManager).await.unwrap();

        Self {
            clam_rx:  Arc::new(Mutex::new(rx)),
            bus_handle,
            adapters: Arc::new(RwLock::new(HashMap::<HandleId, Arc<Mutex<ClaHandle>>>::new())),
        }
    }

    pub async fn start(&mut self) {
        let rx = self.clam_rx.clone();
        let mut bus_handle = self.bus_handle.clone();
 
    
        while let Some(msg) = rx.lock().await.recv().await {
            // Listen for updates from CLAs
            debug!("Received msg: {:?}", msg);
            match msg {
                Message::Shutdown => { 
                    debug!("Received Halt");
                    break; },
                _ => {},
            }


            tokio::spawn(async move {
                // Do something with the msg
            });
        } // end While
        // });  // end spawn

     

    }

    // pub async fn old_start(&self, tx: Sender<(HandleId, Bundle)>) {
    //     let mut cur_id: HandleId = 0;  
    //     let mut inc_id =  || {
    //         cur_id += 1;
    //         cur_id - 1
    //     };

    //     let tx = tx.clone();

    //     let stcp_server = match self.conf.read().await.stcp_enable {
    //         false => None,
    //         true => {
    //             let id = inc_id();
    //             let handle = ClaHandle::new(
    //                 id, 
    //                 "StcpListener0".to_string(), 
    //                 StcpServer::CLA_RW,
    //                 StcpServer::CLA_TYPE );
    //             let handle = Arc::new(Mutex::new(handle));
    //             self.adapters.write().await.insert(id, handle.clone());
    //             Some(StcpServer::new(handle, self.conf.read().await.stcp_port))
    //         }
    //     };
        
    //     if let Some(server) = stcp_server { server.start(tx); };
    //     println!("CLA_Manager finisehed invoking STCP server");
    // }
}