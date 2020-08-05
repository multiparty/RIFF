
use serde_json::Value;
use serde_json::json;
use crate::common::helper;
use std::{
    cmp,
    collections::HashMap,
    env,
    
    sync::{Arc, Mutex, MutexGuard},
    thread,
};
use crate::ext::RiffClientRest;
use crate::RiffClient::JsonEnum;

pub fn gen_op_id2 (riff: Arc<Mutex<RiffClientRest>>, op: String, receivers: Vec<i64>, senders: Vec<i64>) -> String{
    let mut instance = riff.lock().unwrap();
    let mut label = instance.op_id_seed.clone();
    label.push_str(op.as_str());
    label.push_str(":");
    //let join_rec = receivers.join(",");
    let str_nums: Vec<String> = senders.iter() 
        .map(|n| n.to_string())  // map every integer to a string
        .collect();
    let str_nums = str_nums.join(",");
    label.push_str(str_nums.as_str());
    label.push_str(":");
    let str_nums: Vec<String> = receivers.iter() 
        .map(|n| n.to_string())  // map every integer to a string
        .collect();
    let str_nums = str_nums.join(",");
    label.push_str(str_nums.as_str());

    if instance.op_count[label.clone()] == Value::Null {
        instance.op_count.as_object_mut().unwrap().insert(label.clone(), json!(0));
    }
    
    let count = instance.op_count[label.clone()].as_i64().unwrap();
    label.push_str(":");
    label.push_str(count.to_string().as_str());
    instance.op_count.as_object_mut().unwrap().insert(label.clone(), json!(count + 1));
    return label

}

pub fn gen_op_id (riff: Arc<Mutex<RiffClientRest>>, op: String, holders: Vec<i64>) -> String {
    let mut instance = riff.lock().unwrap();
    let mut label = instance.op_id_seed.clone();
    label.push_str(op.as_str());
    label.push_str(":");
    let str_nums: Vec<String> = holders.iter() 
        .map(|n| n.to_string())  // map every integer to a string
        .collect();
    let str_nums = str_nums.join(",");
    label.push_str(str_nums.as_str());
    if instance.op_count[label.clone()] == Value::Null {
        instance.op_count.as_object_mut().unwrap().insert(label.clone(), json!(0));
    }
    let count = instance.op_count[label.clone()].as_i64().unwrap();
    label.push_str(":");
    label.push_str(count.to_string().as_str());
    instance.op_count.as_object_mut().unwrap().insert(label.clone(), json!(count + 1));
    return label
}