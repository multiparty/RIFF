use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::RwLock;
use std::thread::spawn;
use std::time::Duration;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::protocol::WebSocket;
use tungstenite::server::accept;
use tungstenite::{accept_hdr, connect, Message};
use crate::server::utility;
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
use crossbeam_utils::thread as crossbeam_thread;

pub struct SocketMap {
    pub socket_ids: HashMap<u32, HashMap<u32, WebSocket<TcpStream>>>, //first: computation id ; second:party id
    pub computation_ids: HashMap<SocketAddr, u32>,
    pub party_ids: HashMap<SocketAddr, u32>, // party id
}

pub struct Server {
    pub name: String,
    pub mail_box: HashMap<u32, HashMap<u32, Vec<JasonMessage>>>, // first: computation id ; second: party_id
    //socketMap: SocketMap,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JasonMessage {
    pub tag: String,
    pub party_id : u32,
    pub message : String,
}

impl Server {
    //pub fn on(&mut self) {
      pub fn on(this: Arc<Mutex<Server>>) {
        let mut socket_map = SocketMap {
            socket_ids : HashMap::new(),
            computation_ids : HashMap::new(),
            party_ids :HashMap::new(),
        };
        
        let socket_map = Arc::new(RwLock::new(socket_map));

        let server = TcpListener::bind("127.0.0.1:9001").unwrap();
        //let shared_message = Arc::new(Mutex::new(String::from("")));
        //let websockets_hashmap = Arc::new(RwLock::new(HashMap::new()));
        let counter = Arc::new(Mutex::new(0));
        //let arc_self = Arc::new(Mutex::new(self));
        

        while let Ok((stream, addr)) = server.accept() {
            
            //let websockets_hashmap = Arc::clone(&websockets_hashmap);
            let socket_map = Arc::clone(&socket_map);
            let counter = Arc::clone(&counter);
            //let arc_self = arc_self.clone();
            let this_clone = this.clone();
            //crossbeam_thread::scope(|s| {
                //s.
                spawn(move || {
                    println!("new thread!");
                    let id;
                    {
                        let mut num = counter.lock().unwrap();
                        *num += 1;
                        println!("Received: {}", &num);
                        id = num.clone();
                    }
                    let websocket = accept(stream).unwrap();
    
                    //build SocketMap and mail_box
                    {
                        let mut socket_map = socket_map.write().unwrap();
                        if id == 1 || id == 2 || id ==3 {
                            socket_map.computation_ids.insert(addr, 1);
                        } else {
                            socket_map.computation_ids.insert(addr, 2);
                        }
                        socket_map.party_ids.insert(addr, id);
                        let computation_id = socket_map.computation_ids.get(&addr).unwrap();
                        let computation_id = *computation_id;
                        let mut socket_ids = &mut socket_map.socket_ids;
                        
                        socket_ids.entry(computation_id).or_insert(HashMap::new()).insert(id, websocket);
                        
                        // build mail_box
                        this_clone.lock().unwrap().mail_box.entry(computation_id).or_insert(HashMap::new()).insert(id, Vec::new());
                        //self.mail_box.entry(computation_id).or_insert(HashMap::new()).insert(id, Vec::new() );
                        //println!("{:?}", socket_ids.get(&1).unwrap().get(&id));
                    }
    
    
                    let mut planner = periodic::Planner::new();
                    planner.add(
                        move || {//let cur_websocket;//:
                            let mut socket_map = socket_map.write().unwrap();
                            let computation_id = socket_map.computation_ids.get(&addr).unwrap();
                            let computation_id = *computation_id;
                            let cur_websocket: &mut tungstenite::protocol::WebSocket<std::net::TcpStream> =
                                socket_map.socket_ids.get_mut(&computation_id).unwrap().get_mut(&id).unwrap();
                            let msg = cur_websocket.read_message().unwrap();
        
                            println!("Received: {}", msg);
                            let cur_message = msg.to_string();
                            let deserialized: JasonMessage = serde_json::from_str(&cur_message[..]).unwrap();
                            println!("deserialized = {:?}", deserialized);
                            utility::handle_messages(&deserialized, &mut socket_map, addr);
                            
    
                            // let broadcast_recipients = &mut socket_map.socket_ids.get_mut(&computation_id).unwrap().iter_mut().map(|(_, socket)| socket);
                            // for recp in broadcast_recipients {
                            //     //recp.write_message(Message::Text((*(message.clone())).to_string())).unwrap();
                            //     recp.write_message(Message::Text(cur_message.clone()))
                            //         .unwrap();
                            // }
                        },
                        periodic::Every::new(Duration::from_secs(5)),
                    );
                    planner.start();
    
                    
                });
            //}).unwrap();
            
        }
    }
}
