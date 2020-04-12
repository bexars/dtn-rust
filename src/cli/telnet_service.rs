use log::*;
use super::*;
use futures_util::future::*;
use std::net::{TcpListener, TcpStream, SocketAddr, SocketAddrV6};
use std::sync::Arc;
use socket2::{Socket, Domain, Type};
use stdio_override::{StdoutOverride, StdinOverride, StderrOverride};
use msg_bus::MsgBusHandle;
use nix::pty::ptsname_r;
use nix::pty::{posix_openpt, grantpt, unlockpt, PtyMaster};
use nix::fcntl::{ open};
use nix::fcntl::OFlag;
use nix;
use nix::sys::{stat, termios};
pub use nix::sys::{wait, signal};
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::io::copy;


type BusHandle = MsgBusHandle<RouterModule, ModuleMsgEnum>;

#[derive(Clone)]
pub(super) struct TelnetService {
    listen_socket: Arc<socket2::Socket>,
    bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>,
}

impl TelnetService {
    pub(super) async fn start(cli_conf: &CliConfiguration, bus_handle: BusHandle) -> Self {
        // let (sender, receiver) = tokio::sync::oneshot::channel();
        let bind_addr = format!("{}:{}", cli_conf.telnet_address, cli_conf.telnet_port);
        let sock_addr = bind_addr.parse::<SocketAddr>().unwrap();
        debug!("sock_addr: {}", sock_addr);
        let listen_socket = Socket::new(Domain::ipv4(), Type::stream(), None).unwrap();
        listen_socket.bind(&sock_addr.into()).unwrap();
        let telnet_service = Self{ listen_socket: Arc::new(listen_socket), bus_handle };
        let ts = telnet_service.clone();
        tokio::task::spawn_blocking(move || { ts.run(); });

        telnet_service
    }

    fn run(&self) {
        debug!("About to enter TCP telnet listen loop");
        self.listen_socket.listen(4).unwrap();
        while let Ok((mut socket, sockaddr)) = self.listen_socket.accept() {
            debug!("Got an opened socket");

//            let (master_fd, slave_name) = Self::get_tty();
            let master_fd = posix_openpt(OFlag::O_RDWR).unwrap();

            // Allow a slave to be generated for it
            grantpt(&master_fd).unwrap();
            unlockpt(&master_fd).unwrap();
    
            // on Linux this is the libc function, on OSX this is our implementation of ptsname_r
            let slave_name = ptsname_r(&master_fd).unwrap();
            let slave_fd = open(std::path::Path::new(&slave_name),
                                        OFlag::O_RDWR,
                                        stat::Mode::empty()).unwrap();
    
            println!("{}", slave_name);
            let mut file;
            unsafe {
                file = std::fs::File::from_raw_fd(master_fd.as_raw_fd());
            }
            let mut file2 = file.try_clone().unwrap();
            let mut socket2 = socket.try_clone().unwrap();
            let metadata = file.metadata().unwrap();
            println!("{:?}", metadata);
            let metadata = file2.metadata().unwrap();
            println!("{:?}", metadata);
            tokio::task::spawn_blocking(move || { copy(&mut file2, &mut socket).unwrap(); println!("copy closed"); });
            let mut file2 = file.try_clone().unwrap();
            tokio::task::spawn_blocking(move || { copy(&mut socket2, &mut file2).unwrap(); println!("Copy2 closed"); } );

            let bus_handle = self.bus_handle.clone();
            tokio::task::spawn_blocking(move || {



                super::terminal::start(Some(slave_name), bus_handle.clone());            
            });

        };
        debug!("Left the TCP telnet listen loop");
    }

    fn get_tty() -> (nix::pty::PtyMaster, String) {

        let win_size = nix::pty::Winsize{
            ws_row:25,
            ws_col:80,
            ws_xpixel:0,
            ws_ypixel:0,
        };

        let master_fd = posix_openpt(OFlag::O_RDWR).unwrap();

        // Allow a slave to be generated for it
        grantpt(&master_fd).unwrap();
        unlockpt(&master_fd).unwrap();

        // on Linux this is the libc function, on OSX this is our implementation of ptsname_r
        let slave_name = ptsname_r(&master_fd).unwrap();
        let slave_fd = open(std::path::Path::new(&slave_name),
                                    OFlag::O_RDWR,
                                    stat::Mode::empty()).unwrap();
        (master_fd, slave_name)
    }

    pub(super) async fn stop() {

    }
}
