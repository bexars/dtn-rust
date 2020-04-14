use log::*;
use std::sync::{Arc};
use crate::processor::Processor;
use super::ClaRW;
use super::ClaType;
use super::{ClaTrait, ClaHandleTrait};
use crate::system::SystemModules;
use crate::routing::*;
use tokio::sync::mpsc::{Sender,Receiver};
use tokio::sync::RwLock;
use msg_bus::{MsgBus, MsgBusHandle};
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
    cla: Arc<RwLock<Box<ClaTrait>>>,
}

pub type HandleId = usize;


impl ClaHandle {
    pub fn new( id: HandleId, name: String, 
                        bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>, 
                        cla_config: AdapterConfiguration, cla_rw: ClaRW, cla: Arc<RwLock<Box<ClaTrait>>>) -> ClaHandle 
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


    pub async fn start(mut self_handle:  &Arc<RwLock<ClaHandle>>) {
        let self_handle = self_handle.clone();
 
        tokio::task::spawn(async move {
            let mut self_handle = &mut self_handle.write().await;
            let routing_handle = crate::routing::router::add_cla_handle(&mut self_handle.bus_handle.clone(), self_handle.id, self_handle.tx.clone()).await;
            self_handle.router_handle = Some(routing_handle);        

            match &self_handle.rw {
                ClaRW::RW | ClaRW::W => {
                    // send the route to the router
                    let rte = Route { 
                        dest: NodeRoute::from(&*self_handle.cla_config.peernode), 
                        nexthop: RouteType::ConvLayer(self_handle.id),
                    };
                    router::add_route(&mut self_handle.bus_handle, rte).await;
                }
                _ => {}
            };
            let rx = &self_handle.rx;
            self_handle.cla.write().await.start();
            loop {
                let bundle = self_handle.rx.recv().await;
            }
        });
    }

    pub async fn process_bundle(metabundle: Arc<MetaBundle>) {
        // update stats
        // send to the router
    }
}