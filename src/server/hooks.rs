use crate::server::datastructure::intervals::*;
use crate::server::restfulAPI::restfulAPI;
use serde_json::Value;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::PublicKey;
use sodiumoxide::crypto::box_::SecretKey;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex, MutexGuard},
    thread,
};
pub struct serverHooks {}

impl serverHooks {
    pub fn trackFreeIds(party_count: u64) -> intervals {
        return intervals_fn(1, party_count);
    }

    pub fn generateKeyPair(sodium: bool) -> (Option<PublicKey>, Option<SecretKey>) {
        //println!("in generateKeyPair 1");
        if sodium {
            let (pub_key, sec_key) = box_::gen_keypair();
            return (Some(pub_key), Some(sec_key));
        } else {
            return (None, None);
        }
    }

    pub fn parseKey(sodium: bool, keyString: &Value) -> Option<Vec<u8>> {
        if sodium {
            let array: Vec<u8> = serde_json::from_str(keyString.as_str().unwrap()).unwrap();
            return Some(array);
        } else {
            return None;
        }
    }

    pub fn dumpKey(sodium: bool, key: &Value) -> String {
        if sodium {
            return key.to_string();
        } else {
            String::new()
        }
    }

    pub fn beforeInitialization(
        riff: MutexGuard<restfulAPI>,
        computation_id: Value,
        params: Value,
    ) {
        let party_count = &params["party_count"];
        // validate party_count
        if *party_count == Value::Null {
            // no party count given or saved.
            panic!("party count is not specified nor pre-saved");
        } else if party_count.as_u64().unwrap() < 1 {
            // Too small
            panic!("party count is less than 1");
        } else if riff.computationMaps.maxCount[computation_id.to_string()] != Value::Null
            && *party_count != riff.computationMaps.maxCount[computation_id.to_string()]
        {
            // contradicting values
            panic!("contradicting party count");
        }

        // validate party_id
        let party_id = &params["party_id"];
        if *party_id != Value::Null {
            // party_id is given, check validity
            if *party_id != "s1" {
                if !party_id.is_number()
                    || party_id.as_u64().unwrap() <= 0
                    || party_id.as_u64().unwrap() > party_count.as_u64().unwrap()
                {
                    panic!("Invalid party ID: not a valid number");
                }
            }
        } else {
            // party_id is null, must generate a new free id, if the computation is full we have a problem!
            if riff.computationMaps.clientIds[computation_id.to_string()] != Value::Null
                && riff.computationMaps.clientIds[computation_id.to_string()]
                    .as_array()
                    .unwrap()
                    .len()
                    == riff.computationMaps.maxCount[computation_id.to_string()]
            {
                panic!("Maximum parties capacity reached");
            }
        }

        // All is good
    }
}
