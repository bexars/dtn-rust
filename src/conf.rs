use log::*;
use fondant::Configure;
use serde::{Serialize, Deserialize};
use bp7::EndpointID;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
use crate::router::RouterModule;
use msg_bus::{MsgBusHandle, Message};
use crate::bus::ModuleMsgEnum;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfMessage {
    GetConfigString,
    GetConfigStruct,
    DataConfigStruct(Configuration),
    DataConfigString(String),
    ConfigUpdated(RouterModule, Configuration),  // Try to be smart and tell which part of the config changed
}


#[derive(Configure, Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[config_file = "config.toml"]
pub struct Configuration {
    pub stcp_port: u16,
    pub stcp_enable: bool,
    pub local_eid: EndpointID,
}


pub struct ConfManager {
    config: Arc<RwLock<Configuration>>,
    config_file: PathBuf,
    bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>,
    conf_rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,
}

type BusHandle = MsgBusHandle<RouterModule, ModuleMsgEnum>;

impl ConfManager {
    pub async fn new(config_file: String, mut bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>) -> Self {
        let rx = bus_handle.register(RouterModule::Configuration).await.unwrap();
        let config_file = PathBuf::from(config_file);
        let config = Arc::new(RwLock::new(Configuration::load_file(&config_file).unwrap()));
        Self {
            config,
            config_file,
            bus_handle: bus_handle,
            conf_rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub async fn start(&mut self) {
        let rx = self.conf_rx.clone();
        let mut bus_handle = self.bus_handle.clone();


        while let Some(msg) = rx.lock().await.recv().await {
            match msg {
                Message::Shutdown => {
                    debug!("Received Shutdown");
                    break;
                },
                Message::Rpc(ModuleMsgEnum::MsgConf(GetConfigString), sender) => {
                    let mut conf = self.config.write().await;
                    let conf_str = toml::to_string_pretty(&*conf).unwrap();
                    sender.send(ModuleMsgEnum::MsgConf(ConfMessage::DataConfigString(conf_str)));
                },
                _ => { trace!("Received unknown msg: {:?}", msg); },
            }
    
            // Listen for config updates and requests
        }
    
        

    }
}

pub async fn get_conf(bh: &mut BusHandle) -> String {
    let conf = bh.rpc(RouterModule::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::GetConfigString)).await;
    if let Ok(ModuleMsgEnum::MsgConf(ConfMessage::DataConfigString(conf_str))) = conf {
        return conf_str;
    } 
    
        "Test".to_owned()
}

