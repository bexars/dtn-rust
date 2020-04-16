use log::*;
use bp7::Bundle;
use crate::cla::cla_handle::{HandleId};
use crate::bus::ModuleMsgEnum;
use crate::system::{ SystemModules, BusHandle };
use tokio::sync::mpsc::*;
use tokio::sync::{Mutex, RwLock};
use std::sync::{Arc};
use msg_bus::{MsgBusHandle, Message};
use crate::routing::{ MetaBundle, MetaBundleStatus, RouteTableEntry };
use crate::routing::RoutingMessage;
use crate::conf::Configuration;



#[derive(Clone, Debug, PartialEq)]
pub enum ProcessorMsg {
    Process(MetaBundle),
}

#[derive(Clone)]
pub struct Processor {
    // pub adapters: Arc<RwLock<HashMap<HandleId, Arc<Mutex<ClaHandle>>>>>,
    // conf: Arc<RwLock<Configuration>>,
    bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>,
    rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,
    // config: Mutex<Arc<RwLock<Configuration>>>,

}

impl Processor {
    pub async fn new(mut bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>) -> Self {
        let rx = bus_handle.register(SystemModules::Processing).await.unwrap();

        Self {
            rx:  Arc::new(Mutex::new(rx)),
            bus_handle,
            // config: Mutex::new(Arc::new(RwLock::new(Configuration::default()))),
        }
    }

    pub async fn start(&self) {
        let rx = self.rx.clone();
        let mut bus_handle = self.bus_handle.clone();
        // let conf = crate::conf::get_conf_struct(&mut bus_handle).await.unwrap();


        while let Some(msg) = rx.lock().await.recv().await {
                // Listen for updates from CLAs
                match msg {
                    Message::Shutdown => { break; },
                    Message::Message(ModuleMsgEnum::MsgProcessing(proc_msg)) => {
                        match proc_msg {
                            ProcessorMsg::Process(metabun) => {
                                let s = self.clone();
                                tokio::task::spawn(async move { s.process_inbound(metabun).await });
                            }

                            _ => { debug!("Unhandled message: {:?}", proc_msg) },
                        }
                    },
                    _ => {},
                };
            
              
          

        }; // end While
        debug!("Exited process loop");
 


    }
    /// Processes inbound bundles.  It's Ellis Island
    pub async fn process_inbound(&self, mut metabun: MetaBundle) {
        let config = crate::conf::CONFIGURATION.load();

        // Process flags
            
        // TODO remove the hardcode EID
        // Add hopcount block if needed

        
        if  None == metabun.bundle.extension_block_by_type(bp7::HOP_COUNT_BLOCK) {
            let mut hop_block = bp7::new_hop_count_block(1, 0, 64);
            metabun.bundle.add_canonical_block(hop_block);
        }

        // Drop if ttl or time expired
        if !metabun.bundle.update_extensions(config.local_eid.clone(), 0 as u128) { 
            trace!("TTL exceeded");  //TODO respond to admin record if requested
            return; }

        // send to router
        let mut bh = self.bus_handle.clone();
        crate::routing::router::forward_bundle(&mut bh, metabun).await;
        
    }
}


// ***  Helper functions 

pub async fn process_bundle(mut bh: BusHandle, metabun: MetaBundle) {
    bh.send(SystemModules::Processing, ModuleMsgEnum::MsgProcessing(ProcessorMsg::Process(metabun))).await.unwrap();
}
