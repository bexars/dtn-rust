use std::sync::{Arc, Mutex};
use crate::router::processor::Processor;
use crate::cla::cla_handle::*;
use crate::router::Configuration;
use super::stcp_server::StcpServer;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};
use bp7::Bundle;
use tokio::prelude::*;




pub struct ClaManager {
    pub adapters: Arc<Mutex<HashMap<HandleID, Arc<Mutex<ClaHandle>>>>>,
    conf: Arc<Configuration>,
}

impl ClaManager {
    pub fn new(conf: Arc<Configuration>) -> ClaManager {
        Self {
            adapters: Arc::new(
                Mutex::new(HashMap::<HandleID, Arc<Mutex<ClaHandle>>>::new(),
            )),
            conf,
        }
    }

    pub async fn start(&self, tx: Sender<(HandleID, Bundle)>) {
        let mut cur_id: HandleID = 0;  
        let mut inc_id =  || {
            cur_id += 1;
            cur_id - 1
        };

        let tx = tx.clone();

        let stcp_server = match self.conf.stcp_enable {
            false => None,
            true => {
                let id = inc_id();
                let handle = ClaHandle::new(
                    id, 
                    "StcpListener0".to_string(), 
                    StcpServer::CLA_RW,
                    StcpServer::CLA_TYPE );
                let handle = Arc::new(Mutex::new(handle));
                self.adapters.lock().unwrap().insert(id, handle.clone());
                Some(StcpServer::new(handle, self.conf.stcp_port))
            }
        };
        
        if let Some(server) = stcp_server { server.start(tx); };
        println!("CLA_Manager finisehed invoking STCP server");
    }
}