use crate::server::datastructure::intervals::*;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex},
    thread,
};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::PublicKey;
use sodiumoxide::crypto::box_::SecretKey;
use crate::server::restfulAPI::restfulAPI;
use serde_json::Value;
pub struct serverHooks {

}

impl serverHooks {
    pub fn trackFreeIds (party_count: u64) -> intervals {
        return intervals_fn(1, party_count)
    }

    pub fn generateKeyPair (instance: Arc<Mutex<restfulAPI>>) -> (Option<PublicKey>, Option<SecretKey>){
        if instance.lock().unwrap().sodium {
            let (pub_key, sec_key) = box_::gen_keypair();
            return (Some(pub_key), Some(sec_key))
        } else {
            return (None, None)
        }
    }

    pub fn parseKey (instance: Arc<Mutex<restfulAPI>>, keyString: &Value) -> Option<Vec<u8>> {
        if instance.lock().unwrap().sodium {
            let array: Vec<u8> = serde_json::from_str(keyString.as_str().unwrap()).unwrap();
            return Some(array)
        } else {
            return None
        }
    }

    pub fn dumpKey (instance: Arc<Mutex<restfulAPI>>, key: &Vec<u8>) -> String{
        if instance.lock().unwrap().sodium {
            return  format!("{:?}", key);
        } else {
            String::new()
        }
    }
}