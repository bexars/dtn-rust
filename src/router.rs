use tokio::net::TcpListener;
use tokio::prelude::*;
use futures::stream::StreamExt;
use bp7::eid::EndpointID;

pub mod stcp_server;

pub struct Configuration {
    pub stcp_port: u16,
    pub stcp_enable: bool,
    pub local_eid: EndpointID,
}


#[tokio::main]
pub async fn start(conf: Configuration) {
    let stcp_server = match conf.stcp_enable {
        false => None,
        true => Some(stcp_server::StcpServer::new(conf.stcp_port)),
    };


    if let Some(server) = stcp_server { server.start().await; };

}