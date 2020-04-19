use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum UserMgrMessage {}

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    id: u32,
    name: String,
    anonymous: bool,
    pubkey: Option<Vec<u8>>,
}

impl Default for User {
    fn default() -> User {
        User {
            id: 0,
            name: "".to_string(),
            anonymous: true,
            pubkey: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Users {
    users: HashMap<String, User>,
}