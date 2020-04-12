use log::*;
use shrust::{Shell, ShellIO};
use std::io::prelude::*;
use crate::bus::ModuleMsgEnum::*;
use crate::router::RouterModule::*;
use crate::bus::ModuleMsgEnum;
use crate::router::RouterModule;
use msg_bus::{MsgBusHandle, Message};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::Receiver;
use tokio::sync::{Mutex};
use std::sync::Arc;


mod terminal;
// mod telnet_service;

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
    bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>,
    rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,
    cli_conf: CliConfiguration,
    // telnet_handle: Option<telnet_service::TelnetService>,
}

impl CliManager {
    pub async fn new(mut bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>) -> Self {
        let rx = bus_handle.register(RouterModule::CLI).await.unwrap();
        Self {
            rx:  Arc::new(Mutex::new(rx)),
            bus_handle,
            cli_conf: CliConfiguration::default(),
            // telnet_handle: None,
        }
    }

    async fn conf_updated(mut self, cli_conf: &CliConfiguration) {
        trace!("Old conf: {:?}", self.cli_conf);
        trace!("New conf: {:?}", cli_conf);
        // if self.cli_conf.telnet_enabled == cli_conf.telnet_enabled {
        //     if self.cli_conf.telnet_enabled == false  { return; };
        //     if self.cli_conf.telnet_port == cli_conf.telnet_port &&
        //        self.cli_conf.telnet_address == cli_conf.telnet_address { return; };
        // };

        // self.telnet_handle = Some(telnet_service::TelnetService::start(cli_conf, self.bus_handle.clone()).await);
           

    } 


    pub async fn start(self) { //  -> tokio::task::JoinHandle<()> {
        // bus_tx.send(ModuleMsgEnum::MsgBus(BusMessage::SetTx(tx.clone(), RouterModule::CLI))).await.unwrap();
        // let mut bus_handle = self.bus_handle.clone();
        debug!("In CliManager.start()");



        // let handle = tokio::task::spawn_blocking(move || CliManager::start_shell(self.bus_handle.clone()));
        let mut bh = self.bus_handle.clone();
        let orig_conf = crate::conf::get_cli_conf(&mut bh.clone()).await;
        let clim = self.clone();
        &clim.conf_updated(&orig_conf).await;



        debug!("About to spawn terminal");
        let handle = tokio::task::spawn_blocking(move || self::terminal::start(None, bh.clone()));
        debug!("spawn terminal returned");

        let rx = self.rx.clone();
        while let Some(msg) = rx.lock().await.recv().await {
            match msg {
                Message::Message(ModuleMsgEnum::MsgConf(crate::conf::ConfMessage::DataConfigCli(conf_cli))) => {
                    let clim = self.clone();
                    clim.conf_updated(&conf_cli).await;
                },
                _ => {}
            }
        }
        // handle
//        CliManager::start_shell(self.bus_handle.clone()).await;
    }
    
}
