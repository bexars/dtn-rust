use serde_cbor;
use serde_cbor::value::{Value, to_value};
use serde_cbor::to_vec;
// use crc::crc32;

pub struct PayloadBlock {
    block_num: u8,
    block_type: u8,
    flags: u64,
    crc_type: u8,
    payload: Vec<u8>,
    crc: u32,
}

impl PayloadBlock {
    pub fn new() -> PayloadBlock {
        PayloadBlock {
            block_num: 1,
            block_type: 1,
            flags: 0,
            crc_type: 0,
            payload: Vec::<u8>::new(),
            crc: 0,
        }
    }

    pub fn set_payload(&mut self, payload: Vec::<u8>) {
        self.payload = payload;
    }   

}

impl super::Block for PayloadBlock {

    fn get_id() -> u8 { 1 } // Payload is always 1

    fn get_type() -> u8 { 1 } //Payload is always 1
}

impl super::ConvertCbor for PayloadBlock {
    fn as_cbor(&self) -> Value { 
        let mut buf = Vec::<Value>::new();
        buf.push(to_value(&self.block_type).unwrap());
        buf.push(to_value(&self.block_num).unwrap());
        buf.push(to_value(&self.flags).unwrap());
        buf.push(to_value(&self.crc_type).unwrap());
        buf.push(Value::Bytes(self.payload.to_vec()));
        Value::Array(buf)
    }

    fn as_bytes(&self) -> std::vec::Vec<u8> { 
        to_vec(& self.as_cbor()).unwrap()
     }
}