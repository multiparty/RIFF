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

//use futures::lock::Mutex as fMutex;
/*
 * Default way of computing shares (can be overridden using hooks).
 * Compute the shares of the secret (as many shares as parties) using Shamir secret sharing
 * @ignore
 * @function jiff_compute_shares
 * @param {module:jiff-client~JIFFClient} jiff - the jiff instance
 * @param {number} secret - the secret to share.
 * @param {Array} parties_list - array of party ids to share with.
 * @param {number} threshold - the min number of parties needed to reconstruct the secret, defaults to all the receivers.
 * @param {number} Zp - the mod.
 * @returns {object} a map between party number and its share, this means that (party number, share) is a
 *          point from the polynomial.
 *
 */

pub fn jiff_compute_shares(
    secret: Value,
    parties_list: Value,
    threshold: Value,
    Zp: Value,
) -> Value {
    let mut shares = json!({}); // Keeps the shares
    let mut i = 1;

    // Each player's random polynomial f must have
    // degree threshold - 1, so that threshold many points are needed
    // to interpolate/reconstruct.
    let t = (threshold.as_u64().unwrap() - 1) as usize;
    let mut polynomial = vec![Value::Null; t + 1];

    // Each players's random polynomial f must be constructed
    // such that f(0) = secret
    polynomial[0] = secret;

    // Compute the random polynomial f's coefficients
    while i <= t {
        polynomial[i] = json!(helper::random(Zp.clone()));
        i = i + 1;
    }

    // Compute each players share such that share[i] = f(i)
    for party in parties_list.as_array().unwrap() {
        let p_id = party.clone();
        shares
            .as_object_mut()
            .unwrap()
            .insert(p_id.clone().to_string(), polynomial[0].clone());
        let mut power = helper::get_party_number(p_id.clone());

        // let mut j = 1;
        // while j < polynomial.len() {
        //     let tmp = helper::modF(json!(polynomial[j].as_i64().unwrap() * power.as_i64().unwrap()), Zp.clone());
        //     let temp_share = shares[p_id.to_string()].as_i64().unwrap();
        //     shares.as_object_mut().unwrap().insert(p_id.clone().to_string(), json!(helper::modF(json!(temp_share + tmp), Zp.clone())));
        //     power = json!(helper::modF(json!(power.as_i64().unwrap() * helper::get_party_number(p_id.clone()).as_i64().unwrap()), Zp.clone()));
        //     println!("power: {:?}", power);
        //     j = j + 1;
        // }
        for j in 1..polynomial.len() {
            let tmp = helper::modF(
                json!(polynomial[j].as_i64().unwrap() * power.as_i64().unwrap()),
                Zp.clone(),
            );
            let temp_share = shares[p_id.to_string()].as_i64().unwrap();
            shares.as_object_mut().unwrap().insert(
                p_id.clone().to_string(),
                json!(helper::modF(json!(temp_share + tmp), Zp.clone())),
            );
            power = json!(helper::modF(
                json!(
                    power.as_i64().unwrap()
                        * helper::get_party_number(p_id.clone()).as_i64().unwrap()
                ),
                Zp.clone()
            ));
        }
    }
    return shares;
}

pub fn riff_share(
    riff: Arc<Mutex<RiffClientRest>>,
    secret_p: Option<i64>,
    options: HashMap<String, JsonEnum>,
) -> Vec<SecretShare> {
    //print!("shamir");
    let mut instance = riff.lock().unwrap();
    // defaults
    let mut secret = 0;
    if let Some(data) = secret_p {
        secret = data;
    }

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

    // if party is uninvolved in the share, do nothing
    if let None = receivers_list.iter().position(|&x| x == instance.id) {
        if let None = senders_list.iter().position(|&x| x == instance.id) {
            return vec![];
        }
    }

    // compute operation id
    let mut share_id = String::new();
    if let Some(data) = options.get(&String::from("share_id")) {
        if let JsonEnum::String(str) = data {
            share_id = str.clone();
        }
    } else {
        std::mem::drop(instance);
        //println!("share_id");
        share_id = counters::gen_op_id2(
            riff.clone(),
            String::from("share"),
            receivers_list.clone(),
            senders_list.clone(),
        );
        instance = riff.lock().unwrap();
    }

    let mut shares = json!({});
    // stage sending of shares
    if let Some(_) = senders_list.iter().position(|&x| x == instance.id) {
        // compute shares
        shares = jiff_compute_shares(
            json!(secret),
            json!(receivers_list),
            json!(threshold),
            json!(Zp),
        );
        //println!("shares {:?}", shares);
        // send shares
        for receiver in receivers_list.clone() {
            if receiver == instance.id {
                continue;
            }

            // send encrypted and signed shares_id[p_id] to party p_id
            let mut msg = json!({
                "party_id": receiver,
                "share": shares[receiver.to_string()],
                "op_id": json!(share_id),
            });
            //println!("ms{:?}", msg["share"]);
            let pubkey = instance.keymap[msg["party_id"].to_string()].clone(); // without ""

            //println!("{:?}", instance.keymap);
            //println!("{:?}",msg["party_id"]);
            // println!("{:?}", pubkey);
            let seckey = instance.secret_key.clone();
            std::mem::drop(instance);
            let mut res = hook::encryptSign(riff.clone(), msg["share"].clone(), pubkey, seckey);
            if res.is_number() {
                res = json!(res.as_i64().unwrap().to_string());
            }
            //println!("after encrypted");
            instance = riff.lock().unwrap();
            msg.as_object_mut()
                .unwrap()
                .insert(String::from("share"), res);
            std::mem::drop(instance);
            RiffClientRest::emit(riff.clone(), String::from("share"), msg.to_string());

            instance = riff.lock().unwrap();
            //println!("mailbox: {:?}", instance.mailbox.current["messages"]);
        }
    }

    // stage receiving of shares
    let mut result: Vec<SecretShare> = Vec::new();
    result.push(SecretShare::new(0, receivers_list.clone(), threshold, Zp));

    if let Some(_) = receivers_list.iter().position(|&x| x == instance.id) {
        let mut _remaining = senders_list.len();
        for sender in senders_list {
            if sender == instance.id {
                // Keep party's own share
                let my_share = shares[sender.to_string()].clone().as_i64().unwrap();
                result.push(SecretShare::new(
                    my_share,
                    receivers_list.clone(),
                    threshold,
                    Zp,
                ));
                _remaining -= 1;
                continue;
            }
            //let share_from_other;
            std::mem::drop(instance);
            //TODO: async/await implementation - await the incoming shares without blocking
            //      should be able to avoid locking and unlocking the RIFF Instance frequently
            loop {
                instance = riff.lock().unwrap();
                if let Some(data) = instance.share_map.get(&share_id) {
                    //println!("share_id {:?}", share_id);
                    if let Some(share) = data.get(&sender) {
                        result.push(SecretShare::new(
                            *share,
                            receivers_list.clone(),
                            threshold,
                            Zp,
                        ));
                        break;
                    }
                }
                std::mem::drop(instance);
                thread::sleep(Duration::from_millis(100));
            }

            //let b = result.get(&sender).unwrap();
            //a shared data structure A to store received share
            //a async function B to get the real value from A
            //store the reference of the  future of B in ds C
            //when reveice, call future in ds C await
        }
    }
    return result;
}

// pub async fn get_share (riff: Arc<Mutex<RiffClientRest>>, op_id: String, sender: i64) -> i64{
//     let instance = riff.lock().unwrap();
//     return *instance.share_map.get(&op_id).unwrap().get(&sender).unwrap()
// }
