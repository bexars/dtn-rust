use crate::eid::Eid;
use serde_cbor;
use serde_cbor::value::{Value, to_value};
use serde_cbor::to_vec;
use crc::crc32;
use super::ConvertCbor;


    pub struct PriBlock {
        version: u64, // bpbis-24 4.2.2
        flags: u32,  // bpb 4.1.3 // enumflags2  // TODO
        crc_type: u8,     // bpbis-24 4.2.2   0, 1, 2
        d_eid: Eid,
        s_eid: Eid,
        r_eid: Eid,
        ts_sec: u64,  // creation time since y2k (946684800 UNIX EPOCH) in seconds
        ts_inc: u32,   // incremental for bundles created in the same second
        life: u64, // microseconds that bundle should live before deletion
        frag_offs: u32,  // fragment offset  // only include if frag flag set
        orig_size: u32,  // size of original Application Data Unit (this bundle will be smaller)
        crc: u32, // set to 0 until CRC computed on the primary block, then added back
    }

    impl PriBlock {
        pub fn new() -> PriBlock {
            PriBlock {
                version: 7,
                flags: 0,
                crc_type: 2,
                //  d_eid: "dtn://casper/test".to_owned(),
                d_eid: Eid::new(Some("//casper/test".to_string())),
                s_eid: Eid::new(None),
                r_eid: Eid::new(None),
                ts_sec: 0, // 946684    800 is y2k
                ts_inc: 0,
                life: 0xffffffff,
                frag_offs: 0,
                orig_size: 0,
                crc: 0,
            }
        }

        pub fn set_d_eid(&mut self, eid:Eid) {
            self.d_eid = eid;
        }
        pub fn set_s_eid(&mut self, eid:Eid) {
            self.s_eid = eid;
        }
        pub fn set_r_eid(&mut self, eid:Eid) {
            self.r_eid = eid;
        }


    }

    impl super::Block for PriBlock {
        fn get_id() -> u8 { 0 }
        fn get_type() -> u8 { 0 } 
    }

    impl ConvertCbor for PriBlock {

        fn as_bytes(&self) -> Vec<u8> {
            to_vec(& self.as_cbor()).unwrap()
            // out
        }


        fn as_cbor(&self) -> Value { 
            let mut buf = Vec::<Value>::new();
            buf.push(to_value(&self.version).unwrap());
            buf.push(to_value(&self.flags).unwrap());
            buf.push(to_value(&self.crc_type).unwrap());
            buf.push(self.d_eid.as_cbor());
            buf.push(self.s_eid.as_cbor());
            buf.push(self.r_eid.as_cbor());
            let mut ts = Vec::<Value>::new();
            ts.push(to_value(&self.ts_sec).unwrap());
            ts.push(to_value(&self.ts_inc).unwrap());
            buf.push(to_value(ts).unwrap());
            buf.push(to_value(&self.life).unwrap());
            buf.push(to_value(&self.crc).unwrap());
            
            if self.crc_type > 0 {
                let out = to_value(&buf).unwrap();
                let mut out = to_vec(&out).unwrap();
                buf.pop();
                // let test = to_vec(& to_value(&buf).unwrap()).unwrap();
                // println!("After pop {:?}", test);                
                out.pop();
                out.push(0x44);
                out.push(0);
                out.push(0);
                out.push(0);
                out.push(0); // append a fake CRC of 0

                let mut temp: Vec<u8> = vec![159];
                temp.append(&mut out);
                println!("crc test {}", crc32::checksum_castagnoli("crctest".as_bytes()));

                let crc = crc32::checksum_castagnoli(&temp);
                // let crc = crc32::checksum_ieee(&out);

                println!("CRC = {:?}", crc.to_le_bytes());

                println!("{:?}",temp);

                buf.push(Value::Bytes(crc.to_be_bytes().to_vec()));
                //                buf.push(to_value(crc).unwrap();
                // buf.push(Value::Bytes([0,0,0,0].to_vec()));
                let test = to_vec(& to_value(&buf).unwrap()).unwrap();
                println!("After pop {:?}", test);                

                
            }
            Value::Array(buf)    
        }
    }

