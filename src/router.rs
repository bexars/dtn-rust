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
use crate::bus;
use crate::conf;
use crate::cla::cla_manager::ClaManager;
use strum::{IntoEnumIterator};
use strum_macros::*;
// use std::path::PathBuf;

pub mod processor;



pub struct CmdLineOpts {
    pub config_file: String,
}

#[derive(EnumIter, Debug, PartialEq, Eq, Hash)]
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
}


#[tokio::main]
pub async fn start(conf_file: String) {

    //conf.store_file(&conf_file).unwrap();
    //println!("{}", toml::to_string_pretty(&conf).unwrap());

    let (mut msg_bus, bus_tx, bus_rx) = bus::Bus::new();
    let han_bus = msg_bus.start(bus_rx);

    let mut conf_mgr = conf::ConfManager::new(conf_file);
    let han_conf = conf_mgr.start(bus_tx.clone());

    // Storage here

    let mut proc_mgr = processor::Processor::new();
    let han_proc = proc_mgr.start(bus_tx.clone());
    

    
    let mut cla_mgr = ClaManager::new();
    let han_clam = cla_mgr.start(bus_tx.clone());

    let mut cli_mgr = cli::CliManager::new();
    let han_clim = cli_mgr.start(bus_tx.clone());

    //    let mut processor = Processor::new();        
//    task::spawn_blocking(|| {cli::start()});
//    processor.start().await;
    // cli::start_shell();
    info!("Waiting for threads");
    tokio::join!(han_clam, han_bus, han_conf, han_proc);
    // tokio::join!(han_bus, han_conf, han_proc, han_clam);   
    info!("All threads shut down.");
    tokio::join!(han_clim);    
    std::process::exit(0);
    

}