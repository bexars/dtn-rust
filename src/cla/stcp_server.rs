use tokio::net::TcpListener;
use tokio::io::BufReader;
use tokio::prelude::*;
use futures::stream::StreamExt;
use bp7::Bundle;
use bp7::ByteBuffer;
use std::convert::TryFrom;
use crate::cla::{ClaRW, ClaTrait, ClaBundleStatus};
use tokio::sync::mpsc::{Sender};
use crate::routing::MetaBundle;


pub struct StcpServer {
    address: String,
    port: u16,
    stop_sender: Option<Sender<()>>,
}



impl StcpServer {

    // pub const CLA_TYPE: ClaType = ClaType::StcpListener;
    pub const CLA_RW: ClaRW = ClaRW::R;

    pub fn new(address: String, port: u16 ) -> StcpServer {
        StcpServer {
            address,
            port,
            stop_sender: None,
        }
        
    }

}


impl ClaTrait for StcpServer {

    fn start(&mut self, tx: Sender<ClaBundleStatus>) {
        

        // start listening!
        let addr = format!("{}:{}", self.address, self.port);
//        println!("Starting STCP listener: {}", addr);
        let addr2 = addr.clone();
        let tx = tx.clone();
        let (stop_sender, stop_listener) = tokio::sync::mpsc::channel(1);
        self.stop_sender = Some(stop_sender);
        let server = {
            async move {
                let mut listener = TcpListener::bind(addr).await.unwrap();
                let mut incoming = listener.incoming();
                let mut stop_listener = stop_listener;
                loop {
                    tokio::select! { 
                        _  = stop_listener.recv() => { break; } // stop due to to stop being received
                        Some(conn) = incoming.next() => {
                            // let cla_handle = self.cla_handle.clone();
                            let mut tx = tx.clone();
                            match conn {
                                Err(e) => eprintln!("stcp accept failed: {:?}", e),
                                Ok(mut sock) => {
                                    tokio::spawn(async move {
                                        let remote_addr = sock.peer_addr().expect("Unable to get peer address");
                                        println!("Incoming stcp from: {}", remote_addr);
                                        
                                        let (reader, _) = sock.split();
                                        let mut reader = BufReader::new(reader);
                                        let array_start = reader.read_u8().await;
                                        if let Ok(c) = array_start {
                                            println!("First byte received: {}", c);                                    
                                        };

                                        let cbor_maj = reader.read_u8().await.unwrap();
                                        println!("2nd byte: {}", cbor_maj);
                                        let mut size: usize = 0;
                                        if cbor_maj & 24 == 24  {
                                            match cbor_maj & 31 {
                                                24 => size = reader.read_u8().await.unwrap().into(),
                                                25 => size = reader.read_u16().await.unwrap().into(),
                                                26 => size = reader.read_u32().await.unwrap() as usize,
                                                0..=23 => size = (cbor_maj & 31).into(),
                                                _ => size = 0,
                                            } 
                                        }
                                        let mut buf: Vec<u8> = vec![0; size];
                                        let mut total = 0;
                                        while total < size {
                                            let bytes_read = reader.read(& mut buf[total..size]).await.unwrap();
                                            total += bytes_read;
                                        };
                                        
                                
                                        assert_eq!(total, size); 
                                        let buf = ByteBuffer::from(buf);
                                        let bundle = Bundle::try_from(buf).unwrap();
                                        println!("STCP received bundle, sending to processing");
                                        tx.send(ClaBundleStatus::New(bundle)).await;

                                    });
                                    

                                }
                            }
                        }
                    };
                }
            }
        };

        tokio::spawn(server);
        println!("stcp service running on {}", addr2);

    }

    fn stop(&mut self) { if matches!(self.stop_sender, Some(_)) { 
        futures::executor::block_on(self.stop_sender.as_ref().unwrap().clone().send(())); } }

    fn send(&mut self, bundle: MetaBundle) { unimplemented!(); }
}