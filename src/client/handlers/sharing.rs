use serde_json::Value;
use std::{
    //collections::HashMap,
    sync::{Arc, Mutex},
    //thread,
};
use crate::client::architecture::hook;
use serde_json::json;

use crate::ext::RiffClientRestful::RiffClientRest;

pub fn receive_share (riff: Arc<Mutex<RiffClientRest>>,mut msg: Value) {
    let mut instance = riff.lock().unwrap();
    // Decrypt share
    let secret_key = instance.secret_key.clone();
    let signing_public_key = instance.keymap[msg["party_id"].to_string()].clone();
    let encrypted_message = msg["share"].clone();
    std::mem::drop(instance);
    let decrpted = hook::decryptSign(riff.clone(), encrypted_message, secret_key, signing_public_key);
    instance = riff.lock().unwrap();
    let decrpted = decrpted.as_array().unwrap().to_owned();
    let mut Decrpted = [0; 8];
    
    for i in 0..8 {
        Decrpted[i] = decrpted[i].as_u64().unwrap() as u8;
    }

    let decrpted_ten_integer: i64 = i64::from_be_bytes(Decrpted); 
    msg.as_object_mut().unwrap().insert(String::from("share"), json!(decrpted_ten_integer));
    
    let sender_id = msg["party_id"].clone();
    let op_id = msg["op_id"].clone();
    let share = msg["share"].clone();

    // check if a deferred is set up (maybe the share was received early)
    // if instance.deferreds[op_id.to_string()] == Value::Null {
    //     instance.deferreds.as_object_mut().unwrap().insert(op_id.to_string(), json!({}));
    // }

    // if instance.deferreds[op_id.to_string()][sender_id.to_string()] == Value::Null {
    //     // Share is received before deferred was setup, store it.

    // }
}