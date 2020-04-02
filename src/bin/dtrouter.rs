use dtn::router;
use dtn::eid;
use clap::Clap;

#[derive(Debug)]
#[derive(Clap)]
struct Opts {

    #[clap(short = "e", long = "eid", help = "Local EID (ex: dtn://example.com")]
    local_eid: String,
    #[clap(long = "stcp", help = "Enable STCP listener ")]
    stcp_enable: bool,
    #[clap(long = "stcp-port", help = "STCP listen port", default_value = "4556" )]
    stcp_port: u16,
}

pub fn main() {
    let opts: Opts = Opts::parse();
    println!("{:?}", opts);
    let local_eid = eid::Eid::new_uri(&opts.local_eid);


    let conf = router::Configuration {
        local_eid,
        stcp_enable: opts.stcp_enable,
        stcp_port: opts.stcp_port,

    };

    router::start(conf);
}