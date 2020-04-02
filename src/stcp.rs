use super::bundle::Bundle;
use serde_cbor::Value;
use serde_cbor::to_vec;

// Helper functions for stcp 

pub fn encapsulate_stcp(bundle: Bundle) -> Vec<u8> {
    let mut buffer: Vec<Value> = Vec::new();
    let mut bunbuffer = bundle.as_bytes();
    buffer.push(Value::Integer(bunbuffer.len() as i128));
    buffer.push(Value::Null);
    let mut buffer = to_vec(&Value::Array(buffer)).unwrap();
    buffer.pop();
    buffer.append(&mut bunbuffer);
    buffer
}