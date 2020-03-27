use serde_cbor;
use serde_cbor::value::Value;

pub mod bundle;


fn main() {
    let x = bundle::Bundle::new();

    let b = x.as_bytes();
    println!("{:?}", b);
//    let v = serde_cbor::to_vec(&b).unwrap();
//    println!("{:?}", v);
    let d: Value = serde_cbor::from_slice(&b).unwrap();

    println!("{:?}", d);
}
