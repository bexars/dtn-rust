use log::*;

use crate::processor::Processor;
use crate::cla::cla_handle::*;
use crate::conf::Configuration;
use super::stcp_server::StcpServer;
use std::collections::HashMap;
use bp7::Bundle;
use tokio::prelude::*;
use crate::bus::{ModuleMsgEnum};
use crate::system::SystemModules;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
use msg_bus::{MsgBusHandle, Message};





pub struct ClaManager {
    pub adapters: Arc<RwLock<HashMap<HandleId, Arc<Mutex<ClaHandle>>>>>,
    // conf: Arc<RwLock<Configuration>>,
    bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>,
    clam_rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,

}

impl ClaManager {

    pub async fn new(mut bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>) -> Self {
        let rx = bus_handle.register(SystemModules::ClaManager).await.unwrap();

        Self {
            clam_rx:  Arc::new(Mutex::new(rx)),
            bus_handle,
            adapters: Arc::new(RwLock::new(HashMap::<HandleId, Arc<Mutex<ClaHandle>>>::new())),
        }
    }

    async fn start_clas(&mut self) {
        let conf = crate::conf::get_cla_conf(&mut self.bus_handle.clone()).await;
        
        for cla_conf in conf.adapters {

        }
    }

    pub async fn start(&mut self) {
        let rx = self.clam_rx.clone();
        let mut bus_handle = self.bus_handle.clone();

        // Instantiate all the CLAs
        self.start_clas().await;
    
        while let Some(msg) = rx.lock().await.recv().await {
            // Listen for updates from CLAs
            debug!("Received msg: {:?}", msg);
            match msg {
                Message::Shutdown => { 
                    debug!("Received Halt");
                    break; },
                _ => {},
            }
        } // end While

        debug!("Exited cla_manager loop");
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