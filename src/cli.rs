use log::*;
use crate::bus::ModuleMsgEnum;
use crate::system::SystemModules;
use msg_bus::{MsgBusHandle, Message};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::Receiver;
use tokio::sync::{Mutex};
use std::sync::Arc;

mod terminal;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CliConfiguration {

}
impl Default for CliConfiguration {
    fn default() -> Self { 
        Self {
        }
    }
}

#[derive(Clone)]
pub struct CliManager {
    bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>,
    rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,
    cli_conf: CliConfiguration,
}

impl CliManager {
    pub async fn new(mut bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>) -> Self {
        let rx = bus_handle.register(SystemModules::CLI).await.unwrap();
        Self {
            rx:  Arc::new(Mutex::new(rx)),
            bus_handle,
            cli_conf: CliConfiguration::default(),
        }
    }

    pub async fn start(self) { 
        debug!("In CliManager.start()");

        let bh = self.bus_handle.clone();
        let orig_conf = crate::conf::get_cli_conf(&mut bh.clone()).await;
        let clim = self.clone();
        
        let _handle = tokio::task::spawn_blocking(move || self::terminal::start(bh.clone()));

        let rx = self.rx.clone();
        while let Some(msg) = rx.lock().await.recv().await {
            match msg {
                Message::Message(ModuleMsgEnum::MsgConf(crate::conf::ConfMessage::DataConfigCli(conf_cli))) => {
                    let clim = self.clone();
                    // TODO Think if we need to do something
                },
                _ => {}
            }
        }

    }
    
}
