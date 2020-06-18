use riff::server;

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
    let mut s = server::Server{ name: String::from("test_server"), mail_box: HashMap::new()};
    let this = Arc::new(Mutex::new(s));
    //s.on();
    server::Server::on(this);
}
