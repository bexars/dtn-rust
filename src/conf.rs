use fondant::Configure;
use serde::{Serialize, Deserialize};
use bp7::EndpointID;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
use crate::bus::{ModuleMsgEnum, BusMessage};
use crate::router::RouterModule;

pub enum ConfMessage {
    GetFullConfig(RouterModule),
    PayloadFullConfig(Configuration),
    ConfigUpdated(RouterModule, Configuration),  // Try to be smart and tell which part of the config changed
}


#[derive(Configure, Serialize, Deserialize, Default)]
#[config_file = "config.toml"]
pub struct Configuration {
    pub stcp_port: u16,
    pub stcp_enable: bool,
    pub local_eid: EndpointID,
}


pub struct ConfManager {
    config: Arc<RwLock<Configuration>>,
    config_file: PathBuf,
    bus_tx: Option<Sender<ModuleMsgEnum>>,
    conf_tx: Sender<ModuleMsgEnum>,
    conf_rx: Arc<Mutex<Receiver<ModuleMsgEnum>>>,
}

impl ConfManager {
    pub fn new( config_file: String) -> Self {
        let (tx,rx) = channel::<ModuleMsgEnum>(50);
        let config_file = PathBuf::from(config_file);
        let config = Arc::new(RwLock::new(Configuration::load_file(&config_file).unwrap()));
        Self {
            config,
            config_file,
            bus_tx: None,
            conf_tx: tx,
            conf_rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn start(&mut self, bus_tx: Sender<ModuleMsgEnum>) {
        self.bus_tx = Some(bus_tx.clone());
        let rx = self.conf_rx.clone();
        let tx = self.conf_tx.clone();
        let mut bus_tx = bus_tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.lock().await.recv().await {
                // Listen for config updates and requests
            }
    
        });
        
        tokio::spawn( {
            async move {   
                let res = bus_tx.send(ModuleMsgEnum::MsgBus(
                                BusMessage::SetTx(
                                    tx.clone(), RouterModule::Configuration))).await;
            }});

    }    
}