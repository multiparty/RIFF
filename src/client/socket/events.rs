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

pub fn resolve_messages_waiting_for_keys (riff: Arc<Mutex<riffClientRest>>) {
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