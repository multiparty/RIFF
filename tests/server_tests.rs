use riff::server::{Server, restfulAPI};


use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
};

#[test]
fn open_websocket() {
    let mut s = Server{ name: String::from("test_server"), mail_box: HashMap::new()};
    let this = Arc::new(Mutex::new(s));
    //s.on();
    Server::on(this);
}

#[test]
fn open_restfulAPI() {
    let c_map = restfulAPI::computationMaps {clientIds:HashMap::new(), maxCount:HashMap::new(), keys:HashMap::new(), secretKeys:HashMap::new(), freeParties:HashMap::new()};
    let  restfulAPI = restfulAPI::restfulAPI{ mail_box: HashMap::new(), computationMaps: c_map };
    
    //s.on();
    //restfulAPI.on();
    restfulAPI::on(Arc::new(Mutex::new(restfulAPI)));
}


