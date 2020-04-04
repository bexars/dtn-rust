use std::sync::Arc;
use crate::router::processor::Processor;
use super::ClaRW;
use super::ClaType;



pub struct ClaHandle {
        pub rw: ClaRW,
        pub id: HandleID,
        pub in_bytes: usize,
        pub out_bytes: usize,
        pub in_bundles: usize,
        pub out_bundles: usize,
        pub name: String,
       
        pub cla_type: ClaType,
}

pub type HandleID = usize;

impl ClaHandle {
    pub fn new( id: HandleID, name: String, rw: ClaRW, cla_type: ClaType) -> ClaHandle {
        Self {
            id,
            name,
            rw,
            cla_type,
            in_bundles: 0,
            in_bytes: 0,
            out_bundles: 0,
            out_bytes: 0,
        }
    }
}