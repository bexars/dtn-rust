use serde::{Serialize, Deserialize};



pub mod cla_handle;
pub mod cla_manager;
pub mod stcp_server;

pub enum ClaRW {
    R,
    RW,
    W,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClaType {
    StcpListener( String, u16),  // local address, port
    Stcp(String, u16),  // remote address, port
    StcpIp(String, u16, String), // remote address, port, dns domain to search (. for ip.earth) 
    LoopBack,  // ...
}
impl Default for ClaType {
    fn default() -> Self { ClaType::LoopBack }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ClaConfiguration {
    name: String,
    peernode: String,
    enabled: bool,
    cla_type: ClaType,
}
