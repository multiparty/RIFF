use crate::server::utility;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::RwLock;
use std::thread::spawn;
use std::time::Duration;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::protocol::WebSocket;
use tungstenite::server::accept;
use tungstenite::{accept_hdr, connect, Message};

use crate::server::mailbox;
use crate::server::utility::JasonMessage;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
};

type PartyId = u32;
type ComputationId = u32;

pub struct SocketMap {
    pub socket_ids: HashMap<ComputationId, HashMap<PartyId, WebSocket<TcpStream>>>, //first: computation id ; second:party id
    pub computation_ids: HashMap<SocketAddr, ComputationId>,
    pub party_ids: HashMap<SocketAddr, PartyId>, // party id
}

pub struct Server {
    pub name: String,
    pub mail_box: HashMap<ComputationId, HashMap<PartyId, Vec<String>>>, // first: computation id ; second: party_id
                                                                         //socketMap: SocketMap,
}

impl Server {
    //pub fn on(&mut self) {
    pub fn on(this: Arc<Mutex<Server>>) {
        let mut socket_map = SocketMap {
            socket_ids: HashMap::new(),
            computation_ids: HashMap::new(),
            party_ids: HashMap::new(),
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
                    if id == 1 || id == 2 || id == 3 {
                        socket_map.computation_ids.insert(addr, 1);
                    } else {
                        socket_map.computation_ids.insert(addr, 2);
                    }
                    socket_map.party_ids.insert(addr, id);
                    let computation_id = socket_map.computation_ids.get(&addr).unwrap();
                    let computation_id = *computation_id;
                    let mut socket_ids = &mut socket_map.socket_ids;

                    socket_ids
                        .entry(computation_id)
                        .or_insert(HashMap::new())
                        .insert(id, websocket);

                    // build mail_box
                    let mut server_instance = this_clone.lock().unwrap();
                    let mut computation_mailbox = server_instance
                        .mail_box
                        .entry(computation_id)
                        .or_insert(HashMap::new());
                    computation_mailbox.entry(id).or_insert(Vec::new());
                    //self.mail_box.entry(computation_id).or_insert(HashMap::new()).insert(id, Vec::new() );
                    //println!("{:?}", socket_ids.get(&1).unwrap().get(&id));
                }

                let mut planner = periodic::Planner::new();
                planner.add(
                    move || {
                        //let cur_websocket;//:
                        let mut socket_map = socket_map.write().unwrap();
                        let computation_id = socket_map.computation_ids.get(&addr).unwrap();
                        let computation_id = *computation_id;
                        let cur_websocket: &mut tungstenite::protocol::WebSocket<
                            std::net::TcpStream,
                        > = socket_map
                            .socket_ids
                            .get_mut(&computation_id)
                            .unwrap()
                            .get_mut(&id)
                            .unwrap();
                        let msg = cur_websocket.read_message().unwrap();

                        println!("Received: {}", msg);
                        let cur_message = msg.to_string();
                        let deserialized: JasonMessage =
                            serde_json::from_str(&cur_message[..]).unwrap();
                        println!("deserialized = {:?}", deserialized);
                        let (receiver_id, msg) =
                            utility::handle_messages(&deserialized, &mut socket_map, addr);
                        let mut mailbox = &mut this_clone.lock().unwrap().mail_box;

                        //mailbox::put_in_mailbox(mailbox, computation_id,receiver_id, msg );
                        //let vec =mailbox::get_from_mailbox(mailbox, computation_id, id);
                        //println!("Clinet{} mailbox: {:?}", id,vec);
                        // let broadcast_recipients = &mut socket_map.socket_ids.get_mut(&computation_id).unwrap().iter_mut().map(|(_, socket)| socket);
                        // for recp in broadcast_recipients {
                        //     //recp.write_message(Message::Text((*(message.clone())).to_string())).unwrap();
                        //     recp.write_message(Message::Text(cur_message.clone()))
                        //         .unwrap();
                        // }
                    },
                    periodic::Every::new(Duration::from_secs(3)),
                );
                planner.start();
            });
            //}).unwrap();
        }
    }
}
