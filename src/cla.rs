use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};    
use std::sync::Arc;
use async_trait::async_trait;
use crate::routing::MetaBundle;


pub mod cla_handle;
pub mod cla_manager;
pub mod stcp_server;
pub mod loopback;

#[async_trait]
pub trait BundleReader {
    async fn start(cla_handle: cla_handle::ClaHandle);
    // async fn accept() -> Arc<MetaBundle>;
}

#[async_trait]
pub trait BundleWriter {
    async fn send(bundle: &Arc<MetaBundle>);
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClaRW {
    R,
    RW,
    W,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    pub adapters: HashMap<String, AdapterConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfiguration {
    pub(crate) name: String,
    pub(crate) peernode: String,
    pub(crate) shutdown: bool,
    pub(crate) cla_type: ClaType,
}

impl PartialEq for AdapterConfiguration {
    fn eq(&self, other: &Self) -> bool {
        return self.name.eq(&other.name);
    }
}

impl Eq for AdapterConfiguration {}

impl Hash for AdapterConfiguration {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Default for AdapterConfiguration {
    fn default() -> AdapterConfiguration {
        AdapterConfiguration {
            name: String::from(""),
            peernode: String::from(""),
            shutdown: true,
            cla_type: ClaType::LoopBack,
        }
    }
}
