use crate::ext::RiffClientRestful::*;
use serde_json::Value;
use crate::architecture::hook;
use serde_json::json;
use crate::architecture;
use crate::{RiffClientTrait::RiffClientTrait, RiffClient::*};
use std::{
    cmp,
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex,MutexGuard},
    thread,
};
use crate::handlers::events;

//Builds the initialization message for this instance
pub fn build_initialization_message (riff_locked: Arc<Mutex<RiffClientRest>>) -> Value{
    let mut riff = riff_locked.lock().unwrap();
    let temp_pubkey;
    if riff.public_key != Value::Null {
        std::mem::drop(riff);
        temp_pubkey = json!(hook::dumpKey(riff_locked.clone()));
        riff = riff_locked.lock().unwrap();
    } else {
        temp_pubkey = Value::Null;
    }
    let mut msg = json!({
        "computation_id": riff.computation_id,
        "party_id": riff.id,
        "party_count": riff.party_count,
        "public_key": temp_pubkey,
    });
    if let Some(data) = riff.options.get(&String::from("initialization")) {
        if let JsonEnum::Value(initialization) = data {
            for (key, value) in initialization.as_object().unwrap() {
                msg.as_object_mut().unwrap().insert(key.clone(), value.clone());
            }
        }
    }
    

    // Initialization Hook
    //return jiffClient.hooks.execute_array_hooks('beforeOperation', [jiffClient, 'initialization', msg], 2);
    msg
}

pub fn connected (riff: Arc<Mutex<RiffClientRest>>) {
    let mut riff_instance = riff.lock().unwrap();
    //let riff_instance = riff.lock().unwrap();
    riff_instance.initialization_counter += 1;

    if riff_instance.secret_key == Value::Null && riff_instance.public_key == Value::Null {
        std::mem::drop(riff_instance);
        let key = architecture::generateKeyPair(riff.clone());
        riff_instance = riff.lock().unwrap();
        match key.0 {
            
            None => riff_instance.public_key = Value::Null,
            Some(publicKey) => riff_instance.public_key = json!(publicKey.0.to_vec()),
        }
        match key.1 {
            None => riff_instance.secret_key = Value::Null,
            Some(secretKey) => riff_instance.secret_key = json!(secretKey.0.to_vec()),
        }
        
    }
    //println!("{:?}", riff_instance.public_key);

    // Initialization message
    std::mem::drop(riff_instance);
    let msg = build_initialization_message(riff.clone());
    riff_instance = riff.lock().unwrap();

    // Emit initialization message to server
    std::mem::drop(riff_instance);
    RiffClientRest::emit(riff.clone(),String::from("initialization"), msg.to_string());

}

pub fn initialized (riff: Arc<Mutex<RiffClientRest>>, msg: Value) {
    //println!("initialized");
    let mut instance = riff.lock().unwrap();
    instance.__initialized = true;
    instance.initialization_counter = 0;
    //println!("{}", msg.as_str());
    let msg:Value = serde_json::from_str(msg.as_str().unwrap()).unwrap();
    instance.id = msg["party_id"].clone();
    instance.party_count = msg["party_count"].as_i64().unwrap();

    //jiffClient.socket.resend_mailbox(); do nothing in rest ext
    std::mem::drop(instance);
    store_public_keys(riff.clone(), msg["public_keys"].clone());

}

pub fn store_public_keys (riff: Arc<Mutex<RiffClientRest>>, keymap: Value) {
    let mut instance = riff.lock().unwrap();
    for (key, value) in keymap.as_object().unwrap() {
        if instance.keymap[key.clone()] == Value::Null {
            std::mem::drop(instance);
            let v = hook::parseKey(riff.clone(), &keymap[key.clone()]).unwrap();
            instance = riff.lock().unwrap();
            instance.keymap.as_object_mut().unwrap().insert(key.clone(), json!(v));
        }
    }

    // Resolve any pending messages that were received before the sender's public key was known
    std::mem::drop(instance);
    events::resolve_messages_waiting_for_keys(riff.clone());
    instance = riff.lock().unwrap();

    // Resolve any pending waits that have satisfied conditions
    //jiffClient.execute_wait_callbacks();

    // Check if all keys have been received
    if instance.keymap["s1"] == Value::Null {
        return 
    }

    for (key, value) in instance.keymap.as_object().unwrap() {
        if *value == Value::Null {
            return
        }
    }

    // all parties are connected; execute callback
    if instance.__ready != true && instance.__initialized {
        instance.__ready = true;
        if let Some(data) =  instance.options.clone().get(&String::from("onConnect")) {
            if let JsonEnum::func(onConnect) = data {
                std::mem::drop(instance);
                onConnect(riff.clone());
            }
        }
    }


} 