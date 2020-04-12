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
use crate::cla::ClaConfiguration;
use crate::cli::CliConfiguration;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfMessage {
    GetConfigString,
    GetConfigStruct,
    GetConfigCli,
    Save(Option<String>),
    DataConfigStruct(Configuration),
    DataConfigCli(CliConfiguration),
    DataConfigString(String),
    ConfigUpdated(RouterModule, Configuration),  // Try to be smart and tell which part of the config changed
    SetConfigCli(CliConfiguration),
}


#[derive(Configure, Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[config_file = "config.toml"]
pub struct Configuration {
    pub stcp_port: u16,
    pub stcp_enable: bool,
    pub local_eid: EndpointID,
    pub cla: Vec<ClaConfiguration>,
    pub cli: CliConfiguration,
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
                Message::Rpc(ModuleMsgEnum::MsgConf(call),rcpt) => {
                    match call {
                        ConfMessage::GetConfigString => {
                            let conf = self.config.read().await;
                            let conf_str = toml::to_string_pretty(&*conf).unwrap();
                            rcpt.send(ModuleMsgEnum::MsgConf(ConfMessage::DataConfigString(conf_str)));
                        },
                        ConfMessage::GetConfigCli => {
                            rcpt.send(ModuleMsgEnum::MsgConf(ConfMessage::DataConfigCli(self.config.read().await.cli.clone())));
                        },
                        ConfMessage::SetConfigCli(cli_conf) => {
                            self.config.write().await.cli = cli_conf.clone();
                            bus_handle.send(RouterModule::CLI, ModuleMsgEnum::MsgConf(ConfMessage::DataConfigCli(cli_conf))).await;
                            rcpt.send(ModuleMsgEnum::MsgOk("".to_string()));
                        },
                        ConfMessage::Save(file_name) => {
                            if let Some(file_name) = file_name {
                                let pb = PathBuf::from(file_name);
                                self.config.read().await.store_file(&pb);
                                rcpt.send(ModuleMsgEnum::MsgOk("".to_string()));   
                            } else {
                                self.config.read().await.store();
                                rcpt.send(ModuleMsgEnum::MsgOk("".to_string()));
                            }
                        }
                        _ => {},
                    };
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

pub async fn save(bh: &mut BusHandle, file_name: Option<String>) -> Result<(), String> {
    let res = bh.rpc(RouterModule::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::Save(file_name))).await;
    match res {
        Ok(ModuleMsgEnum::MsgOk(_)) => { Ok(()) },
        Ok(ModuleMsgEnum::MsgErr(e)) => { Err(e) },
        _ => {Err("Unknown failure".to_owned()) },
    }
}

pub async fn get_cli_conf(bh: &mut BusHandle) -> CliConfiguration {
    let res = bh.rpc(RouterModule::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::GetConfigCli)).await;
    match res {
        Ok(ModuleMsgEnum::MsgConf(ConfMessage::DataConfigCli(conf))) => { conf },
        _ => { CliConfiguration::default() },
    }
}

pub async fn set_cli_conf(bh: &mut BusHandle, cli_conf: CliConfiguration)  -> Result<(), String> {
    let res = bh.rpc(RouterModule::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::SetConfigCli(cli_conf))).await;
    match res {
        Ok(ModuleMsgEnum::MsgOk(_)) => { Ok(()) },
        Ok(ModuleMsgEnum::MsgErr(e)) => { Err(e) },
        _ => {Err("Unknown failure".to_owned()) },
    }

}