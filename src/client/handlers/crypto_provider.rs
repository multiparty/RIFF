use crate::client::architecture::hook;
use crate::ext::RiffClientRestful::RiffClientRest;
use crate::RiffClient::JsonEnum;
use crate::SecretShare::SecretShare;
use serde_json::json;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    //thread,
};

pub fn receive_crypto_provider(riff: Arc<Mutex<RiffClientRest>>, msg: Value) {
    let op_id = msg["op_id"].clone();
    // parse msg
    let receivers_list = msg["receivers"].clone();
    let mut receivers_list_vec = vec![];
    for rc in receivers_list.as_array().unwrap() {
        receivers_list_vec.push(rc.as_i64().unwrap());
    }
    let threshold = msg["threshold"].clone();
    let Zp = msg["Zp"].clone();

    // construct secret share objects
    let mut result: HashMap<String, JsonEnum> = HashMap::new();
    if msg["values"] != Value::Null {
        result.insert(
            String::from("values"),
            JsonEnum::Value(msg["values"].clone()),
        );
    }
    if msg["shares"] != Value::Null {
        result.insert(String::from("shares"), JsonEnum::ArrayShare(vec![]));
        for share in msg["shares"].as_array().unwrap() {
            if let Some(data) = result.get_mut(&String::from("shares")) {
                if let JsonEnum::ArrayShare(shares) = data {
                    shares.push(SecretShare::new(
                        share.as_i64().unwrap(),
                        receivers_list_vec.clone(),
                        threshold.as_i64().unwrap(),
                        Zp.as_i64().unwrap(),
                    ));
                }
            }
        }
    }

    let mut instance = riff.lock().unwrap();
    instance
        .crypto_map
        .insert(op_id.as_str().unwrap().to_string(), result);
}
