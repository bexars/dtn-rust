use log::*;
use super::cla_handle::ClaHandle;
use super::*;
use std::sync::Arc;
use super::ClaHandleTrait;
use crate::system::BusHandle;

pub struct LoopbackCLA {
    config: AdapterConfiguration,
    bus_handle: BusHandle,
    cla_handle: Option<Box<ClaHandleTrait>>,  
}

impl LoopbackCLA {
    pub fn new(config: super::AdapterConfiguration, bus_handle: BusHandle ) -> LoopbackCLA {
        Self { 
            bus_handle: bus_handle,
            config, 
            cla_handle: None,
        }
    }
}

impl ClaTrait for LoopbackCLA {
    fn start(&self) {
        debug!("Loopback Started");
        // do nothing really.  Would loop on a real CLA
    }
    fn send(&self, mbun: Arc<RwLock<MetaBundle>>) {
        debug!("Loopback received a bundle");
        // TODO Send bundle to the local agent
    }
}

