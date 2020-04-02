use std::fmt;
// use std::str::Split;
use super::bundle::ConvertCbor;

use serde_cbor;
use serde_cbor::value::{Value};
use serde_cbor::{to_vec};

pub struct Eid {
    scheme: u8,  // 1 dtn 2 ipn 251 api:
    ssp: Option<String>,
}

impl Eid {
    pub fn new(ssp: Option<String>) -> Eid {
        Eid {
            scheme: 1,
            ssp
        }
    }

    pub fn new_uri(uri: &String) -> Eid {
        let mut split = uri.split(":");
        let scheme = split.next().unwrap();
        let ssp = split.next().unwrap();

        let scheme = match scheme {
            "dtn" => { 1 },
            "ipn" => { 2 },
            _ => { 255 }
        };

        let ssp = match (scheme, ssp) {
            (1, "none") => None,
            (_, _) => Some(ssp.to_string()),
        };

        Eid {
            scheme,
            ssp
        }
    }

}

impl fmt::Display for Eid {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        match (self.scheme, &self.ssp) {
            (1, Some(ssp)) => { 
                write!(f,"dtn:{}", ssp)?;
            },
            (1, None) => {
                write!(f,"dtn::null")?;
            },
            (2, Some(ssp)) => {
                write!(f,"ipn:{}", ssp)?;
            },
            _ => {}

        }

        Ok(())
    }
}

impl ConvertCbor for Eid {

    fn as_bytes(&self) -> std::vec::Vec<u8> { 
        to_vec(&self.as_cbor()).unwrap()
    }
    
    fn as_cbor(&self) -> Value { 
        let mut buf = Vec::<Value>::new();
        buf.push(Value::Integer(self.scheme as i128));
        match &self.ssp {
            Some(ssp) => buf.push(Value::Text(ssp.to_string())),
            None => buf.push(Value::Integer(0)),
        }
        Value::Array(buf)
    }
}