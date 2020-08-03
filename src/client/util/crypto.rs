use crate::server::datastructure::intervals::*;
use crate::server::restfulAPI::restfulAPI;
use serde_json::Value;
use serde_json::json;
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

use crate::RiffClientRestful::RiffClientRest;

pub fn decrypt_and_sign (riff: Arc<Mutex<RiffClientRest>>, msg: Value, secret_key: Value, signing_public_key: Value) -> Value {
    let nonce: Vec<u8> = serde_json::from_str(msg["nonce"].as_str().unwrap()).unwrap();
    //let nonce = msg["nonce"].as_array().unwrap().to_owned();
    let mut Nonce = [0; 24];
    Nonce.copy_from_slice(nonce.as_slice());
    let Nonce = box_::Nonce(Nonce);

    let cipher_text = msg["cipher"].clone();
    let cipher_text:Vec<u8> = serde_json::from_str(cipher_text.as_str().unwrap()).unwrap();

    let public_key: Vec<u8> = serde_json::from_str(signing_public_key.as_str().unwrap()).unwrap();
    //let nonce = msg["nonce"].as_array().unwrap().to_owned();
    let mut Public_key = [0; 32];
    Public_key.copy_from_slice(public_key.as_slice());
    let Public_key = box_::PublicKey(Public_key);

    let secret_key: Vec<u8> = serde_json::from_str(secret_key.as_str().unwrap()).unwrap();
    //let nonce = msg["nonce"].as_array().unwrap().to_owned();
    let mut Secret_key = [0; 32];
    Secret_key.copy_from_slice(secret_key.as_slice());
    let Secret_key = box_::SecretKey(Secret_key);

    return json!(box_::open(&cipher_text, &Nonce, &Public_key, &Secret_key).unwrap());
}

pub fn encrypt_and_sign (msg: Value, encryption_public_key: Value, signing_private_key: Value) -> Value{
    let nonce = box_::gen_nonce();
    let public_key: Vec<u8> = serde_json::from_str(encryption_public_key.as_str().unwrap()).unwrap();
    let mut Public_key = [0; 32];
    Public_key.copy_from_slice(public_key.as_slice());
    let Public_key = box_::PublicKey(Public_key);

    let secret_key: Vec<u8> = serde_json::from_str(signing_private_key.as_str().unwrap()).unwrap();
    //let nonce = msg["nonce"].as_array().unwrap().to_owned();
    let mut Secret_key = [0; 32];
    Secret_key.copy_from_slice(secret_key.as_slice());
    let Secret_key = box_::SecretKey(Secret_key);
    let cipher = box_::seal(&msg.as_i64().unwrap().to_be_bytes(), &nonce, &Public_key, &Secret_key);

    let nonce_string = format!("{:?}", nonce.0);
    let cipher_string = format!("{:?}", cipher);

    return json!({
        "nonce": json!(nonce_string),
        "cipher": json!(cipher_string),
    })
}