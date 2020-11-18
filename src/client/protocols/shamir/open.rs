use crate::architecture::counters;
use crate::common::helper;
use crate::ext::RiffClientRest;
use crate::handlers::sharing;
use crate::server::restfulAPI::*;
use crate::util::helpers;
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

pub fn riff_open(
    riff: Arc<Mutex<RiffClientRest>>,
    mut share: SecretShare,
    options: HashMap<String, JsonEnum>,
) -> Option<i64> {
    //println!("riff_open");
    let mut instance = riff.lock().unwrap();
    // Default values
    let mut parties = vec![];
    if let Some(data) = options.get(&String::from("parties")) {
        if let JsonEnum::Array(parties_l) = data {
            parties = parties_l.clone();
            parties.sort();
        }
    } else {
        for i in 1..instance.party_count + 1 {
            parties.push(i);
        }
    }

    // If not a receiver nor holder, do nothing
    if let None = share.holders.iter().position(|&x| x == instance.id) {
        if let None = parties.iter().position(|&x| x == instance.id) {
            return None;
        }
    }

    // Compute operation ids (one for each party that will receive a result
    let mut op_id = String::new();
    if let Some(data) = options.get(&String::from("op_id")) {
        if let JsonEnum::String(str) = data {
            op_id = str.clone();
        }
    } else {
        std::mem::drop(instance);
        //println!("share_id");
        op_id = counters::gen_op_id2(
            riff.clone(),
            String::from("open"),
            parties.clone(),
            share.holders.clone(),
        );
        //println!("gen_op_id2: {}", op_id);
        instance = riff.lock().unwrap();
    }
    //println!("op_id in open: {}", op_id);

    // Party is a holder
    if let Some(_) = share.holders.iter().position(|&x| x == instance.id) {
        //println!("share.holders.iter().position");
        // Call hook
        //share = jiff.hooks.execute_array_hooks('beforeOpen', [jiff, share, parties], 1);

        // refresh/reshare, so that the original share remains secret, instead
        // a new share is sent/open without changing the actual value.
        std::mem::drop(instance);
        let mut options = HashMap::new();
        let mut refresh_op_id = op_id.clone();
        refresh_op_id.push_str(":refresh");
        options.insert(String::from("op_id"), JsonEnum::String(refresh_op_id));
        share = share.refresh(riff.clone(), options);
        instance = riff.lock().unwrap();

        // The given share has been computed, broadcast it to all parties
        //instance.pending_opens += 1;
        std::mem::drop(instance);
        jiff_broadcast(riff.clone(), share.clone(), parties.clone(), op_id.clone());
        instance = riff.lock().unwrap();
    }

    // Party is a receiver
    if let Some(_) = parties.iter().position(|&x| x == instance.id) {
        instance.open_finished = false;
        std::mem::drop(instance);
        //TODO: async/await implementation - await the incoming shares without blocking
        //      should be able to avoid locking and unlocking the RIFF Instance frequently
        loop {

            if let Some(shares) = instance.open_map.get(&op_id) {
                if shares.len() as i64 == share.threshold {
                    let recons_secret = jiff_lagrange(shares.clone());
                    instance.open_finished = true;
                    return Some(recons_secret);
                }
            }
            std::mem::drop(instance);
            thread::sleep(Duration::from_millis(100));
        }
    }
    return None;
}

pub fn jiff_broadcast(
    riff: Arc<Mutex<RiffClientRest>>,
    share: SecretShare,
    parties: Vec<i64>,
    op_id: String,
) {
    println!("in jiff broadcast");
    let mut instance = riff.lock().unwrap();
    for party in parties {
        if party == instance.id {
            //to-do: jiff.handlers.receive_open({ party_id: i, share: share.value, op_id: op_id, Zp: share.Zp });
            std::mem::drop(instance);
            sharing::receive_open(
                riff.clone(),
                json!({
                    "party_id": party,
                    "share": share.value,
                    "op_id": op_id,
                    "Zp": share.Zp,
                }),
            );
            instance = riff.lock().unwrap();
            continue;
        }

        // encrypt, sign and send
        let mut msg = json!({
            "party_id": party,
            "share": share.value,
            "op_id": op_id,
            "Zp": share.Zp,
        });
        let pubkey = instance.keymap[msg["party_id"].to_string()].clone();
        let seckey = instance.secret_key.clone();
        std::mem::drop(instance);
        let mut after_encrypted =
            hook::encryptSign(riff.clone(), msg["share"].clone(), pubkey, seckey);
        if after_encrypted.is_number() {
            after_encrypted = json!(after_encrypted.as_i64().unwrap().to_string());
        }
        //println!("send open! {}", after_encrypted);
        instance = riff.lock().unwrap();
        msg.as_object_mut()
            .unwrap()
            .insert(String::from("share"), after_encrypted);
        std::mem::drop(instance);
        RiffClientRest::emit(riff.clone(), String::from("open"), msg.to_string());
        instance = riff.lock().unwrap();
    }
}

pub fn jiff_lagrange(shares: Vec<Value>) -> i64 {
    let mut lagrange_coeff = vec![0; 100];
    // Compute the Langrange coefficients at 0.
    for share_out in shares.clone() {
        let pi = helper::get_party_number(share_out["sender_id"].clone())
            .as_i64()
            .unwrap();
        lagrange_coeff[pi as usize] = 1;

        for share_in in shares.clone() {
            let pj = helper::get_party_number(share_in["sender_id"].clone())
                .as_i64()
                .unwrap();
            if pj != pi {
                let inv = helpers::extended_gcd(pi - pj, share_out["Zp"].as_i64().unwrap()).0;
                lagrange_coeff[pi as usize] = helper::modF(
                    json!(lagrange_coeff[pi as usize] * (0 - pj)),
                    share_out["Zp"].clone(),
                ) * inv;
                lagrange_coeff[pi as usize] =
                    helper::modF(json!(lagrange_coeff[pi as usize]), share_out["Zp"].clone());
            }
        }
    }

    // Reconstruct the secret via Lagrange interpolation
    let mut recons_secret = 0;
    for share in shares.clone() {
        let party = helper::get_party_number(share["sender_id"].clone())
            .as_i64()
            .unwrap();
        let tmp = helper::modF(
            json!(share["value"].as_i64().unwrap() * lagrange_coeff[party as usize]),
            share["Zp"].clone(),
        );
        // let tmp = helper::modF(
        //     json!(share["value"]
        //         .as_i64()
        //         .unwrap()
        //         .wrapping_mul(lagrange_coeff[party as usize])),
        //     share["Zp"].clone(),
        // );
        recons_secret = helper::modF(json!(recons_secret + tmp), share["Zp"].clone());
    }

    return recons_secret;
}
