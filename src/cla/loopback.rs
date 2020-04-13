use super::cla_handle::ClaHandle;
use super::*;
use std::sync::Arc;

pub struct LoopbackCLA {
    config: AdapterConfiguration,
}

impl LoopbackCLA {
    pub fn new(handle: ClaHandle, config: super::AdapterConfiguration) -> LoopbackCLA {
        Self { 
            
            config, 
        }
    }
}

#[async_trait]
impl BundleReader for LoopbackCLA {
    async fn start(handle: ClaHandle) {
        // do nothing really
    }
}

#[async_trait]
impl BundleWriter for LoopbackCLA {
    async fn send(mbun: &Arc<MetaBundle>) {
        println!("Loopback received a bundle");
    }
}