use std::sync::Arc;
use crate::processor::Processor;
use super::ClaRW;
use super::ClaType;



pub struct ClaHandle {
        pub rw: ClaRW,
        pub id: HandleId,
        pub in_bytes: usize,
        pub out_bytes: usize,
        pub in_bundles: usize,
        pub out_bundles: usize,
        pub name: String,
       
        pub cla_type: ClaType,
}

pub type HandleId = usize;

impl Default for ClaHandle {
    fn default() -> ClaHandle {
        Self {
            id: 0,
            name: String::from(""),
            rw: ClaRW::R,
            cla_type: ClaType::LoopBack,
            in_bundles: 0,
            in_bytes: 0,
            out_bundles: 0,
            out_bytes: 0,
        }
    }
}

impl ClaHandle {
    pub fn new( id: HandleId, name: String, rw: ClaRW, cla_type: ClaType) -> ClaHandle {
        Self {
            id,
            name,
            rw,
            cla_type,
            ..Default::default()
        }
    }
}