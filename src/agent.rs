use log::*;
use bp7::EndpointID;
use bp7::Bundle;
use std::collections::HashMap;
use crate::user::User;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use crate::routing::MetaBundle;
use crate::system::{ SystemModules, BusHandle };
use std::sync::Arc;
use tokio::sync::{ Mutex, RwLock };
use msgbus::{MsgBusHandle, Message};
use crate::bus::ModuleMsgEnum;
use std::time::{Duration, SystemTime};


mod agent_service;
mod agent_state;

#[derive(Debug, Clone, PartialEq)]
pub enum AgentMessage {
    RegisterSsp (String, AgentId, String), /// (ssp, AgentId, login)
    UnRegisterSsp(String),
    SetPassive(String),
    SetActive{ ssp: String, id: AgentId, login: String},
    DeliverBundle(MetaBundle),
}


#[derive(Debug, Clone, PartialEq)]
pub enum AgentClientMessage {
    DeliverBundle(MetaBundle),
}

/// Bundle protocol agent
/// Handles registrations from clients and routes bundles to them as they arrive
/// 
///


pub struct Agent {
    registry: Arc<RwLock<Registry>>,  // k: ssp (/blah of dtn://node/blah )
    bus_handle: BusHandle,
    rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,

}

impl Agent {
    pub async fn new(mut bus_handle: BusHandle) -> Agent {
        let rx = bus_handle.register(SystemModules::AppAgent).await.unwrap();

        Agent {
            registry: Arc::new(RwLock::new(Registry::new())),
            bus_handle,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub async fn start(&self) {
        let rx = self.rx.clone();
        let mut bus_handle = self.bus_handle.clone();
        let (mut agent_service_stop_tx, agent_service_stop_rx) = channel::<()>(32);

        let agent_service = Arc::new(agent_service::AgentService::new(bus_handle.clone(), agent_service_stop_rx).await);
        tokio::task::spawn(async move { agent_service.clone().start().await; });
        let mut rx = rx.lock().await;
        while let Some(msg) = rx.recv().await {
            // Listen for updates from CLAs
            match msg {
                Message::Shutdown => { 
                    agent_service_stop_tx.send(()).await.unwrap();
                    break; 
                },
                Message::Message(ModuleMsgEnum::MsgAppAgent(msg)) => {
                    match msg {
                        AgentMessage::DeliverBundle(metabun) => {
                            let metabun: MetaBundle = metabun;
                            let ssp = if let Some(ssp) = metabun.bundle.primary.destination.scheme_specific_part_dtn() { 
                                debug!("Found ssp: {}", ssp);
                                let parts: Vec<&str> = ssp.split("/").collect();
                                parts[1 as usize].to_owned() 
                                }
                                else { continue; };
                            let reg: Registration = if let Some(reg) = self.registry.read().await.find(&ssp).await { 
                                debug!("Found registration: {:?}", reg);
                                reg } 
                                else { continue; };
                            if let Some(agent_id) = reg.connection {
                                bus_handle.send(SystemModules::AgentClient(agent_id), 
                                    ModuleMsgEnum::MsgAgentClient(AgentClientMessage::DeliverBundle(metabun))).await.unwrap();
                                    //TODO handle the bounce
                            } else {
                                // TODO store it til the agent comes back online
                            }
                        },
                        _ => { error!("Unhandled message: {:?}", msg); },
                    }
                },
                Message::Rpc(ModuleMsgEnum::MsgAppAgent(msg), resp) => {
                    match msg {
                        AgentMessage::RegisterSsp(ssp, id, user) => {
                            debug!("Registering: {}", ssp);
                            let res = self.registry.write().await.register(ssp,id,user).await;
                            match res {
                                Ok(_) => resp.send(ModuleMsgEnum::MsgOk("".to_owned())).unwrap(),
                                Err(e) => resp.send(ModuleMsgEnum::MsgErr(e)).unwrap(),
                            }
                        },
                        _ => { error!("Unhandled rpc: {:?}", msg); },
                    }
                },
                _ => {},
            };
        };
        debug!("Exited Agent loop");
    }
}

pub type AgentId = u32;


struct Registry {
    registry: HashMap<String, Registration>
}

impl Registry {
    fn new() -> Registry {
        Registry {
            registry: HashMap::new(),
        }
    }
    async fn register(&mut self, user: String, id: AgentId, ssp: String) -> Result<(), String> {
        if self.registry.contains_key(&ssp) { return Err("Registration already exists".to_owned()); };
        let new_reg = Registration {
            user,
            ssp: ssp.clone(),
            connection: Some(id),
            ..Default::default()
        };
        debug!("Registering {:?}", new_reg);
        self.registry.insert(ssp, new_reg);
        return Ok(());
    }
    async fn find(&self, ssp: &String) -> Option<Registration> {
        debug!("Finding {:?}", ssp);
        self.registry.get(ssp).cloned()
    }
}

#[derive(Clone, Debug)]
pub struct Registration {
    user: String,
    ssp: String,
    connection: Option<AgentId>,
    create_time: SystemTime,
    access_time: SystemTime,
    exp_time: SystemTime,
}

impl Default for Registration {
    fn default() -> Registration {
        Registration {
            user: "".to_owned(),
            ssp: "".to_owned(),
            connection: None,
            create_time: SystemTime::now(),
            access_time: SystemTime::now(),
            exp_time: SystemTime::UNIX_EPOCH,
        }
    }
}

///     RegisterSsp { ssp: String, id: AgentId, login: String}

pub async fn register_ssp(mut bus_handle:BusHandle, reg_ssp: AgentMessage) -> Result<(),String> {
    if let AgentMessage::RegisterSsp(_,_,_) = reg_ssp {} else { return Err("Invalid AgentMessage".to_owned()); };
    let res = bus_handle.rpc(SystemModules::AppAgent, ModuleMsgEnum::MsgAppAgent(reg_ssp)).await.unwrap();
    match res {
        ModuleMsgEnum::MsgOk(_) => { return Ok(()); }
        ModuleMsgEnum::MsgErr(e) => { return Err(e); }
        _ => { return Err("Uknown error".to_owned()); }
    }

}

