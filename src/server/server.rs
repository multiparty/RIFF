use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::RwLock;
use std::thread::spawn;
use std::time::Duration;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::protocol::WebSocket;
use tungstenite::server::accept;
use tungstenite::{accept_hdr, connect, Message};

use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
};

pub struct SocketMap {
    socket_ids: HashMap<u32, HashMap<u32, SocketAddr>>,
    computation_ids: HashMap<SocketAddr, u32>,
    party_ids: HashMap<SocketAddr, u32>,
}

pub struct Server {
    pub name: String,
    //socketMap: SocketMap,
}

impl Server {
    pub fn on(&self) {
        let server = TcpListener::bind("127.0.0.1:9001").unwrap();
        //let shared_message = Arc::new(Mutex::new(String::from("")));
        let websockets_hashmap = Arc::new(RwLock::new(HashMap::new()));
        let counter = Arc::new(Mutex::new(0));

        for stream in server.incoming() {
            //let shared_message = Arc::clone(&shared_message);
            let websockets_hashmap = Arc::clone(&websockets_hashmap);
            let counter = Arc::clone(&counter);

            spawn(move || {
                println!("new thread!");
                let id;
                {
                    let mut num = counter.lock().unwrap();
                    *num += 1;
                    println!("Received: {}", &num);
                    id = num.clone();
                }
                let websocket = accept(stream.unwrap()).unwrap();

                {
                    websockets_hashmap.write().unwrap().insert(id, websocket);
                    println!("{:?}", websockets_hashmap);
                }

                let mut planner = periodic::Planner::new();
                planner.add(
                    move || {//let cur_websocket;//:
                        let mut hashmap = websockets_hashmap.write().unwrap();
                        let cur_websocket: &mut tungstenite::protocol::WebSocket<std::net::TcpStream> =
                            hashmap.get_mut(&id).unwrap();
                        let msg = cur_websocket.read_message().unwrap();
    
                        println!("Received: {}", msg);
                        let cur_message = msg.to_string();
    
                        // let mut message = shared_message.lock().unwrap();
                        // *message = String::from("");
                        // *message += msg.to_text().unwrap();
    
                        let broadcast_recipients = &mut hashmap.iter_mut().map(|(_, socket)| socket);
                        for recp in broadcast_recipients {
                            //recp.write_message(Message::Text((*(message.clone())).to_string())).unwrap();
                            recp.write_message(Message::Text(cur_message.clone()))
                                .unwrap();
                        }
                    },
                    periodic::Every::new(Duration::from_secs(5)),
                );
                planner.start();

                //loop {
                    

                    //websocket.write_message(Message::Text((*(message.clone())).to_string())).unwrap();
                    //thread::sleep(Duration::from_millis(5000));

                    // We do not want to send back ping/pong messages.
                    // if msg.is_binary() || msg.is_text() {
                    //     websocket
                    //         .write_message(Message::Text("Server message".into()))
                    //         .unwrap();
                    //     //websocket.write_message(msg).unwrap();
                    // }
                //}
            });
        }
    }
}
