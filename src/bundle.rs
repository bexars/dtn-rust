//pub mod bundle {
pub mod pri_block;
pub mod payload_block;



use serde_cbor;
use serde_cbor::value::{Value};
use super::eid::Eid;



pub struct Bundle {
    pri_block: pri_block::PriBlock,
    sec_blocks: Vec<SecBlock>,
    payload_block: payload_block::PayloadBlock,
}


impl Bundle {
    pub fn new() -> Bundle {
        Bundle {
            pri_block: pri_block::PriBlock::new(),
            sec_blocks: Vec::<SecBlock>::new(),
            payload_block: payload_block::PayloadBlock::new(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut out: Vec<u8> = vec![159];
        out.append(&mut self.pri_block.as_bytes());
        out.append(&mut self.payload_block.as_bytes());
        out.push(255);
        out
    }

    pub fn set_payload(&mut self, payload: Vec::<u8>) {
        &self.payload_block.set_payload(payload);
    }

    pub fn set_destination(&mut self, destination: Eid) {
        &self.pri_block.set_d_eid(destination);
    }
}




struct SecBlock {
    _id: u8,
}

trait Block: ConvertCbor {
    fn get_id() -> u8;
    fn get_type() -> u8;

    // fn as_cbor(&self) -> Vec<u8>;

}

pub trait ConvertCbor {
    fn as_cbor(&self) -> Value;
    fn as_bytes(&self) -> Vec<u8>;
    // fn from_bytes(buf: Vec<u8>);
}
