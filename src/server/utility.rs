
use crate::server::SocketMap;
use std::net::SocketAddr;
use tungstenite::Message;
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct JasonMessage {
    pub tag: String,
    pub party_id : u32,
    pub message : String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JasonMessage_rest {
    pub messages: Vec<message_rest>,
    pub initialization : initialization_rest,
    pub computation_id : String,
    //pub from_id : u32,
    
    
    //pub ack: String,
    //pub additional: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct message_rest {
    pub label: String,
    pub payload: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct initialization_rest {
    pub computation_id: String,
    pub party_count: u32,
    pub public_key: String,
}

pub fn handle_messages<'a> (
    object: &'a JasonMessage,
    socket_map: &mut SocketMap,
    addr: SocketAddr,
) -> (u32, String) {
    if &object.tag[..] == "connect" {
        println!("Welcome: {}", object.message);
        let mut res = "Welcome: ".to_string();
        res.push_str(&(object.message.clone())[..]);
        let computation_id = socket_map.computation_ids.get(&addr).unwrap();
        let computation_id = *computation_id;
        let id = *socket_map.party_ids.get(&addr).unwrap();
        let cur_websocket: &mut tungstenite::protocol::WebSocket<std::net::TcpStream> = socket_map
            .socket_ids
            .get_mut(&computation_id)
            .unwrap()
            .get_mut(&id)
            .unwrap();
        cur_websocket.write_message(Message::Text(res.clone())).unwrap();
        (id, res)
    //res
    } else if &object.tag[..] == "communicate" {
        let computation_id = socket_map.computation_ids.get(&addr).unwrap();
        let computation_id = *computation_id;
        let broadcast_recipients = &mut socket_map
            .socket_ids
            .get_mut(&computation_id)
            .unwrap()
            .iter_mut();
        let mut res=(0,"error".to_string());
        for (id,recp) in broadcast_recipients {
            if *id == object.party_id {
                recp.write_message(Message::Text(object.message.clone())).unwrap();
            
                res=(*id, object.message.clone());
                
            } 
            
        }
        res
        //(0,"error".to_string())
        
    } else {
        (0,"error".to_string())
    }
    
    
    
    
}
