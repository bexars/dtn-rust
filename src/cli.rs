use log::*;
use shrust::{Shell, ShellIO};
use std::io::prelude::*;
use crate::bus::{ModuleMsgEnum, BusMessage};
use crate::router::RouterModule;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;

pub struct CliManager {
    bus_tx: Option<Sender<ModuleMsgEnum>>,
    tx: Sender<ModuleMsgEnum>,
    rx: Arc<Mutex<Receiver<ModuleMsgEnum>>>,

}

impl CliManager {
    pub fn new() -> Self {
        let (tx,rx) = channel::<ModuleMsgEnum>(50);

        Self {
            tx:  tx,
            rx:  Arc::new(Mutex::new(rx)),
            bus_tx:   None,
        }
    }
    pub fn start(&self, bus_tx: Sender<ModuleMsgEnum>)  -> tokio::task::JoinHandle<()> {
        let bus_tx = bus_tx.clone();
        tokio::task::spawn_blocking(|| start_shell(bus_tx))
    }
    
}

pub fn start_shell(bus_tx: Sender<ModuleMsgEnum>) {
    let mut bus_tx = bus_tx;
    let mut shell = Shell::new(());
    shell.new_command_noargs("hello", "Say 'hello' to the world", |io, _| {
        writeln!(io, "Hello World !!!")?;
        Ok(())
    });
    shell.new_command_noargs("halt", "Stop all processing and shutdown", move |io, _|  {
        writeln!(io, "Halting.")?;
        let mut tx = bus_tx.clone();
            tokio::spawn(async move {
                let mut tx = tx;
                tx.send(ModuleMsgEnum::ShutdownNow).await;
            }
            );
        Ok(())
    });

    shell.run_loop(&mut ShellIO::default());
}