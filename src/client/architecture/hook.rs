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
use crate::RiffClientRestful::*;
pub fn generateKeyPair(riff: Arc<Mutex<riffClientRest>>) -> (Option<PublicKey>, Option<SecretKey>) {
    //println!("in generateKeyPair 1");
    if riff.lock().unwrap().sodium_ != false {
        let (pub_key, sec_key) = box_::gen_keypair();
        return (Some(pub_key), Some(sec_key));
    } else {
        return (None, None);
    }
}

pub fn dumpKey (riff: Arc<Mutex<riffClientRest>>) -> String {
    let riff = riff.lock().unwrap();
    if riff.sodium_ != false {
        return riff.public_key.clone().to_string()
    } else {
        return String::new()
    }
}

pub fn parseKey(riff: Arc<Mutex<riffClientRest>>, keyString: &Value) -> Option<Vec<u8>> {
    let instance = riff.lock().unwrap();
    if instance.sodium_ != false {
        let array: Vec<u8> = serde_json::from_str(keyString.as_str().unwrap()).unwrap();
        return Some(array);
    } else {
        return None;
    }
} 