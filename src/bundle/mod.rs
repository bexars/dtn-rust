use serde_cbor;
use serde_derive::{Serialize};
use serde_cbor::value::{Value, to_value};
use serde_cbor::{to_vec};
use crc::crc32;


#[derive(Serialize)]
pub struct Bundle {
    pri_block: PriBlock,
    sec_blocks: Vec<SecBlock>,
}


impl Bundle {
    pub fn new() -> Bundle {
        Bundle {
            pri_block: PriBlock::new(),
            sec_blocks: Vec::<SecBlock>::new(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut out: Vec<u8> = vec![159];
        out.append(&mut self.pri_block.as_bytes());
        out.push(255);
        out
    }
}

#[derive(Serialize)]
struct PriBlock {
    version: u64, // bpbis-24 4.2.2
    flags: u32,  // bpb 4.1.3 // enumflags2  // TODO
    crc_type: u8,     // bpbis-24 4.2.2   0, 1, 2
    d_eid: String,
    s_eid: String,
    r_eid: String,
    ts_sec: u64,  // creation time since y2k (946684800 UNIX EPOCH) in seconds
    ts_inc: u32,   // incremental for bundles created in the same second
    life: u64, // microseconds that bundle should live before deletion
    frag_offs: u32,  // fragment offset  // only include if frag flag set
    orig_size: u32,  // size of original Application Data Unit (this bundle will be smaller)
    crc: u32, // set to 0 until CRC computed on the primary block, then added back
}

impl PriBlock {
    fn new() -> PriBlock {
        PriBlock {
            version: 0,
            flags: 0,
            crc_type: 2,
            d_eid: "dtn://casper/test".to_owned(),
            s_eid: "".to_owned(),
            r_eid: "".to_owned(),
            ts_sec: 0, // 946684800 is y2k
            ts_inc: 0,
            life: 0,
            frag_offs: 0,
            orig_size: 0,
            crc: 0,
        }
    }
}

impl Block for PriBlock {
    fn as_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::<Value>::new();
        buf.push(to_value(&self.version).unwrap());
        buf.push(to_value(&self.crc_type).unwrap());
        buf.push(to_value(&self.flags).unwrap());
        buf.push(to_value(&self.d_eid).unwrap());
        buf.push(to_value(&self.s_eid).unwrap());
        buf.push(to_value(&self.r_eid).unwrap());
        let mut ts = Vec::<Value>::new();
        ts.push(to_value(&self.ts_sec).unwrap());
        ts.push(to_value(&self.ts_inc).unwrap());
        buf.push(to_value(ts).unwrap());
        buf.push(to_value(&self.life).unwrap());
        buf.push(to_value(&self.crc).unwrap());

        let out = to_value(&buf).unwrap();
        let out = to_vec(&out).unwrap();
        //out[0] = 159;  // change it to an infinite CBOR array
        //out.push(0); // append a fake CRC of 0
        let crc = crc32::checksum_castagnoli(&out);
        buf.pop(); // remove the fake CRC
        buf.push(to_value(crc).unwrap());
        to_vec(& to_value(&buf).unwrap()).unwrap()
        // out
    }
    fn get_id() -> u8 { 0 }
}



#[derive(Serialize)]
struct SecBlock {
    id: u8,
}

trait Block {
    fn get_id() -> u8;

    fn as_bytes(&self) -> Vec<u8>;

}
