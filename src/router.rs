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
// use std::path::PathBuf;

pub mod processor;

#[derive(Configure, Serialize, Deserialize, Default)]
#[config_file = "config.toml"]
pub struct Configuration {
    pub stcp_port: u16,
    pub stcp_enable: bool,
    pub local_eid: EndpointID,
}

pub struct CmdLineOpts {
    pub config_file: String,
}


#[tokio::main]
pub async fn start(conf_file: String) {

    let conf_file = PathBuf::from(conf_file);
    let conf = Configuration::load_file(&conf_file).unwrap();


    


    //conf.store_file(&conf_file).unwrap();
    //println!("{}", toml::to_string_pretty(&conf).unwrap());

    let mut processor = Processor::new(Arc::new(conf));        
    task::spawn_blocking(|| {cli::start()});
    processor.start().await;


    // let stcp_server = match conf.stcp_enable {
    //     false => None,
    //     true => Some(stcp_server::StcpServer::new(conf.stcp_port)),
    // };   


    // if let Some(server) = stcp_server { server.start().await; };

}