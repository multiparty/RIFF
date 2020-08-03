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
pub trait RiffClientTrait {
    fn new (hostname: String,
        computation_id: String,
        options: HashMap<String, JsonEnum>) -> Self ;
    
    fn connect (riff: Arc<Mutex<RiffClientRest>>, immediate: bool);

    fn emit (riff: Arc<Mutex<RiffClientRest>>, label: String, msg: String);

    fn disconnect();

    fn is_empty(&mut self) -> bool;




}