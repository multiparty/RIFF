
use futures::Future;
use crate::common::helper;
use serde_json::json;
use std::cmp;
use crate::server::restfulAPI::*;
use serde_json::Value;

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
use crate::preprocessing::api;
use crate::api::crypto_provider;



pub enum NumberOrFuture {
     Number(i64),
     Future(Box<dyn Future<Output = i64>>),
 }
 #[derive(Debug, Clone)]
pub struct SecretShare {
    pub holders: Vec<i64>,
    pub threshold: i64,
    pub Zp: i64,
    pub value: i64,
    
}

impl SecretShare {
    pub fn new (value: i64, holders: Vec<i64>, threshold: i64, Zp: i64) -> SecretShare {
        // sort holders
        //jiff.helpers.sort_ids(holders);
        SecretShare {
            value: value,
            holders: holders,
            threshold: threshold,
            Zp: Zp,
        }
    }

    pub fn refresh (&self, riff: Arc<Mutex<RiffClientRest>>, options: HashMap<String, JsonEnum>) -> SecretShare {
        let mut instance = riff.lock().unwrap();
        let mut op_id = String::new();
        if let Some(data) = options.get(&String::from("op_id")) {
            if let JsonEnum::String(str) = data {
                op_id = str.clone();
            }
        } else {
            std::mem::drop(instance);
            op_id = counters::gen_op_id(riff.clone(), String::from("refresh"), self.holders.clone());
            instance = riff.lock().unwrap();
        }

        // get shares of zero
        std::mem::drop(instance);
        let zero = api::get_preprocessing(riff.clone(), op_id.clone());
        instance = riff.lock().unwrap();

        let mut result_value:i64 = 0;
        if let Some(zero_l) = zero {
            result_value = self.sadd(zero_l).value;

        } else {
            let mut options = HashMap::new();
            options.insert(String::from("receivers_list"), JsonEnum::Array(self.holders.clone()));
            options.insert(String::from("threshold"), JsonEnum::Number(self.threshold) );
            options.insert(String::from("Zp"), JsonEnum::Number(self.Zp));
            options.insert(String::from("op_id"), JsonEnum::String(op_id.clone()));
            options.insert(String::from("params"), JsonEnum::Value(json!({
                "number": 0,
                "count": 1,
            })));
            std::mem::drop(instance);
            crypto_provider::from_crypto_provider(riff.clone(), String::from("numbers"), options);
            instance = riff.lock().unwrap();

            std::mem::drop(instance);
            loop {
                instance = riff.lock().unwrap();
                
                if let Some(msg) = instance.crypto_map.get(&op_id) {
                    let data = msg.get(&String::from("shares")).unwrap();
                    if let JsonEnum::ArrayShare(shares) = data {
                        result_value = self.sadd(shares[0].clone()).value;
                    }
                    break;
                }
                //println!("in loop");
                std::mem::drop(instance);
                thread::sleep(Duration::from_secs(1));
            }
           
        }
        SecretShare {
            value: result_value,
            holders: self.holders.clone(),
            threshold: self.threshold,
            Zp: self.Zp,
        }

    }
}