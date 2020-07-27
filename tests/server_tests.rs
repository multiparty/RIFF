use riff::server::{Server, restfulAPI, hooks, maps};
use riff::ext::RiffClientRestful::{riffClientRest};
use riff::ext::RiffClientTrait::*;


use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use sodiumoxide::crypto::box_::PublicKey;
use sodiumoxide::crypto::box_::SecretKey;
use sodiumoxide::crypto::box_;

use serde_json::json;




//#[tokio::test]
#[test]
fn SERVER_restAPI() {
    let c_map = restfulAPI::computationMaps {clientIds:json!({}), maxCount:json!({}), keys:json!({}), secretKeys:json!({}), freeParties:json!({}), spareIds:HashMap::new()};
    let serverHooks = hooks::serverHooks {};
    let maps = maps {tags: json!({}), pendingMessages: json!({})};
    let restfulAPI_instance = restfulAPI::restfulAPI{ mail_box: json!({}), computationMaps: c_map, hooks: serverHooks, maps:maps, sodium: true, log: false, cryptoMap: json!({})};

    //s.on();s
    //restfulAPI.on();
    restfulAPI::restfulAPI::on(Arc::new(Mutex::new(restfulAPI_instance)));
}


/*
 *#[test]
 *fn test_server_and_clients() {
 *    let c_map = restfulAPI::computationMaps {clientIds:json!({}), maxCount:json!({}), keys:json!({}), secretKeys:json!({}), freeParties:json!({}), spareIds:HashMap::new()};
 *    let serverHooks = hooks::serverHooks {};
 *    let maps = maps {tags: json!({}), pendingMessages: json!({})};
 *    let restfulAPI_instance = restfulAPI::restfulAPI{ mail_box: json!({}), computationMaps: c_map, hooks: serverHooks, maps:maps, sodium: true, log: false, cryptoMap: json!({})};
 *
 *    thread::spawn(move || {
 *                  restfulAPI::restfulAPI::on(Arc::new(Mutex::new(restfulAPI_instance)));
 *    }
 *    );
 *    thread::spawn(move || {
 *
 *                  let options = HashMap::new();
 *                  let myClient = riffClientRest::new(String::from("127.0.0.1:8080"), String::from("test1"), options);
 *                  let clientAccess = Arc::new(Mutex::new(myClient));
 *                  riffClientRest::connect(clientAccess, true);
 *                  let shares = riffClientRest::share(clientAccess, input_value);
 *                  let result = riffClientRest::open(clientAccess, shares[1]);
 *
 *    }
 *    );
 *
 *    thread::sleep(Duration::from_millis(5000));
 *}
 */
