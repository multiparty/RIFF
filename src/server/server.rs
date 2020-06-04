use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::server::accept;
use tungstenite::{connect, Message, accept_hdr};
use tungstenite::handshake::server::{Request, Response};

use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
};


pub struct SocketMap {
    socket_ids: HashMap<u32, HashMap<u32, SocketAddr>>,
    computation_ids: HashMap<SocketAddr, u32>,
    party_ids: HashMap<SocketAddr, u32>,
}

pub struct Server {
    pub name: String,
    socketMap: SocketMap,
}

impl Server {
    pub fn on(&self) {
        let server = TcpListener::bind("127.0.0.1:9001").unwrap();
        for stream in server.incoming() {
            spawn(move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                loop {
                    let msg = websocket.read_message().unwrap();
                    println!("Received: {}", msg);

                    // We do not want to send back ping/pong messages.
                    if msg.is_binary() || msg.is_text() {
                        websocket
                            .write_message(Message::Text("Server message".into()))
                            .unwrap();
                        //websocket.write_message(msg).unwrap();
                    }
                }
            });
        }
    }
}
