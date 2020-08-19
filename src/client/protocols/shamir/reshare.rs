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
use crate::protocols::shamir;

pub fn shamir_reshare (riff: Arc<Mutex<RiffClientRest>>, options: HashMap<String, JsonEnum>) -> Option<SecretShare> {
    // default values
    let mut instance = riff.lock().unwrap();

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

    let mut senders_list = vec![];
    if let Some(data) = options.get(&String::from("senders_list")) {
        if let JsonEnum::Array(sender_list_j) = data {
            senders_list = sender_list_j.clone();
            senders_list.sort();
        }
    } else {
        for i in 1..instance.party_count + 1 {
            senders_list.push(i);
        }
    }

    let mut threshold: i64 = 0;
    if let Some(data) = options.get(&String::from("threshold")) {
        if let JsonEnum::Number(threshold_j) = data {
            
                threshold = *threshold_j;
            
        }
    } else {
        threshold = receivers_list.len() as i64;
    }

    let mut Zp = 0;
    if let Some(data) = options.get(&String::from("Zp")) {
        if let JsonEnum::Number(Zp_j) = data {
            Zp = *Zp_j;
        }
    } else {
        Zp = instance.Zp;
    }

    let mut op_id = String::new();
    if let Some(data) = options.get(&String::from("op_id")) {
        if let JsonEnum::String(str) = data {
            op_id = str.clone();
        }
    } else {
        std::mem::drop(instance);
        //println!("share_id");
        op_id = counters::gen_op_id(
            riff.clone(),
            String::from("reshare"),
            senders_list.clone(),
        );
        instance = riff.lock().unwrap();
    }
    let mut share:Option<SecretShare>;
    if let Some(data) = options.get(&String::from("share")) {
        if let JsonEnum::SingleShare(share_i) = data {
            share = Some(share_i.clone());
        }
    } else {
        share = None;
    }

    // Check if this party is a sender or receiver
    let mut isSender = false;
    let mut isReceiver = false;
    if let Some(_) = senders_list.iter().position(|&x| x == instance.id) {
        isSender = true;
    }
    if let Some(_) = receivers_list.iter().position(|&x| x == instance.id) {
        isReceiver = true;
    }
    if !isSender && !isReceiver {
        return None
    }

    // optimization, if nothing changes, keep share
    if let Some(_) = share {
        if receivers_list == senders_list && threshold == share.unwrap().threshold {
            return share
        }
         
    }

    // the value of the share has been received.
    if isSender {
        std::mem::drop(instance);
        //let secret_p = Some(share.value);
        let mut options_share = HashMap::new();
        options_share.insert(String::from("threshold"), JsonEnum::Number(threshold));
        options_share.insert(String::from("receivers_list"), JsonEnum::Array(receivers_list));
        options_share.insert(String::from("senders_list"), JsonEnum::Array(senders_list));
        options_share.insert(String::from("Zp"), JsonEnum::Number(Zp));
        options_share.insert(String::from("share_id"), JsonEnum::String(op_id));
        let intermediate_shares = shamir::riff_share(riff, Some(share.unwrap().value), options_share);
        instance = riff.lock().unwrap();
    } else {
        std::mem::drop(instance);
        //let secret_p = Some(share.value);
        let mut options_share = HashMap::new();
        options_share.insert(String::from("threshold"), JsonEnum::Number(threshold));
        options_share.insert(String::from("receivers_list"), JsonEnum::Array(receivers_list));
        options_share.insert(String::from("senders_list"), JsonEnum::Array(senders_list));
        options_share.insert(String::from("Zp"), JsonEnum::Number(Zp));
        options_share.insert(String::from("share_id"), JsonEnum::String(op_id));
        let intermediate_shares = shamir::riff_share(riff, None, options_share);
        instance = riff.lock().unwrap();
    }

    return Some(share.unwrap())

    
    









}