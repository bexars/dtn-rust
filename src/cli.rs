use log::*;
use shrust::{Shell, ShellIO};
use std::io::prelude::*;
use crate::bus::ModuleMsgEnum::*;
use crate::router::RouterModule::*;
use crate::bus::ModuleMsgEnum;
use crate::router::RouterModule;
use msg_bus::{MsgBusHandle, Message};

mod terminal;

pub struct CliManager {
    bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>,
    //rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,

}

impl CliManager {
    pub async fn new( bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>) -> Self {
        // let rx = bus_handle.register(RouterModule::CLI);
        Self {
            // rx:  Arc::new(Mutex::new(rx)),
            bus_handle,
        }
    }


    pub fn start(self) { //  -> tokio::task::JoinHandle<()> {
        // bus_tx.send(ModuleMsgEnum::MsgBus(BusMessage::SetTx(tx.clone(), RouterModule::CLI))).await.unwrap();
        // let mut bus_handle = self.bus_handle.clone();
        debug!("In CliManager.start()");
        // let handle = tokio::task::spawn_blocking(move || CliManager::start_shell(self.bus_handle.clone()));
        let handle = tokio::task::spawn_blocking(move || self::terminal::start(self.bus_handle.clone()));

        // handle
//        CliManager::start_shell(self.bus_handle.clone()).await;
    }
    
}
