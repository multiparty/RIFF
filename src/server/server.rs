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
    socket_ids: HashMap<u32, HashMap<u32, WebSocket<TcpStream>>>, //first: computation id ; second:party id
    //computation_ids: HashMap<SocketAddr, u32>,
    computation_ids: HashMap<WebSocket<TcpStream>, u32>,
    party_ids: HashMap<WebSocket<TcpStream>, u32>, // party id
}

pub struct Server {
    pub name: String,
    //socketMap: SocketMap,
}

impl Server {
    pub fn on(&self) {
        let mut socket_map = SocketMap {
            socket_ids : HashMap::new(),
            computation_ids : HashMap::new(),
            party_ids :HashMap::new(),
        };
        socket_map.socket_ids.insert(1, HashMap::new());
        let socket_map = Arc::new(RwLock::new(socket_map));

        let server = TcpListener::bind("127.0.0.1:9001").unwrap();
        //let shared_message = Arc::new(Mutex::new(String::from("")));
        //let websockets_hashmap = Arc::new(RwLock::new(HashMap::new()));
        let counter = Arc::new(Mutex::new(0));
        

        for stream in server.incoming() {
            
            //let websockets_hashmap = Arc::clone(&websockets_hashmap);
            let socket_map = Arc::clone(&socket_map);
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
                    let mut socket_map = socket_map.write().unwrap();
                    socket_map.computation_ids.insert(websocket, 1);
                    let mut socket_ids = &mut socket_map.socket_ids;
                    socket_ids.get_mut(&1).unwrap().insert(id, websocket);
                    
                    println!("{:?}", socket_ids.get(&1).unwrap().get(&id));
                }

                let mut planner = periodic::Planner::new();
                planner.add(
                    move || {//let cur_websocket;//:
                        let mut socket_map = socket_map.write().unwrap();
                        let cur_websocket: &mut tungstenite::protocol::WebSocket<std::net::TcpStream> =
                            socket_map.socket_ids.get_mut(&1).unwrap().get_mut(&id).unwrap();
                        let msg = cur_websocket.read_message().unwrap();
    
                        println!("Received: {}", msg);
                        let cur_message = msg.to_string();
    
                        // let mut message = shared_message.lock().unwrap();
                        // *message = String::from("");
                        // *message += msg.to_text().unwrap();
    
                        let broadcast_recipients = &mut socket_map.socket_ids.get_mut(&1).unwrap().iter_mut().map(|(_, socket)| socket);
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
