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
    let restfulAPI = restfulAPI::restfulAPI{ mail_box: HashMap::new()};
    
    //s.on();
    restfulAPI.on();
}


