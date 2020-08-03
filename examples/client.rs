
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


fn main() {

        let mut options = HashMap::new();
        options.insert(String::from("sodium"), JsonEnum::Bool(true));
        let my_client = RiffClientRest::new(String::from("http://127.0.0.1:8080"), String::from("test1"), options);
        let client_access = Arc::new(Mutex::new(my_client));
        RiffClientRest::connect(client_access, true);
        thread::sleep(Duration::from_secs(100));
        /*
         *let shares = riffClientRest::share(clientAccess, input_value);
         *let result = riffClientRest::open(clientAccess, shares[1]);
         */


}
