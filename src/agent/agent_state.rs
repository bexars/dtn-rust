use log::*;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufStream, BufWriter, BufReader };
use async_trait::async_trait;
use tokio::sync::{RwLock};
use std::sync::Arc;
use std::io::{ Error, ErrorKind };
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

#[derive(Debug)]
struct ActualClientState {
    //conn: TcpStream,  // current TCP connection
    closed: bool,  // client closed/dropped connection
    buf: BufStream<TcpStream>,
    
}

pub struct AgentClient {}
impl AgentClient {
    pub async fn start(conn: TcpStream) {
        let mut sm: Box<dyn ClientIter> = ClientSm::<WaitingForConnection>::new(conn).await;
        loop {
            
            sm = match sm.next().await {
                Ok(sm) => sm,
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
struct Closed {}

trait ClientState {}

impl ClientState for WaitingForConnection {}
impl ClientState for Authenticating {}
impl ClientState for AnonymousLogin {}
impl ClientState for UserLogin {}
impl ClientState for Waiting {}
impl ClientState for Closed {}

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
    async fn new(conn: TcpStream) -> Box<dyn ClientIter> {
        // conn.write_all(b"Hello World").await.unwrap();
        let mut buf = BufStream::new(conn);
        // buf.write_all(b"Hello again!").await.unwrap();
        // buf.flush().await;
        Box::new(ClientSm {
            state: Arc::new(RwLock::new(
                ActualClientState {
                    closed: false,
                    // conn,
                    buf,
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
        let buf = &mut self.state.write().await.buf;
        buf.write_all(b"OK Temporary registration information follows\n").await?;
        buf.write_all(b"LOGIN gobbdleygook\n").await?;
        buf.write_all(b"PASSWORD anonymous\n").await?;
        buf.write_all(b"SUCCESS\n").await?;
        buf.flush().await?;
        Ok(Box::new(ClientSm {
            state: self.state.clone(),
            extra: Waiting {
                login: "gobbdleygoodk".to_string(),
            },
        }))    
    }
}

#[async_trait]
impl ClientIter for ClientSm<UserLogin> {
    async fn next(&self) -> Result<Box<dyn ClientIter>, Error> {
        debug!("User");
        let buf = &mut self.state.write().await.buf;
        let rand_string = random_string();
        buf.write_all(&format!("CHALLENGE {}\n",rand_string).as_bytes()).await?;
        buf.flush().await?;
        let mut line = String::new();
        buf.read_line(&mut line).await?;
        let args = line.split_whitespace().collect::<Vec<&str>>();
        if args.len() > 0 && args[0] == "password" {
            buf.write(b"SUCCESS\n").await?;
            buf.flush().await?;
            return Ok(Box::new(ClientSm {
                state: self.state.clone(),
                extra: Waiting {
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
        let buf = &mut self.state.write().await.buf;
        let mut input = String::new();
        let count = buf.read_line(&mut input).await?;
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
        Ok(Box::new(ClientSm {
            state: self.state.clone(),
            extra: Waiting { 
                login: self.extra.login.clone(), 
            },
        }))
    }
}


#[async_trait]
impl ClientIter for ClientSm<Closed> {
    async fn next(&self) -> Result<Box<dyn ClientIter>, Error> {
        debug!("Close");
        let buf = &mut self.state.write().await.buf;
        return Err(Error::new(ErrorKind::Other, "Client initiated termination"));

        // Ok(Box::new(ClientSm {
        //     state: self.state.clone(),
        //     extra: Waiting {},
        // }))
    }
}

fn random_string() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .collect();
    rand_string
}