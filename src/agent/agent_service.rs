use log::*;
use crate::bus::ModuleMsgEnum;
use crate::system::{ SystemModules, BusHandle };
use msgbus::{Message};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::io::BufReader;
use tokio::prelude::*;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{ RwLock, Mutex };
use tokio::net::{TcpStream, TcpListener};
use super::agent_state::AgentClient;
use super::AgentId;



/// Manages the TCP connections from Applications/Clients
#[derive(Clone)]
pub(super) struct AgentService {
    connections: Arc<RwLock<HashMap<super::AgentId, Receiver<Message<ModuleMsgEnum>>>>>,
    bus_handle: BusHandle,
    stop_rx: Arc<Mutex<Receiver<()>>>,
}

impl AgentService {
    pub(super) async fn new(bus_handle: BusHandle, stop_rx: Receiver<()>) -> AgentService {
        let stop_rx = Arc::new(Mutex::new(stop_rx));
        AgentService {
            stop_rx,
            bus_handle,
            connections: Arc::new(RwLock::new(HashMap::new())),

        }
    }

    pub(super) async fn start(&self) {
        let mut stop_rx = self.stop_rx.lock().await;
        let mut bus_handle = self.bus_handle.clone();
        let mut listener = TcpListener::bind("0.0.0.0:45560").await.unwrap(); //TODO be graceful about this failure
        let mut incoming = listener.incoming();
        let mut connection_count: AgentId = 0;
        loop {

            tokio::select! {
                Some(_) = stop_rx.recv() => {
                    debug!("Stopping.");
                    break;
                },
                Some(conn) = incoming.next() => {
                    match conn {
                        Err(e) => eprintln!("agent_service accept failed: {:?}", e),
                        Ok(sock) => {
                            connection_count += 1;
                            tokio::spawn(Self::run_client(self.clone(), sock, connection_count));
                        },
                    };
                },

            } // end select
        } // end loop

    }

    async fn run_client(self, sock: TcpStream, id: AgentId) {
        AgentClient::start(sock, self.bus_handle.clone(), id).await;
    }
}
