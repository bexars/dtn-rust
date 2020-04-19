use log::*;
use bp7::EndpointID;
use std::collections::HashMap;
use crate::user::User;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use crate::routing::MetaBundle;
use crate::system::{ SystemModules, BusHandle };
use std::sync::Arc;
use tokio::sync::Mutex;
use msg_bus::{MsgBusHandle, Message};
use crate::bus::ModuleMsgEnum;
use std::time::{Duration, SystemTime};


mod agent_service;
mod agent_state;

/// Bundle protocol agent
/// Handles registrations from clients and routes bundles to them as they arrive
/// 
///


pub struct Agent {
    registry: HashMap<String, User>,
    bus_handle: BusHandle,
    rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,

}

impl Agent {
    pub async fn new(mut bus_handle: BusHandle) -> Agent {
        let rx = bus_handle.register(SystemModules::AppAgent).await.unwrap();

        Agent {
            registry: HashMap::new(),
            bus_handle,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub async fn start(&self) {
        let rx = self.rx.clone();
        let mut bus_handle = self.bus_handle.clone();
        let (agent_service_stop_tx, agent_service_stop_rx) = channel::<()>(32);

        let mut agent_service = Arc::new(agent_service::AgentService::new(bus_handle.clone(), agent_service_stop_rx).await);
        tokio::task::spawn(async move { agent_service.clone().start().await; });
        let mut rx = rx.lock().await;
        while let Some(msg) = rx.recv().await {
            // Listen for updates from CLAs
            match msg {
                Message::Shutdown => { break; },
                Message::Message(ModuleMsgEnum::MsgAppAgent(agent_msg)) => {
                    match agent_msg {
                        

                        _ => { error!("Unhandled message: {:?}", agent_msg) },
                    }
                },
                _ => {},
            };
        };
        debug!("Exited Agent loop");
    }
}

pub type AgentId = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum AgentClientMessage {

}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentMessage {

}

pub struct Registration {
    user: User,
    dir: String,
    connection: Option<Sender<MetaBundle>>,
    create_time: SystemTime,
    access_time: SystemTime,
}

