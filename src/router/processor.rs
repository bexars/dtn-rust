use super::super::router;
use bp7::Bundle;
use crate::cla::cla_handle::{ClaHandle, HandleId};
use crate::cla::cla_manager::ClaManager;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, Receiver};
use tokio::prelude::*;



pub struct Processor {
    node: String,
    cla_manager: Arc<ClaManager>,
}

impl Processor {
    pub fn new(conf: Arc<router::Configuration>) -> Self {
        Self {
            node: conf.local_eid.node_id().unwrap(),
            cla_manager: Arc::new(ClaManager::new(conf)),
        }
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

    pub async fn start(&mut self) {
        println!("Starting processor for node: {}", self.node);

        println!("Building bundle loop");
        let cla_manager = self.cla_manager.clone();
        let (tx, rx) : (Sender<(HandleId,Bundle)>, Receiver< (HandleId, Bundle) >) = channel();

        cla_manager.start(tx);

        let process_loop = async move {
            loop {
                let (id, bun) = rx.recv().unwrap();

                // println!("Received bundle on: {}", &cla_manager.adapters.read().unwrap().get(&id).unwrap().lock().unwrap().name);
                // TODO Update stats on Cla Handle
                // &self.process_bundle(bun, id);
            }
        };
        process_loop.await;
//        tokio::spawn(process_loop);
        println!("Started bundle processing loop");




        // TODO Timer loop to check if bundles can be sent
    }
}