use std::convert::TryFrom;
use std::io::BufRead;
use std::io::Write;
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
use std::io::{ BufReader, BufWriter };
use bp7::ByteBuffer;

#[derive(Debug)]
#[derive(Clap)]
struct Opts {
    #[clap(short = "d", long = "dest", help = "Set destination EID")]
    dest: Option<String>,
    #[clap( long = "source", help = "Set source EID")]
    source: Option<String>,
    #[clap(short = "l", long = "listen", help = "SSP resource to listen on ( omit dtn://nodename/ from EID)")]
    listen: Option<String>,
    #[clap(long = "host", help = "IP address or hostname of dtn node", default_value = "localhost" )]
    host: String,
    #[clap(short = "p", long = "port", help = "port on dtn node to connect to", default_value = "4556" )]
    port: u16,    
    #[clap(short = "u", long = "user", help = "user to connect as")]
    user: Option<String>,
    #[clap(short = "s", long = "secret", help = "password secret")]
    secret: Option<String>,
}

fn main() {
    let opts: Opts = Opts::parse();

    if let Some(_) = &opts.user {
        if let None = &opts.secret {
            eprintln!("Password secret required when user is specified");
            return;
        }
    } 

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
    let mut buffer: Vec::<u8> = Vec::new();
    io::stdin().read_to_end(&mut buffer).expect("Error reading from stdin");

    let primary = PrimaryBlock::default();
    let mut canonicals: Vec<CanonicalBlock> = vec![]; 
    canonicals.push(bp7::new_payload_block(0 as BlockControlFlags, buffer));

    let mut bundle: Bundle = Bundle::new(primary, canonicals);

    bundle.primary.destination = EndpointID::with_dtn(& opts.dest.as_ref().unwrap()).unwrap();
    if matches!(opts.source, Some(_)) {
        bundle.primary.source = EndpointID::with_dtn(& opts.source.as_ref().unwrap()).unwrap();
    }

    let host_port = format!("{}:{}", &opts.host, &opts.port);
    let mut stream: TcpStream = match TcpStream::connect(host_port) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    let res = stream.write(&stcp::encapsulate_stcp(bundle));
    if let Err(e) = res { println!("Error transmitting to DTN host{}",e) };
}

fn recv_bundle(opts: &Opts) {
    let host_port = format!("{}:{}", &opts.host, &opts.port);
    let user = &opts.user.clone().unwrap_or(String::from("anonymous"));
    let pass = &opts.secret;

    let mut stream = TcpStream::connect(host_port).unwrap();
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let lines = &mut reader.by_ref().lines();
    if let None = pass {
        write!(writer, "login anonymous\n");
        writer.flush();
            while let Some(line) = lines.next() {
            let line = line.unwrap();
            if line.contains("LOGIN") {
                let mut s = line.split("LOGIN ");
                let anon_user = s.nth(1).unwrap();
                println!("To re-use bindings set options:  --user {} --secret anonymous", anon_user);
                break;
            }
            
        }
    } else {
        write!(writer, "login {}\npassword {}\n", user, pass.clone().unwrap());
        writer.flush();
    }
    
        //let lines = &mut reader.by_ref().lines();
        while !lines.next().unwrap().unwrap().contains("SUCCESS") {};
        println!("Password success");
    
    writeln!(writer,"register {}", &opts.listen.as_ref().unwrap());
    writer.flush();
    let result = lines.next().unwrap().unwrap();
    if result.contains("ERR") { println!("{}", result);
        return;
    } 
    let mut buf = String::new();
    reader.read_line(&mut buf);
    println!("Attempting to read bundle {}", buf);
    let bun_size = buf.split_whitespace().nth(1).unwrap().parse().unwrap();
    println!("Incoming bundle of {} bytes", bun_size);
    let mut buffer = vec![0; bun_size];
    reader.read_exact(&mut buffer);
    let bundle = Bundle::try_from(buffer).unwrap();
    println!("{}", std::str::from_utf8(bundle.payload().unwrap()).unwrap());
}

