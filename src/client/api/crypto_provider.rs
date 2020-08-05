use crate::architecture::counters;
use crate::common::helper;
use crate::ext::RiffClientRest;
use crate::server::restfulAPI::*;
use crate::RiffClient::JsonEnum;
use crate::SecretShare::SecretShare;
use crate::{architecture::hook, RiffClientTrait::RiffClientTrait};
use serde_json::json;
use serde_json::Value;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

//Requests secret(s) from the server (crypto provider) of type matching the given label.

pub fn from_crypto_provider(
    riff: Arc<Mutex<RiffClientRest>>,
    label: String,
    options: HashMap<String, JsonEnum>,
) {
    let mut instance = riff.lock().unwrap();
    // defaults
    let mut Zp = 0;
    if let Some(data) = options.get(&String::from("Zp")) {
        if let JsonEnum::Number(Zp_j) = data {
            Zp = *Zp_j;
        }
    } else {
        Zp = instance.Zp;
    }

    let mut receivers_list = vec![];
    if let Some(data) = options.get(&String::from("receivers_list")) {
        if let JsonEnum::Array(receivers_list_j) = data {
            receivers_list = receivers_list_j.clone();
            receivers_list.sort();
        }
    } else {
        for i in 1..instance.party_count + 1 {
            receivers_list.push(i);
        }
    }

    let mut threshold: i64 = 0;
    if let Some(data) = options.get(&String::from("threshold")) {
        if let JsonEnum::Number(threshold_j) = data {
            if *threshold_j < 0 {
                threshold = 2;
            } else if *threshold_j > receivers_list.len() as i64 {
                threshold = receivers_list.len() as i64;
            } else {
                threshold = *threshold_j;
            }
        }
    } else {
        threshold = receivers_list.len() as i64;
    }

    let mut op_id = String::new();
    if let Some(data) = options.get(&String::from("op_id")) {
        if let JsonEnum::String(str) = data {
            op_id = str.clone();
        }
    } else {
        std::mem::drop(instance);
        let mut op = String::from("crypto_provider:");
        op.push_str(label.as_str());
        op_id = counters::gen_op_id(riff.clone(), op, receivers_list.clone());
        instance = riff.lock().unwrap();
    }

    let mut params = Value::Null;
    if let Some(data) = options.get(&String::from("params")) {
        if let JsonEnum::Value(params_j) = data {
            params = params_j.clone();
        }
    } else {
        params = json!({});
    }

    // Send a request to the server
    let msg = json!({
        "label": label,
        "op_id": op_id,
        "receivers": receivers_list,
        "threshold": threshold,
        "Zp": Zp,
        "params": params,
    });
    let msg_string = msg.to_string();

    // send a request to the server.
    std::mem::drop(instance);
    RiffClientRest::emit(riff.clone(), String::from("crypto_provider"), msg_string);
    instance = riff.lock().unwrap();
}
