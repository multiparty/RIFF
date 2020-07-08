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

pub fn get_from_mailbox(riff: &mut restfulAPI, computation_id: Value, party_id: Value) -> Vec<Value> {
    let mut computation_mailbox = riff.mail_box[computation_id.to_string()].clone();
    if computation_mailbox == Value::Null {
        return Vec::new()
    }
    if computation_mailbox[party_id.to_string()] == Value::Null {
        computation_mailbox.as_object_mut().unwrap().insert(party_id.to_string(), json!([]));
    }

    let mut result = Vec::new();
    let mut counter = 0;
    for item in computation_mailbox[party_id.to_string()].as_array().unwrap() {
        result.push(json!({
            "id": json!(counter),
            "label": item[String::from("label")].clone(),
            "msg": item[String::from("msg")].clone(),
        }));
        counter = counter + 1;
    }
    return result
    
}

