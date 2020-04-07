use log::*;
use std::collections::HashMap;
use crate::router::RouterModule;
use tokio::sync::mpsc::*;
use tokio::sync::oneshot;
use tokio::sync::{RwLock};
// use tokio::task;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum ModuleMsgEnum {
    MsgProcessing,   
    MsgClaManager,   
    MsgCLI,          
    MsgLogging,      
    MsgStorage,      
    MsgAppAgent,     
    MsgRouting,      
    MsgConfiguration,
    MsgBus(BusMessage),
    ShutdownNow,
    Error(String),  
}

enum RunState {
    Running,
    Stopping,
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum BusMessage {
    SetTx(Sender<ModuleMsgEnum>, RouterModule), // Modules set their TX
    GetTx(RouterModule, Sender<ModuleMsgEnum>),     // Modules can request all modules and respond on the TX
    PayloadTx(RouterModule, Sender<ModuleMsgEnum>),  // TX is in the payload
}

pub struct Bus {
    senders:   Arc<RwLock<HashMap<RouterModule, Sender<ModuleMsgEnum>>>>,
//    rx: Arc<RwLock<Receiver<ModuleMsgEnum>>>,
//    receivers: HashMap<RouterModule, Receiver<ModuleMsgEnum>>,
    running: Arc<RwLock<bool>>,
}

impl Bus {
    pub fn new() -> (Bus, Sender<ModuleMsgEnum>, Receiver<ModuleMsgEnum>) {
        let mut senders = HashMap::new();
        
        let (send, recv) = channel::<ModuleMsgEnum>(50);
        senders.insert(RouterModule::Bus, send.clone());
        let bus = Bus {
            senders: Arc::new(RwLock::new(senders)),
            running: Arc::new(RwLock::new(false)),
        };
        (bus, send, recv)
    }

    pub async fn start(&mut self, rx: Receiver<ModuleMsgEnum>) {
        let running = self.running.clone();
        *running.write().await = true;
        let mut rx = rx;
        let senders = self.senders.clone();
//        let tx = senders.read().await.get(&RouterModule::Bus).clone();

        // let service = { async move {
        while *running.read().await {
            let fut_msg = rx.recv().await;
            if !*self.running.read().await { break; };
            let msg = if let Some(msg) = fut_msg { msg } 
            else { break; };
            let senders = senders.clone();
            let running = running.clone();
            tokio::spawn(async move {
                match msg {
                    ModuleMsgEnum::MsgBus(mb) => {
                        match mb {
                            BusMessage::SetTx(tx, route_mod) => {
                                debug!("Received SetTx from: {:?}", route_mod);
                                senders.write().await.insert(route_mod, tx);
                            },
                            BusMessage::GetTx(route_mod, mut requester) => {
                                if let Some(payload) = senders.read().await.get(&route_mod) {
                                    if let Err(_) = requester.send(ModuleMsgEnum::MsgBus(BusMessage::PayloadTx(route_mod, payload.clone()))).await {
                                        // Log the send error, do debug stuff 
                                    } 
                                };
                            },
                            _ => {
                                eprintln!("Unexpected BusMessage: {:?}", mb);
                            },
                        }

                    }, // end MsgBus match
                    ModuleMsgEnum::ShutdownNow => {
                        *running.write().await = false;
                        debug!("Received Shutdown.  Rebroadcasting");
                        let senders = senders.clone();
                        
                        for (k,v) in senders.write().await.iter_mut() {
                            if *k != RouterModule::Bus {
                            }
                            &v.send(ModuleMsgEnum::ShutdownNow).await;
                            
                        };
                    },
                    _ => {}
                }
            });         

        }
        // tokio::spawn(service);        
    }

}

pub fn send(mut tx: Sender<ModuleMsgEnum>, msg: ModuleMsgEnum) { // -> Result<(), tokio::sync::mpsc::error::SendError<ModuleMsgEnum>> {
    debug!("Send: {:?}", msg);
    
    tokio::spawn(async move {
        tx.send(msg).await;  
    });
}

pub async fn rpc(tx: Sender<ModuleMsgEnum>, msg: ModuleMsgEnum, rx: oneshot::Receiver<ModuleMsgEnum>) -> Result<ModuleMsgEnum, oneshot::error::RecvError> {
    // tokio::spawn(async move {
    //     tx.send(msg).await;
    // });

    // TODO, handle errors better
    let mut tx = tx.clone();
    tx.send(msg).await;
    //  {
    //     Ok(()) => {},
    //     Err(e) => { return Err(e); },
    // }
    // //.unwrap_or(ModuleMsgEnum::Error("Error sending msg".to_string()));
    rx.await
}