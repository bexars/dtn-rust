use log::*;
use crate::system::{ SystemModules, BusHandle };
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use msg_bus::{ Message};
use crate::bus::ModuleMsgEnum;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::{Receiver};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;


#[derive(Clone, Debug, PartialEq)]
pub enum UserMgrMessage {
    AddAnonymousLogin,
    VerifyLogin { login: String, password: String, challenge: String },
    DataUserAdded{ login: String },

}

#[derive(Clone, Debug, PartialEq)]
pub struct User { 
    id: u32,
    login: String,
    anonymous: bool,
    password: Option<String>,
    pubkey: Option<Vec<u8>>,
    create_time: SystemTime,
    access_time: SystemTime,
    exp_time: SystemTime,
}

impl Default for User {
    fn default() -> User {
        User {
            id: 0,
            login: "".to_string(),
            anonymous: true,
            password: None,
            pubkey: None,
            create_time: SystemTime::now(),
            access_time: SystemTime::now(),
            exp_time: SystemTime::now() + Duration::new(86400, 0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Users {
    users: HashMap<String, User>,
    cur_id: u32,
}

impl Users {
    fn add(&mut self, mut user: User) {
        self.cur_id += 1;
        user.id = self.cur_id;
        let user = user;
        self.users.insert(user.login.clone(), user);
    }
    fn find(&self, login: &String) -> Option<&User> {
        self.users.get(login)
    }
}




pub struct UserMgr {
    users: Arc<RwLock<Users>>,
    bus_handle: BusHandle,
    rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,

}

impl UserMgr {
    pub async fn new(mut bus_handle: BusHandle) -> UserMgr {
        let rx = bus_handle.register(SystemModules::UserMgr).await.unwrap();

        UserMgr {
            users: Arc::new(RwLock::new(Users::default())),
            bus_handle,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub async fn start(&self) {
        let rx = self.rx.clone();
        let _bus_handle = self.bus_handle.clone();
        let mut rx = rx.lock().await;
        while let Some(msg) = rx.recv().await {
            match msg {
                Message::Shutdown => { break; },
                Message::Rpc(ModuleMsgEnum::MsgUserMgr(msg), resp) => {
                    match msg {
                        UserMgrMessage::AddAnonymousLogin => { 
                            let u = User {
                                login: random_string(16),
                                password: Some("anonymous".to_owned()),
                                ..User::default()
                            };
                            self.users.write().await.add(u.clone());
                            debug!("Added anonymous");
                            resp.send(ModuleMsgEnum::MsgUserMgr(UserMgrMessage::DataUserAdded{login: u.login})).unwrap();
                        },
                        UserMgrMessage::VerifyLogin{ login, password, challenge } => { 
                            let users = self.users.read().await;
                            let user = users.find(&login);
                            let user = if let Some(user) = user { user } else { 
                                resp.send(ModuleMsgEnum::MsgErr("No such user".to_owned())).unwrap();
                                continue;
                            };
                            if let Some(pass) = &user.password { if pass == &password {
                                    resp.send(ModuleMsgEnum::MsgOk(" ".to_owned())).unwrap();
                                    continue;
                                };
                            };
                            resp.send(ModuleMsgEnum::MsgErr("Invalid Password".to_owned())).unwrap();
                            continue;
                        }

                        _ => { error!("Unhandled rpc: {:?}", msg) },
                    }
                },
                _ => { warn!("Unhandled msg: {:?}", msg)},
            };
        };
        debug!("Exited UserMgr loop");
    }
}

fn random_string(length: usize) -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .collect();
    rand_string
}

pub async fn add_anonymous_login(bus_handle: &mut BusHandle) -> Result<String, String> {
    let res = bus_handle.rpc(SystemModules::UserMgr, ModuleMsgEnum::MsgUserMgr(UserMgrMessage::AddAnonymousLogin)).await.unwrap();
    if let ModuleMsgEnum::MsgUserMgr(UserMgrMessage::DataUserAdded{login}) = res {
        return Ok(login)
    }
    return Err("Anonymous logins disabled".to_owned());
}

pub async fn verify_login(bus_handle: &mut BusHandle, login: String, password: String, challenge: String) -> Result<String, String> {
    let res = bus_handle.rpc(SystemModules::UserMgr, 
        ModuleMsgEnum::MsgUserMgr(UserMgrMessage::VerifyLogin{login: login, password: password, challenge: challenge})).await.unwrap();

    match res {
        ModuleMsgEnum::MsgOk(m) => return Ok(m),
        ModuleMsgEnum::MsgErr(e) => return Err(e),
        _ => return Err("Unknown failure".to_owned()),
    };
}