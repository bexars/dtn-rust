use log::*;
use shrust::{Shell, ShellIO};
use std::io::prelude::*;
use crate::bus::{ModuleMsgEnum, BusMessage, send, rpc};
use crate::router::RouterModule;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
use msg_bus::{MsgBusHandle, Message};



pub struct CliManager {
    bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>,
    rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,

}

impl CliManager {
    pub async fn new(mut bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>) -> Self {
        let rx = bus_handle.register(RouterModule::CLI).await.unwrap();
        Self {
            rx:  Arc::new(Mutex::new(rx)),
            bus_handle,
        }
    }


    pub async fn start(self) {
        // bus_tx.send(ModuleMsgEnum::MsgBus(BusMessage::SetTx(tx.clone(), RouterModule::CLI))).await.unwrap();
        
        CliManager::start_shell(self.bus_handle.clone()).await;
    }
    

    pub async fn start_shell(bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>) {
        let mut shell = Shell::new(());
        let bus_handle = bus_handle.clone();
        shell.new_command_noargs("hello", "Say 'hello' to the world", |io, _| {
            writeln!(io, "Hello World !!!")?;
            Ok(())
        });
        shell.new_command_noargs("halt", "Stop all processing and shutdown", move |io, _|  {
            writeln!(io, "Halting.")?;
            let mut bus_handle = bus_handle.clone();
            bus_handle.blocking_send(RouterModule::Routing, ModuleMsgEnum::ShutdownNow);
            Ok(())
        });
        shell.new_command("show", "Display data", 1, move |io,_, arg| {
        //     let (tx, rx) = tokio::sync::oneshot::channel();
        //    let res = tokio::spawn(rpc())
            Ok(())
        });
        tokio::task::block_in_place(move || shell.run_loop(&mut ShellIO::default()));
    }
}