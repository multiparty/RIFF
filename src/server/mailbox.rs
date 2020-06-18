use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
};

pub fn put_in_mailbox(mailbox:&mut HashMap<u32, HashMap<u32, Vec<String>>>, computation_id: u32, party_id: u32, msg: String) -> &Vec<String> {
    let computation_mailbox = mailbox.get_mut(&computation_id).unwrap();
    computation_mailbox.entry(party_id).or_insert(Vec::new()).push(msg);
    computation_mailbox.get_mut(&party_id).unwrap()
}

pub fn get_from_mailbox(mailbox:&mut HashMap<u32, HashMap<u32, Vec<String>>>, computation_id: u32, party_id: u32) -> Vec<String> {
    let mut res = Vec::new();
    let computation_mailbox = mailbox.get_mut(&computation_id).unwrap();
    res = computation_mailbox.get_mut(&party_id).unwrap().clone();
    res
}