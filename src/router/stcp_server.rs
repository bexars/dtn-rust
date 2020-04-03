use tokio::net::TcpListener;
use tokio::io::BufReader;
use tokio::prelude::*;
use futures::stream::StreamExt;
use bp7::Bundle;
use bp7::ByteBuffer;
use std::convert::TryFrom;

pub struct StcpServer {
    port: u16,
}

impl StcpServer {
    pub fn new(port: u16) -> StcpServer {
        let s = StcpServer {
            port,
        };
        s
    }

    pub async fn start(&self) {
        

        // start listening!
        let addr = format!(":::{}", self.port);
        println!("Starting STCP listener: {}", addr);

        let addr2 = addr.clone();
        let mut listener = TcpListener::bind(addr).await.unwrap();

        let server = {
            async move {
                let mut incoming = listener.incoming();
                while let Some(conn) = incoming.next().await {
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
                                // TODO send to bundle processing
                            });
                            

                        }
                    }
                }
            }
        };

        println!("stcp service running on {}", addr2);
        server.await;

    }
}