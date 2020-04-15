use log::*;
use super::*;
use crate::system::BusHandle;
use tokio::sync::mpsc::Sender;

pub struct LoopbackCLA {
    config: AdapterConfiguration,
    _bus_handle: BusHandle,
}

impl LoopbackCLA {
    pub fn new(config: super::AdapterConfiguration, bus_handle: BusHandle ) -> LoopbackCLA {
        
        Self { 
            _bus_handle: bus_handle,
            config, 
            
        }
    }
}

impl ClaTrait for LoopbackCLA {
    fn start(&mut self, _tx: Sender<ClaBundleStatus>) {
        debug!("Loopback Started");
        
        // do nothing really.  Would loop on a real CLA
    }
    fn send(&mut self, mbun: MetaBundle) {
        debug!("Loopback {} received a bundle", self.config.name );
        println!("Bundle from: {}", mbun.bundle.primary.source);
        if let Some(payload) = mbun.bundle.payload() {
            println!("{}", String::from_utf8(payload.to_vec()).unwrap());
        }
        // TODO Send bundle to the local agent
    }

    fn stop(&mut self) {
        unimplemented!();
    }
}

