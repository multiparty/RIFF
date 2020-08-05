use crate::client::architecture::hook;
use serde_json::json;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    //thread,
};

use crate::ext::RiffClientRestful::RiffClientRest;

pub fn receive_share(riff: Arc<Mutex<RiffClientRest>>, mut msg: Value) {
    let mut instance = riff.lock().unwrap();
    // Decrypt share
    let secret_key = instance.secret_key.clone();
    let signing_public_key = instance.keymap[msg["party_id"].to_string()].clone();
    let encrypted_message = msg["share"].clone();
    std::mem::drop(instance);
    let decrpted = hook::decryptSign(
        riff.clone(),
        encrypted_message,
        secret_key,
        signing_public_key,
    );
    instance = riff.lock().unwrap();
    let decrpted = decrpted.as_array().unwrap().to_owned();
    let mut Decrpted = [0; 8];

    for i in 0..8 {
        Decrpted[i] = decrpted[i].as_u64().unwrap() as u8;
    }

    let decrpted_ten_integer: i64 = i64::from_be_bytes(Decrpted);
    msg.as_object_mut()
        .unwrap()
        .insert(String::from("share"), json!(decrpted_ten_integer));

    let sender_id = msg["party_id"].clone();
    let op_id = msg["op_id"].clone();
    let share = msg["share"].clone();
    //println!("share_id received: {:?}", op_id);
    instance
        .share_map
        .entry(op_id.as_str().unwrap().to_string())
        .or_insert(HashMap::new())
        .insert(sender_id.as_i64().unwrap(), share.as_i64().unwrap());
}

pub fn receive_open(riff: Arc<Mutex<RiffClientRest>>, mut msg: Value) {
    let mut instance = riff.lock().unwrap();
    // Decrypt share

    if msg["party_id"] != instance.id {
        let secret_key = instance.secret_key.clone();
        let signing_public_key = instance.keymap[msg["party_id"].to_string()].clone();
        let encrypted_message = msg["share"].clone();
        std::mem::drop(instance);
        let decrpted = hook::decryptSign(
            riff.clone(),
            encrypted_message,
            secret_key,
            signing_public_key,
        );
        instance = riff.lock().unwrap();
        let decrpted = decrpted.as_array().unwrap().to_owned();
        let mut Decrpted = [0; 8];

        for i in 0..8 {
            Decrpted[i] = decrpted[i].as_u64().unwrap() as u8;
        }

        let decrpted_ten_integer: i64 = i64::from_be_bytes(Decrpted);
        msg.as_object_mut()
        .unwrap()
        .insert(String::from("share"), json!(decrpted_ten_integer));
    }

        let sender_id = msg["party_id"].clone();
        let op_id = msg["op_id"].clone();
        let share = msg["share"].clone();
        let Zp = msg["Zp"].clone();

        // call hook

        // Accumulate received shares
        let shares = instance.open_map.entry(op_id.as_str().unwrap().to_string()).or_insert(vec![]);
        shares.push(json!({
            "value": share,
            "sender_id": sender_id,
            "Zp": Zp,
        }));

        // to-do: Clean up if done


    
}
