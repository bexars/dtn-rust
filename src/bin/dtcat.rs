use std::io::Write;
use std::net::IpAddr;
use std::str::FromStr;
use clap::Clap;
use std::process;
use std::io::{self, Read};
use bp7::Bundle;
use bp7::bundle::BlockControlFlags;
use bp7::{CanonicalBlock};
use bp7::primary::PrimaryBlock;
use bp7::eid::EndpointID;
use dtn::stcp;
use std::net::TcpStream;
use std::net::SocketAddr;

#[derive(Debug)]
#[derive(Clap)]
struct Opts {
    #[clap(short = "d", long = "dest", help = "Set destination EID")]
    dest: Option<String>,
    #[clap(short = "l", long = "listen", help = "SSP to listen on ( omit dtn: from EID)")]
    listen: Option<String>,
    #[clap(long = "host", help = "dtn node to connect to", default_value = "localhost" )]
    host: String,
    #[clap(short = "p", long = "port", help = "port on dtn node to connect to", default_value = "4557" )]
    port: u16,
}

fn main() {
    let opts: Opts = Opts::parse();


    println!("{:?}", opts);

    match (&opts.dest, &opts.listen) {
        (Some(_), Some(_)) => {
            println!("Error: Must specify only dest or listen, not both");
            process::exit(2);
        },
        (None, None) => {
            println!("Error: Must specify either dest or listen");
            process::exit(2);
        },
        (Some(_),None) => {
            send_bundle(&opts);
        },
        (None, Some(_)) => {
            recv_bundle(&opts);
        },

    };
}

fn send_bundle(opts: &Opts) {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).expect("Error reading from stdin");

    let primary = PrimaryBlock::default();
    let mut canonicals: Vec<CanonicalBlock> = vec![]; 
    canonicals.push(bp7::new_payload_block(0 as BlockControlFlags, buffer.as_bytes().to_vec()));

    let mut bundle: Bundle = Bundle::new(primary, canonicals);

    bundle.set_payload(buffer.as_bytes().to_vec());
    bundle.primary.destination = EndpointID::with_dtn(& opts.dest.as_ref().unwrap()).unwrap();
    
    let ip_addr = match IpAddr::from_str(&opts.host) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        } 
    };
    let sock_addr = SocketAddr::new(ip_addr, opts.port);
    let mut stream: TcpStream = match TcpStream::connect(sock_addr) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    let res = stream.write(&stcp::encapsulate_stcp(bundle));
    if let Err(e) = res { println!("Error transmitting to DTN host{}",e) };
}

fn recv_bundle(_opts: &Opts) {
    eprintln!("--listen feature is not implemented");
}

