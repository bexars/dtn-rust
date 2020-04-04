use dtn::router;
use clap::Clap;
use bp7::eid::EndpointID;

#[derive(Debug)]
#[derive(Clap)]
struct Opts {

    // #[clap(short = "e", long = "eid", help = "Local EID (ex: dtn://example.com")]
    // local_eid: String,
    // #[clap(long = "stcp", help = "Enable STCP listener ")]
    // stcp_enable: bool,
    // #[clap(long = "stcp-port", help = "STCP listen port", default_value = "4556" )]
    // stcp_port: u16,
    #[clap(short = "c", long = "conf", help = "Configuration file")]
    conf_file: String,
}

pub fn main() {
    let opts: Opts = Opts::parse();
    println!("{:?}", opts);
//    let local_eid = EndpointID::with_dtn(&opts.local_eid).unwrap();

//    let mut conf = router::Configuration::load().unwrap();

    // let conf = router::Configuration {
    //     local_eid,
    //     stcp_enable: opts.stcp_enable,
    //     stcp_port: opts.stcp_port,

    // };

    

    router::start(opts.conf_file);
}