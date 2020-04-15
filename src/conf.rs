use log::*;
use fondant::Configure;
use serde::{Serialize, Deserialize};
use bp7::EndpointID;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
use crate::system::SystemModules;
use msg_bus::{MsgBusHandle, Message};
use crate::bus::ModuleMsgEnum;
use crate::cla::{ClaConfiguration, AdapterConfiguration};
use crate::cli::CliConfiguration;


#[derive(Debug, Clone, PartialEq)]
pub enum ConfMessage {
    ConfigUpdated(SystemModules, Configuration),  // Try to be smart and tell which part of the config changed
    Save(Option<String>),
    DataConfigStruct(Configuration),
    DataConfigCli(CliConfiguration),
    DataConfigCla(ClaConfiguration),
    DataConfigString(String),
    DelConfigCla(String),
    GetConfigString,
    GetConfigStruct,
    GetConfigCli,
    GetConfigCla,
    GetNodeName,
    SetConfigCli(CliConfiguration),
    SetConfigCla(AdapterConfiguration),
    SetNodeName(String),
}


#[derive(Configure, Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[config_file = "config.json"]
pub struct Configuration {
    pub nodename: String,
    pub local_eid: EndpointID,
    pub cla: ClaConfiguration,
    pub cli: CliConfiguration,
}


pub struct ConfManager {
    config: Arc<RwLock<Configuration>>,
    // config_file: PathBuf,  // TODO save to the correct path
    bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>,
    conf_rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,
}

type BusHandle = MsgBusHandle<SystemModules, ModuleMsgEnum>;

impl ConfManager {
    pub async fn new(config_file: PathBuf, mut bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>) -> Self {
        let rx = bus_handle.register(SystemModules::Configuration).await.unwrap();
        let config_file = PathBuf::from(config_file);
        let config = Configuration::load_file(&config_file);
        let config: Configuration = match config {
            Err(e) => {
                eprintln!("Unable to load configuration file:{:?} {:?}", config_file, e);
                eprintln!("Default config being used");
                Default::default() }
            Ok(conf) => conf
        };
        let config = Arc::new(RwLock::new(config));
        Self {
            config,
            // config_file,
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
                        ConfMessage::SetNodeName(nodename) => {
                            let mut conf = self.config.write().await;
                            conf.nodename = nodename;
                            rcpt.send(ModuleMsgEnum::MsgOk("".to_string())).unwrap();
                        }
                        ConfMessage::GetNodeName => {
                            let conf = self.config.read().await;
                            rcpt.send(ModuleMsgEnum::MsgOk(conf.nodename.clone())).unwrap();
                        }
                        ConfMessage::GetConfigString => {
                            let conf = self.config.read().await;
                            let conf_str = toml::to_string_pretty(&*conf).unwrap();
                            rcpt.send(ModuleMsgEnum::MsgConf(ConfMessage::DataConfigString(conf_str))).unwrap();
                        },
                        ConfMessage::GetConfigCli => {
                            rcpt.send(ModuleMsgEnum::MsgConf(ConfMessage::DataConfigCli(self.config.read().await.cli.clone()))).unwrap();
                        },
                        ConfMessage::GetConfigCla => {
                            rcpt.send(ModuleMsgEnum::MsgConf(ConfMessage::DataConfigCla(self.config.read().await.cla.clone()))).unwrap();
                        },
                        ConfMessage::SetConfigCli(cli_conf) => {
                            self.config.write().await.cli = cli_conf.clone();
                            bus_handle.send(SystemModules::CLI, ModuleMsgEnum::MsgConf(ConfMessage::DataConfigCli(cli_conf))).await.unwrap();
                            rcpt.send(ModuleMsgEnum::MsgOk("".to_string())).unwrap();
                        },
                        ConfMessage::SetConfigCla(cla_conf) => {
                            self.config.write().await.cla.adapters.insert(cla_conf.name.clone(),  cla_conf.clone());
                            bus_handle.broadcast(ModuleMsgEnum::MsgConf(ConfMessage::ConfigUpdated(SystemModules::ClaManager, self.config.read().await.clone()))).await.unwrap();
                            rcpt.send(ModuleMsgEnum::MsgOk("".to_string())).unwrap();
                        },
                        ConfMessage::DelConfigCla(cla_name) => {
                            self.config.write().await.cla.adapters.remove(&cla_name);
                            bus_handle.broadcast(ModuleMsgEnum::MsgConf(ConfMessage::ConfigUpdated(SystemModules::ClaManager, self.config.read().await.clone()))).await.unwrap();
                            rcpt.send(ModuleMsgEnum::MsgOk("".to_string())).unwrap();
                        },

                        ConfMessage::Save(file_name) => {
                            async fn run(config: &Arc<RwLock<Configuration>>, file_name: Option<String>) -> Result<(), FondantError> {
                                if let Some(file_name) = file_name {
                                    let pb = PathBuf::from(file_name);
                                    config.read().await.store_file(&pb)?;
                                } else {
                                    config.read().await.store()?;
                                }
                                Ok(())
                            }
                            if let Err(e) = run(&self.config, file_name).await {
                                rcpt.send(ModuleMsgEnum::MsgErr(format!("Error saving: {:?}",e).to_string())).unwrap();
                            } else {
                                rcpt.send(ModuleMsgEnum::MsgOk("".to_string())).unwrap();
                            };
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
    let conf = bh.rpc(SystemModules::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::GetConfigString)).await;
    if let Ok(ModuleMsgEnum::MsgConf(ConfMessage::DataConfigString(conf_str))) = conf {
        return conf_str;
    } 
            "Test".to_owned()
}

pub async fn get_nodename(bh: &mut BusHandle) -> String {
    let conf = bh.rpc(SystemModules::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::GetNodeName)).await;
    if let Ok(ModuleMsgEnum::MsgOk(nodename)) = conf {
        return nodename;
    } 
            "set_nodename".to_owned()
}

pub async fn save(bh: &mut BusHandle, file_name: Option<String>) -> Result<(), String> {
    let res = bh.rpc(SystemModules::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::Save(file_name))).await;
    match res {
        Ok(ModuleMsgEnum::MsgOk(_)) => { Ok(()) },
        Ok(ModuleMsgEnum::MsgErr(e)) => { Err(e) },
        _ => {Err("Unknown failure".to_owned()) },
    }
}




pub async fn get_cli_conf(bh: &mut BusHandle) -> CliConfiguration {
    let res = bh.rpc(SystemModules::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::GetConfigCli)).await;
    match res {
        Ok(ModuleMsgEnum::MsgConf(ConfMessage::DataConfigCli(conf))) => { conf },
        _ => { CliConfiguration::default() },
    }
}

pub async fn set_nodename(bh: &mut BusHandle, nodename: String)  -> Result<(), String> {
    let res = bh.rpc(SystemModules::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::SetNodeName(nodename))).await;
    match res {
        Ok(ModuleMsgEnum::MsgOk(_)) => { Ok(()) },
        Ok(ModuleMsgEnum::MsgErr(e)) => { Err(e) },
        _ => {Err("Unknown failure".to_owned()) },
    }
}

pub async fn set_cli_conf(bh: &mut BusHandle, cli_conf: CliConfiguration)  -> Result<(), String> {
    let res = bh.rpc(SystemModules::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::SetConfigCli(cli_conf))).await;
    match res {
        Ok(ModuleMsgEnum::MsgOk(_)) => { Ok(()) },
        Ok(ModuleMsgEnum::MsgErr(e)) => { Err(e) },
        _ => {Err("Unknown failure".to_owned()) },
    }
}

pub async fn get_cla_conf(bh: &mut BusHandle) -> ClaConfiguration {
    let res = bh.rpc(SystemModules::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::GetConfigCla)).await;
    match res {
        Ok(ModuleMsgEnum::MsgConf(ConfMessage::DataConfigCla(conf))) => { conf },
        _ => { ClaConfiguration::default() },
    }
}

pub async fn set_cla_conf(bh: &mut BusHandle, cla_conf: AdapterConfiguration)  -> Result<(), String> {
    let res = bh.rpc(SystemModules::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::SetConfigCla(cla_conf))).await;
    match res {
        Ok(ModuleMsgEnum::MsgOk(_)) => { Ok(()) },
        Ok(ModuleMsgEnum::MsgErr(e)) => { Err(e) },
        _ => {Err("Unknown failure".to_owned()) },
    }
}

pub async fn del_cla_conf(bh: &mut BusHandle, cla_name: String)  -> Result<(), String> {
    let res = bh.rpc(SystemModules::Configuration, ModuleMsgEnum::MsgConf(ConfMessage::DelConfigCla(cla_name))).await;
    match res {
        Ok(ModuleMsgEnum::MsgOk(_)) => { Ok(()) },
        Ok(ModuleMsgEnum::MsgErr(e)) => { Err(e) },
        _ => {Err("Unknown failure".to_owned()) },
    }
}