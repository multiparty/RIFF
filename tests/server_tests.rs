use riff::server::{Server, restfulAPI, hooks, maps};


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
//#[tokio::test]
fn open_restfulAPI() {
    let c_map = restfulAPI::computationMaps {clientIds:HashMap::new(), maxCount:HashMap::new(), keys:HashMap::new(), secretKeys:HashMap::new(), freeParties:HashMap::new(), spareIds:HashMap::new()};
    let serverHooks = hooks::serverHooks {};
    let maps = maps {tags: HashMap::new(), pendingMessages: HashMap::new()};
    let  restfulAPI_instance = restfulAPI::restfulAPI{ mail_box: HashMap::new(), computationMaps: c_map, hooks: serverHooks, maps:maps};
    
    //s.on();s
    //restfulAPI.on();
    restfulAPI::restfulAPI::on(Arc::new(Mutex::new(restfulAPI_instance)));
}


