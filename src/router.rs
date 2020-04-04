use tokio::net::TcpListener;
use tokio::prelude::*;
use futures::stream::StreamExt;
use bp7::eid::EndpointID;
use processor::Processor;
use std::sync::Arc;

pub mod processor;

pub struct Configuration {
    pub stcp_port: u16,
    pub stcp_enable: bool,
    pub local_eid: EndpointID,
}


#[tokio::main]
pub async fn start(conf: Configuration) {

    let mut processor =  Processor::new(Arc::new(conf));

    processor.start().await;


    // let stcp_server = match conf.stcp_enable {
    //     false => None,
    //     true => Some(stcp_server::StcpServer::new(conf.stcp_port)),
    // };


    // if let Some(server) = stcp_server { server.start().await; };

}