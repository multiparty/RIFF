use std::{
    cmp,
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex, MutexGuard},
    thread,
};
 use crate::RiffClient::*;
 use crate::client::util::constants;
use primes;
use serde_json::json;
use serde_json::Value;
use crate::ext::RiffClientRest;
use crate::SecretShare::SecretShare;
pub trait RiffClientTrait {
    fn new (hostname: String,
        computation_id: String,
        options: HashMap<String, JsonEnum>) -> Self ;
    
    fn connect (riff: Arc<Mutex<RiffClientRest>>, immediate: bool);

    fn emit (riff: Arc<Mutex<RiffClientRest>>, label: String, msg: String);

    fn disconnect(riff: Arc<Mutex<RiffClientRest>>);

    fn is_empty(riff: Arc<Mutex<RiffClientRest>>) -> bool;

    fn share(riff: Arc<Mutex<RiffClientRest>>, secret: i64, options: HashMap<String, JsonEnum>) -> Vec<SecretShare>;

    fn open(riff: Arc<Mutex<RiffClientRest>>, share: SecretShare, options: HashMap<String, JsonEnum>) -> Option<i64>;


}