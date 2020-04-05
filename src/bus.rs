use log;
use std::collections::HashMap;
use crate::router::RouterModule;
use tokio::sync::mpsc::*;
use tokio::sync::{RwLock};
// use tokio::task;
use std::sync::Arc;

#[derive(Debug)]
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
}

#[derive(Debug)]
pub enum BusMessage {
    SetTx(Sender<ModuleMsgEnum>, RouterModule), // Modules set their TX
    GetTx(RouterModule, Sender<ModuleMsgEnum>),     // Modules can request all modules and respond on the TX
    PayloadTx(RouterModule, Sender<ModuleMsgEnum>),  // TX is in the payload
}

pub struct Bus {
    senders:   Arc<RwLock<HashMap<RouterModule, Sender<ModuleMsgEnum>>>>,
//    rx: Arc<RwLock<Receiver<ModuleMsgEnum>>>,
//    receivers: HashMap<RouterModule, Receiver<ModuleMsgEnum>>,
}

impl Bus {
    pub fn new() -> (Bus, Sender<ModuleMsgEnum>, Receiver<ModuleMsgEnum>) {
        let mut senders = HashMap::new();
        
        let (send, recv) = channel::<ModuleMsgEnum>(50);
        senders.insert(RouterModule::Bus, send.clone());
        let bus = Bus {
            senders: Arc::new(RwLock::new(senders)),
        };
        (bus, send, recv)
    }

    pub fn start(&mut self, rx: Receiver<ModuleMsgEnum>) {
        let mut rx = rx;
        let senders = self.senders.clone();
//        let tx = senders.read().await.get(&RouterModule::Bus).clone();

        let service = { async move {
            while let Some(msg) = rx.recv().await {
                let senders = senders.clone();
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

                        },
                        _ => {}
                    }
                });         

            }
        }};          
            
        tokio::spawn(service);
        
    }

}

