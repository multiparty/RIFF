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
use crate::handlers::initialization;
use crate::handlers::sharing;
use crate::handlers::crypto_provider;

pub fn handler_public_keys (riff: Arc<Mutex<RiffClientRest>>, msg: Value) {
    let msg:Value = serde_json::from_str(msg.as_str().unwrap()).unwrap();
    initialization::store_public_keys(riff, msg["public_keys"].clone());
}

pub fn handler_share (riff: Arc<Mutex<RiffClientRest>>, msg: Value) {
    // parse message
    let msg:Value = serde_json::from_str(msg.as_str().unwrap()).unwrap();
    let sender_id = msg["party_id"].clone();

    let mut instance = riff.lock().unwrap();
    if instance.keymap[sender_id.to_string()] != Value::Null {
        std::mem::drop(instance);
        sharing::receive_share(riff.clone(), msg.clone());
        instance = riff.lock().unwrap();
    } else {
        if instance.messagesWaitingKeys[sender_id.to_string()] == Value::Null {
            instance.messagesWaitingKeys.as_object_mut().unwrap().insert(sender_id.to_string(), json!([]));
        }
        instance.messagesWaitingKeys[sender_id.to_string()].as_array_mut().unwrap().push(json!({
            "label": json!("share"),
            "msg": msg,
        }))
    }
}

pub fn handler_open (riff: Arc<Mutex<RiffClientRest>>, msg: Value) {
    // parse message
    let msg:Value = serde_json::from_str(msg.as_str().unwrap()).unwrap();
    let sender_id = msg["party_id"].clone();

    let mut instance = riff.lock().unwrap();

    if instance.keymap[sender_id.to_string()] != Value::Null {
        std::mem::drop(instance);
        sharing::receive_open(riff.clone(), msg.clone());
        instance = riff.lock().unwrap();
    } else {
        if instance.messagesWaitingKeys[sender_id.to_string()] == Value::Null {
            instance.messagesWaitingKeys.as_object_mut().unwrap().insert(sender_id.to_string(), json!([]));
        }
        instance.messagesWaitingKeys[sender_id.to_string()].as_array_mut().unwrap().push(json!({
            "label": json!("open"),
            "msg": msg,
        }))
    }
}

pub fn handler_crypto_provider (riff: Arc<Mutex<RiffClientRest>>, msg: Value) {
    let msg:Value = serde_json::from_str(msg.as_str().unwrap()).unwrap();
    crypto_provider::receive_crypto_provider(riff.clone(), msg);
}

pub fn resolve_messages_waiting_for_keys (riff: Arc<Mutex<RiffClientRest>>) {
    let mut instance = riff.lock().unwrap();
    let keymap = instance.keymap.clone();
    for (party_id, value) in keymap.as_object().unwrap() {
        let messageQueue = instance.messagesWaitingKeys[party_id.clone()].clone();
        if messageQueue == Value::Null {
            continue;
        }
        for msg in messageQueue.as_array().unwrap() {
            if msg["label"] == "share" {
                //this.handlers.receive_share(msg.msg);
            } else if msg["label"] == "open" {

            } else if msg["label"] == "custom" {

            } else {
                panic!("Error resolving pending message: unknown label {}", msg["label"].to_string());
            }
        }

        instance.messagesWaitingKeys.as_object_mut().unwrap().insert(party_id.clone(), Value::Null);

    }
}