use tokio::net::{TcpListener, TcpStream};
use tokio::io::BufReader;
use tokio::prelude::*;
use futures::stream::StreamExt;
use bp7::Bundle;
use bp7::ByteBuffer;
use std::convert::TryFrom;
use crate::cla::cla_handle::ClaHandle;
use crate::cla::{ClaType, ClaRW, ClaTrait, ClaBundleStatus};
use tokio::sync::{Mutex, RwLock};
use std::sync::{Arc};
use tokio::sync::mpsc::Sender;
use crate::routing::MetaBundle;
use crate::stcp;


pub struct Stcp {
    address: String,
    port: u16,
}



impl Stcp {

    // pub const CLA_TYPE: ClaType = ClaType::StcpListener;
    pub const CLA_RW: ClaRW = ClaRW::W;

    pub fn new(address: String, port: u16 ) -> Stcp {
        Stcp {
            address,
            port,
        }
        
    }

}


impl ClaTrait for Stcp {

    fn start(&self, tx: Sender<ClaBundleStatus>) {
        // TODO handle this
    }

    fn stop(&self) { unimplemented!(); }
    fn send(&self, bundle: MetaBundle) { 

         let addr = format!("{}:{}", self.address, self.port);
        
        tokio::task::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();

            stream.write(&stcp::encapsulate_stcp(bundle.bundle)).await;
        });
    }

}
