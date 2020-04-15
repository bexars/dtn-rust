use tokio::net::{TcpStream};
use tokio::prelude::*;
use crate::cla::{ClaRW, ClaTrait, ClaBundleStatus};
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

    fn start(&mut self, _tx: Sender<ClaBundleStatus>) {
        // TODO handle this
    }

    fn stop(&mut self) { unimplemented!(); }
    fn send(&mut self, bundle: MetaBundle) { 

         let addr = format!("{}:{}", self.address, self.port);
        
        tokio::task::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap(); //TODO Handle bad connection

            stream.write(&stcp::encapsulate_stcp(bundle.bundle)).await.unwrap(); //TODO Handle error
        });
    }

}
