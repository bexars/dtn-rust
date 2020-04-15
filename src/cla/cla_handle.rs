use log::*;
use std::sync::{Arc};
use super::ClaRW;
use super::{ClaTrait, ClaBundleStatus};
use crate::system::{ SystemModules};
use crate::routing::*;
use tokio::sync::mpsc::{Sender,Receiver};
use tokio::sync::RwLock;
use msg_bus::{Message, MsgBusHandle};
use crate::bus::ModuleMsgEnum;
use super::AdapterConfiguration;


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
    cla_config: AdapterConfiguration,
    cla: Arc<RwLock<Box<dyn ClaTrait>>>,
}

pub type HandleId = usize;


impl ClaHandle {
    pub fn new( id: HandleId, name: String, 
                        bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>, 
                        cla_config: AdapterConfiguration, cla_rw: ClaRW, cla: Arc<RwLock<Box<dyn ClaTrait>>>) -> ClaHandle 
        {
        debug!("Inside ClaHandle new");
        let (tx, rx) = tokio::sync::mpsc::channel(50);
        Self {
            id,
            name,
            bus_handle,
            rw: cla_rw,
            router_handle: None,
            in_bundles: 0,
            in_bytes: 0,
            out_bundles: 0,
            out_bytes: 0,
            tx,
            rx,
            cla_config,
            cla,
        }
    }


    pub async fn start(&mut self) {

        let mut system_handle = self.bus_handle.clone().register(SystemModules::Cla(self.id)).await.unwrap();

        let mut routing_handle = crate::routing::router::add_cla_handle(&mut self.bus_handle.clone(), self.id, self.tx.clone()).await;
        self.router_handle = Some(routing_handle.clone());        

        match &self.rw {
            ClaRW::RW | ClaRW::W => {
                // send the route to the router
                let rte = Route { 
                    dest: NodeRoute::from(&self.cla_config.peernode), 
                    nexthop: RouteType::ConvLayer(self.id),
                };
                router::add_route(&mut self.bus_handle, rte).await;
            }
            _ => {}
        };
        let rx = &mut self.rx;

        let (cla_tx, mut cla_rx) = tokio::sync::mpsc::channel::<ClaBundleStatus>(50);

        self.cla.write().await.start(cla_tx);
        loop {
            let _ = tokio::select! {
                Some(msg) = system_handle.recv() => {
                    match msg {
                        Message::Shutdown => {
                            self.cla.write().await.stop();
                            break;
                        },
                        _ => {},
                    }
                }
                Some(router_bun) = rx.recv() => { // Received bundle from Router 
                    self.cla.write().await.send(router_bun);
                },
                Some(cla_bun) = cla_rx.recv() => { // Received bundle from CLA
                    match cla_bun {
                        ClaBundleStatus::New(bundle) => {
                            debug!("Received Bundle");
                            let metabun = MetaBundle{ 
                                dest: NodeRoute::from(&bundle),
                                bundle,  
                            };
                            routing_handle.send(metabun).await;
                        }
                        _ => {},  // TODO Implement Failure, Success
                    };                      

                },
            };

        }
    }

    // pub async fn process_bundle(metabundle: Arc<MetaBundle>) {
    //     // update stats
    //     // send to the router
    
    // }
}