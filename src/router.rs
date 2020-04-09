use log::*;
use tokio::net::TcpListener;
use tokio::prelude::*;
use futures::stream::StreamExt;
use tokio::task;
use bp7::eid::EndpointID;
use processor::Processor;
use std::sync::Arc;
use fondant::Configure;
use serde::{Serialize, Deserialize};
use crate::cli;
use crate::bus::ModuleMsgEnum;
use crate::conf;
use crate::cla::cla_manager::ClaManager;
use strum::{IntoEnumIterator};
use strum_macros::*;
use msg_bus::{Message, MsgBus, MsgBusHandle};
use msg_bus::Message::*;    

// use std::path::PathBuf;

pub mod processor;

type BusHandle = MsgBusHandle<RouterModule, ModuleMsgEnum>;


pub struct CmdLineOpts {
    pub config_file: String,
}

#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
pub enum SystemMessage {
    ShutdownRequested,
}

#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
pub enum RouterModule {
    Processing,      // Actually reads the Bundle and decides what to do with it
    ClaManager,      // Manages the various CLA, stats, up/down
    CLI,             // User interface
    Logging,         // Catches and distributes all logging
    Storage,         // Bundles being written to disk
    AppAgent,        // Registering clients, send/receive bundles
    Routing,         // Updates and lookups to the forwarding table
    Configuration,   // Reads, stores, updates the config.  Let's other modules know
    Bus,             // The messaging backbone
    System,          // System to control the system
}

#[tokio::main]
//#[tokio::main(core_threads = 2)]
pub async fn start(conf_file: String) {

    //conf.store_file(&conf_file).unwrap();
    //println!("{}", toml::to_string_pretty(&conf).unwrap());

    let (bus, mut bus_handle) = MsgBus::<RouterModule, ModuleMsgEnum>::new();

    let mut rx = bus_handle.clone().register(RouterModule::System).await.unwrap();
    // let (mut msg_bus_old, bus_tx, bus_rx) = bus::Bus::new();
    // let han_bus = msg_bus_old.start(bus_rx);

    let mut conf_mgr = conf::ConfManager::new(conf_file, bus_handle.clone()).await;
    // Storage here
    let mut proc_mgr = processor::Processor::new(bus_handle.clone()).await;
    let mut cla_mgr = ClaManager::new(bus_handle.clone()).await;
    let mut cli_mgr = cli::CliManager::new(bus_handle.clone()).await;

    let han_conf = task::spawn(async move { conf_mgr.start().await; });
    let han_proc = task::spawn(async move { proc_mgr.start().await; });
    let han_clam = task::spawn(async move { cla_mgr.start().await; });
    let han_clim = cli_mgr.start();

    //    let mut processor = Processor::new();        
//    task::spawn_blocking(|| {cli::start()});
//    processor.start().await;
    // cli::start_shell();
//     info!("Waiting for threads");
// //    tokio::join!(han_clam, han_conf, han_proc);
//     // tokio::join!(han_bus, han_conf, han_proc, han_clam);   
//     info!("All threads shut down.");
    // tokio::join!(han_clim);    

    trace!("About to enter router loop");
    while let Some(msg) = rx.recv().await {
        match msg {
            Message(ModuleMsgEnum::MsgSystem(SystemMessage::ShutdownRequested)) => {
                debug!("Received shutdown");
                // bus_handle.broadcast(ModuleMsgEnum::ShutdownNow).await;
                bus.clone().shutdown().await; 
                break;
            },
            _ => {},
        }
    }
}

pub async fn halt(bh: &mut BusHandle) {
    bh.send(RouterModule::System, ModuleMsgEnum::MsgSystem(SystemMessage::ShutdownRequested)).await;
}
