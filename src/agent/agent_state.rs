use msg_bus::Message;
use log::*;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufStream};
use tokio::sync::mpsc::Receiver;
use async_trait::async_trait;
use tokio::sync::{RwLock};
use std::sync::Arc;
use std::io::{ Error, ErrorKind };
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use crate::system::{ BusHandle, SystemModules };
use msg_bus::{ MsgBusHandle};
use crate::bus::ModuleMsgEnum;
use super::{ AgentId, AgentClientMessage };
// use crate::routing::MetaBundle;


struct ActualClientState {
    //conn: TcpStream,  // current TCP connection
    // closed: bool,  // client closed/dropped connection
    buf: BufStream<TcpStream>,
    bus_handle: MsgBusHandle<SystemModules, ModuleMsgEnum>,
    id: AgentId,
    rx: Option<Receiver<Message<ModuleMsgEnum>>>,
}

pub struct AgentClient {}
impl AgentClient {
    pub async fn start(conn: TcpStream, bus_handle: BusHandle, id: AgentId) {
        let mut sm: Box<dyn ClientIter> = ClientSm::<WaitingForConnection>::new(conn, bus_handle, id).await;
        loop {
            
            sm = match sm.next().await {
                Ok(sm) => {
                    debug!("sm.next() ok");    
                    sm
                },
                Err(err) => {
                    debug!("{:?}", err);
                    break;    
                }
            }; 
        }
    }
}


struct ClientSm<S: ClientState> {
    /// Store long lived state
    state: Arc<RwLock<ActualClientState>>,
    /// Store extra state if needed
    extra: S,
}


#[async_trait]
trait ClientIter: Sync + Send {
    async fn next(&self) -> Result<Box<dyn ClientIter>, Error>;

}

struct WaitingForConnection {}
struct Authenticating {}
struct AnonymousLogin {}
struct UserLogin {
    login: String,
}
struct Waiting {
    login: String,
}
struct ProcessBusMsg {
    msg: Message<ModuleMsgEnum>,
    login: String,
}
struct Register {
    ssp: String,
    login: String,
}

struct Closed {}

trait ClientState {}

impl ClientState for WaitingForConnection {}
impl ClientState for Authenticating {}
impl ClientState for AnonymousLogin {}
impl ClientState for UserLogin {}
impl ClientState for Waiting {}
impl ClientState for Closed {}
impl ClientState for ProcessBusMsg {}
impl ClientState for Register {}

#[async_trait]
impl ClientIter for ClientSm<WaitingForConnection> {
    async fn next(&self) -> Result<Box<dyn ClientIter>, Error> {
        debug!("waiting for connection");
        Ok(Box::new(ClientSm {
            state: self.state.clone(),
            extra: Authenticating{},
        }))
    }
}

impl ClientSm<WaitingForConnection> {
    async fn new(conn: TcpStream, mut bus_handle: BusHandle, id: AgentId) -> Box<dyn ClientIter> {
        // conn.write_all(b"Hello World").await.unwrap();
        let buf = BufStream::new(conn);
        // buf.write_all(b"Hello again!").await.unwrap();
        // buf.flush().await;

        let rx = bus_handle.register(SystemModules::AgentClient(id)).await.unwrap();
        Box::new(ClientSm {
            state: Arc::new(RwLock::new(
                ActualClientState {
                    // closed: false,
                    buf,
                    bus_handle,
                    id,
                    rx: Some(rx),
                }
            )),
            extra: WaitingForConnection {} 
        })      
    }
}

#[async_trait]
impl ClientIter for ClientSm<Authenticating> {
    async fn next(&self) -> Result<Box<dyn ClientIter>, Error> {
        debug!("Authenticating");
        let buf = &mut self.state.write().await.buf;

        buf.write_all(b"OK Welcome to the DTN Bundling Agent\n").await.unwrap();
        buf.write_all(b"*  Anonymous users welcome but restricted\n").await.unwrap();
        buf.flush().await?;
        let mut input = String::new();
        buf.read_line(&mut input).await?;
        let args: Vec<&str> = input.split_whitespace().collect();
        
        if args.len() != 2 || args[0] != "login"  {
            buf.write_all(&format!("Invalid 'login <username>' command {}\n",args[0]).as_bytes()).await?;
            buf.flush().await?;
            return Err(Error::new(ErrorKind::InvalidInput, format!("Invalid command sent: {}", args[0])))
        };
        if args[1] == "anonymous" {
            return Ok(Box::new(ClientSm {
                state: self.state.clone(),
                extra: AnonymousLogin {},
            }))    
        }

        let state = self.state.clone();

        Ok(Box::new(ClientSm {
            state,
            extra: UserLogin {
                login: args[1].to_string(),
            },
        }))
    }
}

#[async_trait]
impl ClientIter for ClientSm<AnonymousLogin> {
    async fn next(&self) -> Result<Box<dyn ClientIter>, Error> {
        debug!("Anonymous");
        let bus_handle = &mut self.state.read().await.bus_handle.clone();
        let login = crate::user::add_anonymous_login(bus_handle).await.unwrap();
        let buf = &mut self.state.write().await.buf;
        buf.write_all(b"OK Temporary registration information follows\n").await?;
        buf.write_all(&format!("* LOGIN {}\n",login).as_bytes()).await?;
        buf.write_all(b"* PASSWORD anonymous\n").await?;
        buf.write_all(b"SUCCESS\n").await?;
        buf.flush().await?;
        Ok(Box::new(ClientSm {
            state: self.state.clone(),
            extra: Waiting {
                login,
            },
        }))    
    }
}

#[async_trait]
impl ClientIter for ClientSm<UserLogin> {
    async fn next(&self) -> Result<Box<dyn ClientIter>, Error> {
        debug!("UserLogin");
        let state = &mut self.state.write().await;
        let mut bus_handle = &mut state.bus_handle.clone();
        let buf = &mut state.buf;
        let rand_string = random_string();
        buf.write_all(&format!("CHALLENGE {}\n",rand_string).as_bytes()).await?;
        buf.flush().await?;
        let mut line = String::new();
        buf.read_line(&mut line).await?;
        let args = line.split_whitespace().collect::<Vec<&str>>();
        
        
        if args.len() == 2 && args[0] == "password" {
            debug!("Verifying password");
            // let mut bus_handle = state.bus_handle.clone();
            let res = crate::user::verify_login(
                 &mut bus_handle, self.extra.login.clone(), args[1].to_owned(),  "".to_owned()).await;

            debug!("Verify result {:?}", res);
            match res {
                Ok(_) => {
                    buf.write(b"SUCCESS\n").await?;
                    buf.flush().await?;
                    return Ok(Box::new(ClientSm {
                        state: self.state.clone(),
                        extra: Waiting {
                            login: self.extra.login.clone(),
                        },
                    }));        
                },
                Err(_) => {
                    buf.write(b"Incorrect.  Try again.").await?;
                    buf.flush().await?;
                },
            };
            return Ok(Box::new(ClientSm {
                state: self.state.clone(),
                extra: UserLogin {
                    login: self.extra.login.clone(),
                },
            }))
        }
        Ok(Box::new(ClientSm {
            state: self.state.clone(),
            extra: Closed {},
        }))    
    }
}

#[async_trait]
impl ClientIter for ClientSm<Waiting> {
    async fn next(&self) -> Result<Box<dyn ClientIter>, Error> {
        debug!("Waiting");
        let state_clone = self.state.clone();
        let mut state = self.state.write().await;
        let rx = state.rx.take();
        let mut rx = if let Some(rx) = rx { rx } else {        
             return Err(Error::new(ErrorKind::Other, "Client initiated termination"));
         };
        let mut input = String::new();
        // let state:usize = 0;
        // let state = self.state.write().await;
        
        let buf = &mut state.buf;

        let count = tokio::select! {
            Some(msg) = rx.recv() => {    
                state.rx = Some(rx);
                return Ok(Box::new(ClientSm {
                    state: state_clone,
                    extra: ProcessBusMsg {
                        msg,
                        login: self.extra.login.clone(),
                    },
                })); 
            }
            Ok(count) = buf.read_line(&mut input) => {
                debug!("read_line");
                count
            }
        };
        drop(state);
        #[allow(unused_variables)]
        let state:usize = 0;
        let mut state = self.state.write().await;
        debug!("Got state lock");
        state.rx = Some(rx);

        #[allow(unused_variables)]
        let rx:usize = 0;
        let buf = &mut state.buf;
        // let count = buf.read_line(&mut input).await?;
        if count == 0 { debug!("Input count = 0");
                        buf.write_all(b"HELLO").await?;
                        buf.flush().await?;
                    };
        let args = input.split_whitespace().collect::<Vec<&str>>();
        if args.len() > 0 && args[0] == "quit" {
            return Ok(Box::new(ClientSm {
                state: self.state.clone(),
                extra: Closed {},
            }));    
        }
        if args.len() == 2 && args[0] == "register" {
            return Ok(Box::new(ClientSm {
                state: self.state.clone(),
                extra: Register {
                    login: self.extra.login.clone(),
                    ssp: args[1].to_owned(),
                },
            }));    
        }
        Ok(Box::new(ClientSm {
            state: self.state.clone(),
            extra: Waiting { 
                login: self.extra.login.clone(), 
            },
        }))
    }
}

#[async_trait]
impl ClientIter for ClientSm<ProcessBusMsg> {
    async fn next(&self) -> Result<Box<dyn ClientIter>, Error> {
        debug!("ProcessBusMsg");
        let buf = &mut self.state.write().await.buf;  
        //let msg = self.extra.msg.clone();

        if let Message::Message(ModuleMsgEnum::MsgAgentClient(AgentClientMessage::DeliverBundle(metabun))) = &self.extra.msg {
            // let mut metabun: MetaBundle = metabun;
            let bun_buffer = metabun.clone().bundle.to_cbor();
            let size = bun_buffer.len();
            buf.write_all(&format!("BUNDLE {}\n", size).as_bytes()).await?;
            buf.write_all(&bun_buffer).await?;
            buf.write_all(b"\n\nOK\n").await?;
            buf.flush().await?;
        } 
        Ok(Box::new(ClientSm {
            state: self.state.clone(),
            extra: Waiting {
                login: self.extra.login.clone(),
            },
        }))            

    }
}

#[async_trait]
impl ClientIter for ClientSm<Register> {
    async fn next(&self) -> Result<Box<dyn ClientIter>, Error> {
        debug!("Register");
        let state = self.state.write().await;
        let bus_handle = state.bus_handle.clone();
        
        let res = crate::agent::register_ssp(bus_handle, crate::agent::AgentMessage::RegisterSsp(self.extra.login.clone(), state.id, self.extra.ssp.clone())).await;
        drop(state);
        let buf = &mut self.state.write().await.buf;
        match res {
            Ok(_) => { buf.write_all(b"OK Registered.\n").await?; },
            Err(e) => { buf.write_all(&format!("ERR {}\n", e).as_bytes()).await?; },
        }        
        buf.flush().await?;
        return Ok(Box::new(ClientSm {
            state: self.state.clone(),
            extra: Waiting {
                login: self.extra.login.clone(),
            },
        }));

    }
}


#[async_trait]
impl ClientIter for ClientSm<Closed> {
    async fn next(&self) -> Result<Box<dyn ClientIter>, Error> {
        debug!("Close");
        let buf = &mut self.state.write().await.buf;   // TODO unregister myself
        buf.write_all(b"Goodbye.").await?;
        buf.flush().await?; 
        return Err(Error::new(ErrorKind::Other, "Client initiated termination"));

    }
}

fn random_string() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .collect();
    rand_string
}