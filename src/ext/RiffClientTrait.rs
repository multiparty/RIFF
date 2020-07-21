use std::{
    cmp,
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex},
    thread,
};
 use crate::RiffClient::*;
 use crate::client::util::constants;
use primes;
use serde_json::json;
use serde_json::Value;
pub trait RiffClientTrait {
    fn new (hostname: String,
        computation_id: String,
        options: HashMap<String, JsonEnum>) -> Self ;
    
    fn connect ();

    fn emit ();

    fn disconnect();

    fn is_empty();




}