// use serde_cbor;
// use serde_cbor::value::{Value};
// use serde_cbor::to_vec;

// use std::error::Error;
// use std::fs::File;
// use std::io::prelude::*;
// use std::path::Path;

// use dtn;


// pub mod bundle;


fn main() {
//     let mut x = bundle::Bundle::new();

//     x.set_payload("Woohoo!\n".as_bytes().to_vec());
//     let b: Vec<u8> = x.as_bytes();
//     println!("{:?}", b);

//     let mut stcp_out = Vec::<Value>::new();
//     stcp_out.push(Value::Integer(b.len() as i128));
//     stcp_out.push(Value::Null);

//  //   stcp_out.push(x.as_cbor())
//     // println!("v = {:?}",to_vec(&v).unwrap());
//     // stcp_out.push(v);   



//     let buf: Vec<u8> = to_vec(&stcp_out).unwrap();
//     // buf.pop();
//     // buf.append(&mut b);



//     let path = Path::new("out/bundle.dat");
//     let display = path.display();
//     let mut file = match File::create(&path) {
//         Err(why) => panic!("couldn't create {}: {}", display, why),
//         Ok(file) => file,
//     };
 
//     match file.write_all(&buf) {
//         Err(why) => panic!("couldn't write to {}: {}", display, why),
//         Ok(_) => println!("successfully wrote to {}", display),
//     }





// //    let v = serde_cbor::to_vec(&b).unwrap();
// //    println!("{:?}", v);
//     let d: Value = serde_cbor::from_slice(&buf).unwrap();

//     println!("{:?}", d);
}
