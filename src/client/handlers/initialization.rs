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

//Builds the initialization message for this instance
pub fn build_initialization_message (riff_locked: Arc<Mutex<riffClientRest>>) -> Value{
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
    if let JsonEnum::Value(initialization) = riff.options.get(&String::from("initialization")).unwrap() {
        for (key, value) in initialization.as_object().unwrap() {
            msg.as_object_mut().unwrap().insert(key.clone(), value.clone());
        }
    }

    // Initialization Hook
    //return jiffClient.hooks.execute_array_hooks('beforeOperation', [jiffClient, 'initialization', msg], 2);
    msg
}

pub fn connected (mut riff: Arc<Mutex<riffClientRest>>) {
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

    // Initialization message
    std::mem::drop(riff_instance);
    let msg = build_initialization_message(riff.clone());
    riff_instance = riff.lock().unwrap();

    // Emit initialization message to server
    std::mem::drop(riff_instance);
    riffClientRest::emit(riff.clone(),String::from("initialization"), msg.to_string());

}

pub fn initialized (riff: &mut riffClientRest, msg: String) {

}