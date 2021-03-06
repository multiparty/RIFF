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
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

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
        log: true,
        cryptoMap: json!({}),
    };

    //s.on();s
    //restfulAPI.on();
    restfulAPI::restfulAPI::on(Arc::new(Mutex::new(restfulAPI_instance)));
}

pub fn start_client(input: i64, times: i64, test_name: String) -> i64 {
    //let args: Vec<String> = env::args().collect();
    //sodiumoxide::init().unwrap();
    let mut options = HashMap::new();
    options.insert(String::from("sodium"), JsonEnum::Bool(true));
    options.insert(String::from("crypto_provider"), JsonEnum::Bool(true));
    options.insert(String::from("party_count"), JsonEnum::Number(3));

    //options.insert(String::from("onConnect"), JsonEnum::func(callback_computation));
    let mut computation_id = test_name.clone();
    computation_id.push_str(times.to_string().as_str());
    let my_client = RiffClientRest::new(
        String::from("http://127.0.0.1:8080"),
        computation_id,
        options,
    );
    let client_access = Arc::new(Mutex::new(my_client));
    RiffClientRest::connect(client_access.clone(), true);
    //thread::sleep(Duration::from_secs(7));
    let mut options_share = HashMap::new();
    //let secret: i64 = args[1].parse().unwrap();
    let shares: Vec<SecretShare> =
        RiffClientRest::share(client_access.clone(), input, options_share);
    // for sc in shares {
    //     println!("{:?}",sc);
    // }

    //compute sadd
    let test_str = test_name.as_str();
    let mut res = shares[1].clone();
    match test_str {
        "sadd" => {
            let mut clinet_instance = client_access.lock().unwrap();
            for i in 2..clinet_instance.party_count + 1 {
                res = res.sadd(shares[i as usize].clone());
            }
        },
        "smult" => {
            let mut clinet_instance = client_access.lock().unwrap();
            for i in 2..clinet_instance.party_count + 1 {
                std::mem::drop(clinet_instance);
                res = res.smult(shares[i as usize].clone(), None, client_access.clone());
                clinet_instance = client_access.lock().unwrap();
            }
        },
        _ => (),
    }

    //cadd
    // let mut sum = shares[1].clone();
    // let mut clinet_instance = client_access.lock().unwrap();
    // for i in 2..clinet_instance.party_count + 1 {

    // }

    //std::mem::drop(clinet_instance);
    let options_open = HashMap::new();
    let result = RiffClientRest::open(client_access.clone(), res, options_open);
    //clinet_instance = client_access.lock().unwrap();

    RiffClientRest::disconnect(client_access.clone());
    return result.unwrap();
    //thread::sleep(Duration::from_secs(100));
    /*
     *let shares = riffClientRest::share(clientAccess, input_value);
     *let result = riffClientRest::open(clientAccess, shares[1]);
     */
}

//#[test]
pub fn one_test(inputs: Vec<i64>, times: i64, test_name: String) {
    //println!("input: {:?}", inputs);
    let mut correct_answer = 1;
    let test_name_str = test_name.as_str();
    match test_name_str {
        "sadd" => {
            correct_answer -= 1;
            for input in inputs.clone() {
                correct_answer += input;
                correct_answer = correct_answer % 16777729;
            }
        }
        "smult" => {
            for input in inputs.clone() {
                correct_answer *= input;
                correct_answer = correct_answer % 16777729;
            }
        }
        _ => ()
    }
    
    correct_answer = correct_answer % 16777729;
    let (tx, rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
    let mut children = Vec::new();
    for input in inputs.clone() {
        let thread_tx = tx.clone();
        let test_name = test_name.clone();
        let child = thread::spawn(move || {
            thread_tx
                .send(start_client(input, times, test_name))
                .unwrap();
        });
        children.push(child);
    }
    let mut results = Vec::with_capacity(inputs.len() as usize);
    for _ in 0..inputs.len() {
        // The `recv` method picks a message from the channel
        // `recv` will block the current thread if there are no messages available
        results.push(rx.recv());
    }
    // Wait for the threads to complete any remaining work
    // for child in children {
    //     child.join().expect("oops! the child thread panicked");
    // }
    let result_from_test = results.pop().unwrap().unwrap();
    for result in results {
        assert_eq!(result_from_test, result.unwrap());
    }
    assert_eq!(result_from_test, correct_answer, "inputs: {:?}", inputs);
    println!("{} test succeeded: {}", test_name ,times);
}
