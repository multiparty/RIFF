use riff::ext::RiffClientRestful::RiffClientRest;
//use riff::ext::RiffClientTrait::*;
use rand::prelude::*;
use riff::client::RiffClientTrait::*;
use riff::server::{hooks, maps, restfulAPI, Server};
use riff::RiffClient::JsonEnum;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::PublicKey;
use sodiumoxide::crypto::box_::SecretKey;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

// use serde_json::json;
use rand::Rng;
use riff::SecretShare::SecretShare;
use serde_json::json;
mod arithemetic_test;

#[test]
pub fn test_suite() {
    let all_tests = vec![String::from("smult")];//String::from("sadd"), 

    let mut config = HashMap::new();
    config.insert(String::from("sadd"), HashMap::new());
    config
        .get_mut(&String::from("sadd"))
        .unwrap()
        .insert(String::from("party_count"), String::from("3"));
    config
        .get_mut(&String::from("sadd"))
        .unwrap()
        .insert(String::from("number_of_tests"), String::from("10"));
    config.insert(String::from("smult"), HashMap::new());
    config
        .get_mut(&String::from("smult"))
        .unwrap()
        .insert(String::from("party_count"), String::from("3"));
    config
        .get_mut(&String::from("smult"))
        .unwrap()
        .insert(String::from("number_of_tests"), String::from("10"));
    thread::spawn(|| SERVER_restAPI());
    let mut rng = rand::thread_rng();

    for test in all_tests {
        let number_of_tests: i64 = config
            .get(&test)
            .unwrap()
            .get(&String::from("number_of_tests"))
            .unwrap()
            .parse()
            .unwrap();
        for t in 1..number_of_tests + 1 {
            let party_count: i64 = config
                .get(&test)
                .unwrap()
                .get(&String::from("party_count"))
                .unwrap()
                .parse()
                .unwrap();
            let mut inputs: Vec<i64> = vec![];
            for _ in 1..party_count + 1 {
                inputs.push(rng.gen_range(0, 16777729));
            }
            
            arithemetic_test::one_test(inputs, t, test.clone());
           
            

            // let mut result = first_input.checked_add(second_input);
            // if let Some(i64result) = result {
            //     result =i64result.checked_add(thrid_input);
            //     if let Some(i64result) = result {
            //         correct_result = i64result;
            //     } else {

            //     }
            // } else {

            // }
        }
    }
}

fn SERVER_restAPI() {
    let c_map = restfulAPI::computationMaps {
        clientIds: json!({}),
        maxCount: json!({}),
        keys: json!({}),
        secretKeys: json!({}),
        freeParties: json!({}),
        spareIds: HashMap::new(),
    };
    let serverHooks = hooks::serverHooks {};
    let maps = maps {
        tags: json!({}),
        pendingMessages: json!({}),
    };
    let restfulAPI_instance = restfulAPI::restfulAPI {
        mail_box: json!({}),
        computationMaps: c_map,
        hooks: serverHooks,
        maps: maps,
        sodium: true,
        log: false,
        cryptoMap: json!({}),
    };

    //s.on();s
    //restfulAPI.on();
    restfulAPI::restfulAPI::on(Arc::new(Mutex::new(restfulAPI_instance)));
}
