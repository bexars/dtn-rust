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
    rx: Arc<RwLock<Receiver<MetaBundle>>>,
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
            rx: Arc::new(RwLock::new(rx)),
            cla_config,
            cla,
        }
    }

    /// Should be called whenever a CLA leaves shutdown state

    async fn start_cla(&mut self, cla_tx: Sender<ClaBundleStatus>) {

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
        self.cla.write().await.start(cla_tx);

    }


    pub async fn start(&mut self) {

        let mut system_handle = self.bus_handle.clone().register(SystemModules::Cla(self.id)).await.unwrap();

        let routing_handle = crate::routing::router::add_cla_handle(&mut self.bus_handle.clone(), self.id, self.tx.clone()).await;
        self.router_handle = Some(routing_handle.clone());        

        let (cla_tx, mut cla_rx) = tokio::sync::mpsc::channel::<ClaBundleStatus>(50);
        if !self.cla_config.shutdown { self.start_cla(cla_tx.clone()).await; };
        
        let rx = &mut self.rx.clone();
        let mut rx = rx.write().await;
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
                Some(rcvd_bundle) = cla_rx.recv() => { // Received bundle from CLA
                    match rcvd_bundle {
                        ClaBundleStatus::New(_,_) => {
                            debug!("Received Bundle");
                            self.process_bundle(rcvd_bundle, routing_handle.clone());

                        }
                        _ => {},  // TODO Implement Failure, Success
                    };                      

                },
            };

        }
    }

    fn process_bundle<'a>(&mut  self, bundle: ClaBundleStatus, routing_handle: Sender<MetaBundle>)  {
        
        let (bundle, size) = match bundle {
            ClaBundleStatus::New(bundle, size) => { (bundle, size) },
            _ => { return; },
        };

        self.in_bundles += 1;
        self.in_bytes += size;
        
        let metabun = MetaBundle{ 
            dest: NodeRoute::from(&bundle),
            bundle,  
        };

        let mut routing_handle = routing_handle.clone();
        tokio::task::spawn(async move { routing_handle.send(metabun).await });

    }

}