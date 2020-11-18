use crate::architecture::counters;
use crate::common::helper;
use crate::ext::RiffClientRest;
use crate::server::restfulAPI::*;
use crate::RiffClient::JsonEnum;
use crate::SecretShare::SecretShare;
use crate::{architecture::hook, RiffClientTrait::RiffClientTrait};
use serde_json::json;
use serde_json::Value;
use std::cmp;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};
// use futures::{} TODO
use crate::util::helpers as uhelper;
use crate::preprocessing::api;
use crate::api::crypto_provider;
use crate::shamir::open;


impl SecretShare {

    pub fn cadd(&self, cst: i64) -> SecretShare {
        let sum = self.value + cst;
        let afterMod = helper::modF(json!(sum), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: self.threshold,
            Zp: self.Zp,
        }
    }

    pub fn csub(&self, cst: i64) -> SecretShare {
        let sum = self.value - cst;
        let afterMod = helper::modF(json!(sum), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: self.threshold,
            Zp: self.Zp,
        }
    }

    pub fn cmult(&self, cst: i64) -> SecretShare {
        let product = self.value * cst;
        let afterMod = helper::modF(json!(product), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: self.threshold,
            Zp: self.Zp,
        }
    }

    pub fn cdivfac(&self, cst: i64) -> SecretShare {
        let inv = uhelper::extended_gcd(cst, self.Zp).0;
        let product = self.value * inv;
        let afterMod = helper::modF(json!(product), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: self.threshold,
            Zp: self.Zp,
        }
    }








    pub fn sadd(&self, o: SecretShare) -> SecretShare {
        if !self.Zp == o.Zp {
            panic!("shares must belong to the same field (+)");
        }

        if !(self.holders == o.holders) {
            panic!("shares must be held by the same parties (+)");
        }

        let sum = self.value + o.value;
        let afterMod = helper::modF(json!(sum), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: cmp::max(self.threshold, o.threshold),
            Zp: self.Zp,
        }
    }

    pub fn ssub(&self, o: SecretShare) -> SecretShare {
        if !self.Zp == o.Zp {
            panic!("shares must belong to the same field (+)");
        }

        if !(self.holders == o.holders) {
            panic!("shares must be held by the same parties (+)");
        }

        let sum = self.value - o.value;
        let afterMod = helper::modF(json!(sum), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: cmp::max(self.threshold, o.threshold),
            Zp: self.Zp,
        }
    }


   //Multiplication of two secret shares through Beaver Triplets.
    pub fn smult(&self, o: SecretShare,mut op_id: Option<String>, riff: Arc<Mutex<RiffClientRest>>) -> SecretShare{
        if !self.Zp == o.Zp {
            panic!("shares must belong to the same field (*)");
        }

        if !(self.holders == o.holders) {
            panic!("shares must be held by the same parties (*)");
        }

        if op_id == Option::None {
            op_id = Some(counters::gen_op_id(riff.clone(), String::from("smult"), self.holders.clone()));
        }


        let triplet = api::get_preprocessing(riff.clone(), String::from(":triplet"));
        let mut triplet_shares= vec![];


        if let None = triplet {
            let mut options = HashMap::new();
            options.insert(String::from("receivers_list"), JsonEnum::Array(self.holders.clone()));
            options.insert(String::from("threshold"), JsonEnum::Number(cmp::max(self.threshold, o.threshold)));
            options.insert(String::from("Zp"), JsonEnum::Number(self.Zp));
            let mut op_id_options = op_id.clone().unwrap();
            op_id_options.push_str(":triplet");
            options.insert(String::from("op_id"), JsonEnum::String(op_id_options.clone()));
            crypto_provider::from_crypto_provider(riff.clone(), String::from("triplet"), options);

            //TODO: async/await implementation - await the incoming shares without blocking
            //      should be able to avoid locking and unlocking the RIFF Instance frequently
            loop {
                let instance = riff.lock().unwrap();

                if let Some(msg) = instance.crypto_map.get(&op_id_options) {
                    let data = msg.get(&String::from("shares")).unwrap();
                    if let JsonEnum::ArrayShare(shares) = data {
                        triplet_shares = shares.clone();
                    }
                    break;
                }
                std::mem::drop(instance);
                thread::sleep(Duration::from_secs(1));
            }


        } else {
            //to-do: client side preprocessing

        }

        let a = triplet_shares[0].clone();
        let b = triplet_shares[1].clone();
        let c = triplet_shares[2].clone();

        // d = s - a. e = o - b.
        let d = self.sadd(a.cmult(-1));
        let e = o.sadd(b.cmult(-1));

        // Open d and e.
        // The only communication cost.
        let mut options_1 = HashMap::new();
        let mut op_id_options = op_id.clone().unwrap();
        op_id_options.push_str(":open1");
        println!("op_id_1: {}", op_id_options);
        options_1.insert(String::from("parties"), JsonEnum::Array(e.holders.clone()));
        options_1.insert(String::from("op_id"), JsonEnum::String(op_id_options));

        let e_value = open::riff_open(riff.clone(), e, options_1).unwrap();
        println!("after open");
        let mut options_2 = HashMap::new();
        let mut op_id_options = op_id.clone().unwrap();
        op_id_options.push_str(":open2");
        println!("op_id_2: {}", op_id_options);
        options_2.insert(String::from("parties"), JsonEnum::Array(d.holders.clone()));
        options_2.insert(String::from("op_id"), JsonEnum::String(op_id_options));
        let d_value = open::riff_open(riff.clone(), d, options_2).unwrap();

        // result_share = d_open * e_open + d_open * b_share + e_open * a_share + c.
        let t1 = helper::modF(json!(d_value * e_value), json!(self.Zp));
        let t2 = b.cmult(d_value);
        let t3 = a.cmult(e_value);

        // All this happens locally.
        let mut final_result = t2.cadd(t1);
        final_result = final_result.sadd(t3);
        final_result = final_result.sadd(c);



        return final_result;


    }

    pub fn smult_bgw(&self, o: SecretShare,mut op_id: Option<String>, riff: Arc<Mutex<RiffClientRest>>) -> SecretShare {
        if !self.Zp == o.Zp {
            panic!("shares must belong to the same field (bgw*)");
        }

        if !(self.holders == o.holders) {
            panic!("shares must be held by the same parties (bgw*)");
        }

        if ((self.threshold - 1) + (o.threshold - 1)) > (self.holders.len() as i64 - 1) {
            panic!("threshold too high for BGW (*)");
        }

        if op_id == Option::None {
            op_id = Some(counters::gen_op_id(riff.clone(), String::from("smult_bgw"), self.holders.clone()));
        }

        // ensure thresholds are fine
        let new_threshold = (self.threshold - 1) + (o.threshold - 1) + 1;
        //if new_threshold > self.holders
        // if (new_threshold > this.holders) {
        //     var errorMsg = 'Threshold too large for smult_bgw: ' + new_threshold;
        //     errorMsg += '. Shares: ' + this.toString() + ', ' + o.toString();
        //     throw new Error(errorMsg);
        //   }

        // multiply via the BGW protocol
        let result_value = helper::modF(json!(self.value * o.value), json!(self.Zp));
        let result = SecretShare::new(result_value, self.holders, new_threshold, self.Zp);

    }


}
