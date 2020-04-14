use log::*;

// use crate::processor::Processor;
use crate::cla::cla_handle::*;
use super::*;
// use crate::conf::Configuration;
// use super::stcp_server::StcpServer;
use std::collections::HashMap;
// use bp7::Bundle;
use tokio::prelude::*;
use crate::bus::{ModuleMsgEnum};
use crate::system::SystemModules;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
use msg_bus::{MsgBusHandle, Message};





pub struct ClaManager {
    pub adapters: Arc<RwLock<HashMap<HandleId, Arc<RwLock<ClaHandle>>>>>,
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
            adapters: Arc::new(RwLock::new(HashMap::<HandleId, Arc<RwLock<ClaHandle>>>::new())),
        }
    }
    fn init_cla(&self, i: HandleId, name: String, cla_conf: AdapterConfiguration) -> Arc<RwLock<ClaHandle>> {
        let (rw, cla) = match cla_conf.cla_type {
            ClaType::LoopBack => { (ClaRW::RW, loopback::LoopbackCLA::new(cla_conf.clone(), self.bus_handle.clone())) },
            _ => { (ClaRW::RW, loopback::LoopbackCLA::new(cla_conf.clone(), self.bus_handle.clone())) },
        };
        let handle = Arc::new(RwLock::new(ClaHandle::new(i, 
                            name.clone(),
                            self.bus_handle.clone(), 
                            cla_conf.clone(), 
                            rw, 
                            Arc::new(RwLock::new(Box::new(cla))),
        )));
        handle

    }

    async fn start_clas(&self) {
        debug!("start_clas");
        let mut adapters = self.adapters.write().await;
        for (_, h) in adapters.iter_mut() {
            ClaHandle::start(&h).await;
        }
    }

    async fn init_clas(&mut self) {
        let conf = crate::conf::get_cla_conf(&mut self.bus_handle.clone()).await;
        let mut adapters = self.adapters.write().await;
        let mut i: HandleId = 0;
        debug!("init_clas");

        for (name, cla_conf) in conf.adapters {
            let h = self.init_cla(i, name.clone(), cla_conf.clone());
            adapters.insert(i, h);
            i += 1;
        };
    }

    pub async fn start(&mut self) {
        let mut bus_handle = self.bus_handle.clone();

        // Instantiate all the CLAs
        self.init_clas().await;
        self.start_clas().await;
    
        let mut rx = self.clam_rx.lock().await;
        while let Some(msg) = rx.recv().await {
            // Listen for updates from CLAs
            // debug!("Received msg: {:?}", msg);
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