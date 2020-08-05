use crate::server::restfulAPI::*;
use serde_json::Value;
use serde_json::json;
use crate::common::helper;
use std::{
    time::Duration,
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex, MutexGuard},
    thread,
};
use crate::ext::RiffClientRest;
use crate::RiffClient::JsonEnum;
use crate::architecture::counters;
use crate::{RiffClientTrait::RiffClientTrait, architecture::hook};
use crate::SecretShare::SecretShare;

pub fn get_preprocessing (riff: Arc<Mutex<RiffClientRest>>,op_id: String) -> Option<SecretShare> {
    let instance = riff.lock().unwrap();
    let values = instance.preprocessing_table.get(&op_id.to_string());
    if let Some(ss) = values {
        return Some(ss.clone())
    }
    if instance.crypto_provider == true {
        return None
    }
    panic!("No preprocessed value(s) that correspond to the op_id ")
}