
//use riff::server::{Server, restfulAPI, hooks, maps};
use riff::ext::RiffClientRestful::{RiffClientRest};
use riff::client::RiffClientTrait::*;
use riff::RiffClient::JsonEnum;


use std::{
    collections::HashMap,
    // env,
    // io::Error as IoError,
    // net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

// use sodiumoxide::crypto::box_::PublicKey;
// use sodiumoxide::crypto::box_::SecretKey;
// use sodiumoxide::crypto::box_;

// use serde_json::json;
use riff::SecretShare::SecretShare;

fn main() {

        let mut options = HashMap::new();
        options.insert(String::from("sodium"), JsonEnum::Bool(true));
        options.insert(String::from("crypto_provider"), JsonEnum::Bool(true));
        let my_client = RiffClientRest::new(String::from("http://127.0.0.1:8080"), String::from("test1"), options);
        let client_access = Arc::new(Mutex::new(my_client));
        RiffClientRest::connect(client_access.clone(), true);
        thread::sleep(Duration::from_secs(3));
        let mut options_share = HashMap::new();
        let shares: Vec<SecretShare> = RiffClientRest::share(client_access.clone(), 10, options_share);
        // for sc in shares {
        //     println!("{:?}",sc);
        // }

        //compute sadd
        let mut sum = shares[1].clone();
        let mut clinet_instance = client_access.lock().unwrap();
        for i in 2..clinet_instance.party_count + 1 {
            sum = sum.sadd(shares[i as usize].clone())
        }
        std::mem::drop(clinet_instance);
        let options_open = HashMap::new();
        let result = RiffClientRest::open(client_access.clone(), sum, options_open);
        clinet_instance = client_access.lock().unwrap();
        println!("result: {}", result.unwrap());
        thread::sleep(Duration::from_secs(100));
        /*
         *let shares = riffClientRest::share(clientAccess, input_value);
         *let result = riffClientRest::open(clientAccess, shares[1]);
         */


}
