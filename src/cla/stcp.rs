use tokio::net::{TcpStream};
use tokio::prelude::*;
use crate::cla::{ClaRW, ClaTrait, ClaBundleStatus};
use tokio::sync::mpsc::Sender;
use crate::routing::MetaBundle;
use crate::stcp;


pub struct Stcp {
    address: String,
    port: u16,
    status_handler: Option<Sender<ClaBundleStatus>>
}



impl Stcp {

    // pub const CLA_TYPE: ClaType = ClaType::StcpListener;
    pub const CLA_RW: ClaRW = ClaRW::W;

    pub fn new(address: String, port: u16 ) -> Stcp {
        Stcp {
            address,
            port,
            status_handler: None,
        }
        
    }

}


impl ClaTrait for Stcp {

    fn start(&mut self, tx: Sender<ClaBundleStatus>) {
        self.status_handler = Some(tx);
    }

    fn stop(&mut self) { unimplemented!(); }
    fn send(&mut self, bundle: MetaBundle) { 
        // TODO be smarter and not re-open every bundle
         let addr = format!("{}:{}", self.address, self.port);
        
        tokio::task::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap(); //TODO Handle bad connection

            stream.write(&stcp::encapsulate_stcp(bundle.bundle)).await.unwrap(); //TODO Handle error
        });
    }

}
