use riff::server::{Server, restfulAPI, hooks, maps};


use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
};

use sodiumoxide::crypto::box_::PublicKey;
use sodiumoxide::crypto::box_::SecretKey;
use sodiumoxide::crypto::box_;

use serde_json::json;



#[test]
fn open_websocket() {
    let mut s = Server{ name: String::from("test_server"), mail_box: HashMap::new()};
    let this = Arc::new(Mutex::new(s));
    //s.on();
    Server::on(this);
}

#[test]
fn sodiumTest() {
    let (ourpk, oursk) = box_::gen_keypair();
    println!("{:?}", ourpk);
}

#[test]
//#[tokio::test]
fn open_restfulAPI() {
    let c_map = restfulAPI::computationMaps {clientIds:json!({}), maxCount:json!({}), keys:json!({}), secretKeys:json!({}), freeParties:json!({}), spareIds:HashMap::new()};
    let serverHooks = hooks::serverHooks {};
    let maps = maps {tags: json!({}), pendingMessages: json!({})};
    let restfulAPI_instance = restfulAPI::restfulAPI{ mail_box: json!({}), computationMaps: c_map, hooks: serverHooks, maps:maps, sodium: true, log: false};
    
    //s.on();s
    //restfulAPI.on();
    restfulAPI::restfulAPI::on(Arc::new(Mutex::new(restfulAPI_instance)));
}


