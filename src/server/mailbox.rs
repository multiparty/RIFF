use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
};
use serde_json::json;
use serde_json::Value;
use crate::server::restfulAPI::restfulAPI;

pub fn put_in_mailbox(riff: &mut restfulAPI , label: String, msg: String, computation_id: &Value, to_id: &Value) ->  usize{
    let mut computation_mailbox = riff.mail_box[computation_id.to_string()].clone();
    if computation_mailbox[to_id.to_string()] == Value::Null {
        computation_mailbox.as_object_mut().unwrap().insert(to_id.to_string(), json!([]));
    }

    // add message to mailbox, return pointer
    computation_mailbox[to_id.to_string()].as_array_mut().unwrap().push(json!({
        "label": label,
        "msg": msg,
    }));
    return computation_mailbox[to_id.to_string()].as_array_mut().unwrap().len() - 1;
}

pub fn get_from_mailbox(mailbox:&mut HashMap<u32, HashMap<u32, Vec<String>>>, computation_id: u32, party_id: u32) -> Vec<String> {
    let mut res = Vec::new();
    let computation_mailbox = mailbox.get_mut(&computation_id).unwrap();
    res = computation_mailbox.get_mut(&party_id).unwrap().clone();
    res
}