use crate::server::JasonMessage;
use crate::server::SocketMap;
use std::net::SocketAddr;
use tungstenite::Message;

pub fn handle_messages<'a>(
    object: &'a JasonMessage,
    socket_map: &mut SocketMap,
    addr: SocketAddr,
) {
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
        cur_websocket.write_message(Message::Text(res)).unwrap();
    //res
    } else if &object.tag[..] == "communicate" {
        let computation_id = socket_map.computation_ids.get(&addr).unwrap();
        let computation_id = *computation_id;
        let broadcast_recipients = &mut socket_map
            .socket_ids
            .get_mut(&computation_id)
            .unwrap()
            .iter_mut();
        for (id,recp) in broadcast_recipients {
            if *id == object.party_id {
                recp.write_message(Message::Text(object.message.clone()))
                .unwrap();
            }
            
        }
    }
}
