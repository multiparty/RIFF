
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
fn main() {

        let options = HashMap::new();
        let myClient = riffClientRest::new(String::from("127.0.0.1:8080"), String::from("test1"), options);
        let clientAccess = Arc::new(Mutex::new(myClient));
        riffClientRest::connect(clientAccess, true);
        /*
         *let shares = riffClientRest::share(clientAccess, input_value);
         *let result = riffClientRest::open(clientAccess, shares[1]);
         */


}
