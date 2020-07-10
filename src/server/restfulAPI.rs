use crate::server::utility;
use futures_util::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use sodiumoxide::crypto::box_::PublicKey;
use sodiumoxide::crypto::box_::SecretKey;

//use serde_json::Result;
use serde_json::json;
use crate::server::handlers;
use crate::server::mailbox;
use crate::server::trait_server::server_trait;
use crate::server::utility::initialization_rest;
use async_trait::async_trait;
use serde_json::Result as sResult;
use serde_json::Value;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex},
    thread,
    cmp,
};


use crate::server::datastructure::intervals;
use crate::server::hooks::serverHooks;

type PartyId = u64;
type ComputationId = String;

pub struct pending_message {
    tag: String,
    store_id: u64,
}
// restAPI specific maps
pub struct maps {
    pub tags: Value,//HashMap<Value, HashMap<Value, u64>>, // { computation_id -> { party_id -> lastTag } }
    pub pendingMessages: Value,//HashMap<Value, HashMap<Value, pending_message>> // { computation_id -> { party_id -> { tag: tag, ptr: ptr } } }
}

// pub struct output_initial {
//     pub success: bool,
//     pub error: Option<String>,
//     pub initialization: Option<initialization_rest>,
//     pub party_id: Option<u32>,
// }
// maps that store state of computations
pub struct computationMaps {
    pub clientIds: Value,//HashMap<Value, Vec<Value>>, // { computation_id -> [ party1_id, party2_id, ...] } for only registered/initialized clients
    pub spareIds: HashMap<String, intervals::intervals>, // { computation_id -> <interval object> }
    pub maxCount: Value,//HashMap<Value, Value>, // { computation_id -> <max number of parties allowed> }
    pub keys: Value,//HashMap<Value, HashMap<Value, Vec<u8>>>, // { computation_id -> { party_id -> <public_key> } }
    pub secretKeys: Value,//HashMap<Value, Vec<u8>>,             // { computation_id -> <privateKey> }
    pub freeParties: Value,//HashMap<Value, HashMap<Value, bool>>, // { computation_id -> { id of every free party -> true } }
}
pub struct restfulAPI {
    pub mail_box: Value,//HashMap<Value, HashMap<Value, Vec<Value>>>,
    pub computationMaps: computationMaps,
    pub hooks: serverHooks,
    pub maps: maps,
    pub sodium: bool,
}


impl server_trait for restfulAPI {
    fn send(&mut self, json: String, party_id: PartyId, computation_id: ComputationId) {
        let mut mailbox = &mut self.mail_box;
        //mailbox::put_in_mailbox(mailbox, computation_id, party_id, json);
    }


}

impl restfulAPI {

    #[tokio::main]
    pub async fn on(instance:   Arc<Mutex<restfulAPI>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = ([127, 0, 0, 1], 8080).into(); //3001 8080
        
        //let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(restfulAPI::listen)) });
        let service = make_service_fn(move |_| {
            let instance = Arc::clone(&instance);
            async { 
            
            Ok::<_, hyper::Error>(service_fn(move |req| {
                
                restfulAPI::listen(req, instance.to_owned())
            }))
        }
        });
        let server = Server::bind(&addr).serve(service);
        println!("Listening on http://{}", addr);
        server.await.unwrap();
        Ok(())
    }

    async fn listen(
        req: Request<Body>,
        restful_instance: Arc<Mutex<restfulAPI>>,
    ) -> Result<Response<Body>, hyper::Error> {
        match (req.method(), req.uri().path()) {
            // Convert to uppercase before sending back to client using a stream.
            (&Method::POST, "/poll") => {
                let full_body = hyper::body::to_bytes(req.into_body()).await?;
                //let body = req.into_body();
                let body_string = std::str::from_utf8(&full_body[..]).unwrap();
                let msg: Value = serde_json::from_str(body_string).unwrap();
                //let deserialized: JasonMessage_rest = serde_json::from_str(&body_string[..]).unwrap();
                println!("{:?}", msg);
                // First: attempt to initialize if needed.
                let output = restfulAPI::initializeParty(msg.clone(), restful_instance.clone());
                if !output["success"].as_bool().unwrap() {
                    return Ok(Response::new(Body::from(json!({
                        "success": json!(false),
                        "label": json!("initialization"),
                        "error": output[String::from("error")],
                    }).to_string())))
                }
                //println!("{:?}", output);
                // Initialization successful: read parameters and construct response!
                let mut response = json!({
                    "success": json!(true),
                    "initialization": output["message"],
                });
                let computation_id = msg["computation_id"].clone();
                let from_id = output["party_id"].clone();

                // Second: free acknowledged tag.
                //jiff.restful.freeTag(computation_id, from_id, msg['ack']);
                restful_instance.lock().unwrap().freeTag(computation_id.clone(), from_id.clone(), msg["ack"].clone());

                // Third: handle given messages / operations.
                //let (receiver_id, msg) =utility::handle_messages(&deserialized, &mut socket_map, addr);
                //println!("{}", body_string);
                // Execute end hooks

                // Fourth: dump mailbox and encrypt.
                let dumped = restful_instance.lock().unwrap().dumpMailbox(computation_id, from_id);
                response.as_object_mut().unwrap().insert(String::from("messages"), dumped["messages"].clone());
                if dumped["ack"] != Value::Null {
                    response.as_object_mut().unwrap().insert(String::from("ack"), dumped["ack"].clone());
                }

                // Execute end hooks
                //response = jiff.hooks.execute_array_hooks('afterOperation', [jiff, 'poll', computation_id, from_id, response], 4);
                // Respond back!
                println!("{}", response.to_string());
                Ok(Response::new(Body::from(response.to_string())))
            }

            // Return the 404 Not Found for other routes.
            _ => {
                let mut not_found = Response::default();
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found)
            }
        }
    }
    // Helpers used within the '/poll' route
    fn initializeParty(mut msg: Value, instance: Arc<Mutex<restfulAPI>>) -> Value {
        let mut initialization = &msg["initialization"];
        if *initialization == Value::Null {
            //println!("initialization");
            if msg["from_id"] == Value::Null {
                return json!({
                    "success": false,
                    "error": "cannot determine party id",
                })
            } 
            return json!({
                "success": true,
                "initialization": Value::Null,
                "party_id": msg["from_id"],
            })       
        } 
        let mut output = handlers::initializeParty(
                instance.clone(),
                &msg["computation_id"],
                &initialization["party_id"],
                &initialization["party_count"],
                initialization,
                false,
            );
        if !output["success"].as_bool().unwrap() {
            return output
        }
        //println!("{:?}", output);
        let inner = output["message"]["party_id"].clone();
        output.as_object_mut().unwrap().insert(String::from("party_id"), inner);
        //output["message"] = json!(output["message"].to_string());
        //println!("{:?}", output);
            //output_initial {success: false, error: Some("cannot determine party id".to_string()), initialization: None, party_id:None}
        return output
    }

    pub fn initComputation (&mut self, computation_id: &Value, party_id: &Value, party_count: &Value) {
        if self.computationMaps.clientIds[computation_id.to_string()] == Value::Null {
            self.computationMaps.clientIds.as_object_mut().unwrap().insert(computation_id.to_string(), json!([]));
            self.computationMaps.maxCount.as_object_mut().unwrap().insert(computation_id.to_string(), party_count.clone());
            self.computationMaps.freeParties.as_object_mut().unwrap().insert(computation_id.to_string(), json!({}));
            self.computationMaps.keys.as_object_mut().unwrap().insert(computation_id.to_string(), json!({}));
            self.mail_box.as_object_mut().unwrap().insert(computation_id.to_string(), json!({}));
 
        }
        if !self.computationMaps.clientIds[computation_id.to_string()].as_array_mut().unwrap().contains(party_id) {
            //println!("push to clinets");
            self.computationMaps.clientIds[computation_id.to_string()].as_array_mut().unwrap().push(party_id.clone());
            println!("{:?}", self.computationMaps.clientIds[computation_id.to_string()]);
        }

        //restful spcific
        if self.maps.tags[computation_id.to_string()] == Value::Null {
            self.maps.tags.as_object_mut().unwrap().insert(computation_id.to_string(), json!({}));
            self.maps.pendingMessages.as_object_mut().unwrap().insert(computation_id.to_string(), json!({}));
        }

        if self.maps.tags[computation_id.to_string()][party_id.to_string()] == Value::Null {
            self.maps.tags[computation_id.to_string()].as_object_mut().unwrap().insert(party_id.to_string(), json!(0));
        }
    }

    pub fn safe_emit (&mut self, label: String, msg: String, computation_id: &Value, to_id: &Value) {
        if to_id == "s1" {
            return
        }

        let store_id = mailbox::put_in_mailbox(self, label, msg, computation_id, to_id);

        // store message in mailbox so that it can be resent in case of failure.

    }

    pub fn dumpMailbox (&mut self, computation_id: Value, party_id: Value) -> Value {
        let mailbox = mailbox::get_from_mailbox(self, computation_id.clone(), party_id.clone());
        if mailbox.len() == 0 {
            return json!({
                "messages": json!([])
            })
        }

        let mut messages = Vec::new();
        let count = cmp::min(150, mailbox.len());
        let mut i = 0;
        while i < count {
            let letter = mailbox[i].clone();
            messages.push(json!({
                "label": letter[String::from("label")],
                "payload": letter[String::from("msg")],
            }));
            i = i + 1;
        }

        // come up with unique tag
        let x: u64 = 2;
        let LARGE = x.pow(32);
        let tag = (self.maps.tags[computation_id.to_string()][party_id.to_string()].as_u64().unwrap() + 1) % LARGE;
        self.maps.tags[computation_id.to_string()].as_object_mut().unwrap().insert(party_id.to_string(), json!(tag));
        self.maps.pendingMessages[computation_id.to_string()].as_object_mut().unwrap().insert(party_id.to_string(), json!({
            "tag": json!(tag),
            "store_id": mailbox[count - 1]["id"],
        }));
        return json!({
            "ack": json!(tag),
            "messages": json!(messages),
        })
    }

    pub fn freeTag (&mut self, computation_id: Value, party_id: Value, tag: Value) {
        if tag != Value::Null && self.maps.pendingMessages[computation_id.to_string()] != Value::Null && self.maps.pendingMessages[computation_id.to_string()][party_id.to_string()] != Value::Null {
            let pending = self.maps.pendingMessages[computation_id.to_string()][party_id.to_string()].clone();
            if tag == pending["tag"] {
                mailbox::sliceMailbox(self, computation_id.clone(), party_id.clone(), pending["store_id"].clone());
                self.maps.pendingMessages[computation_id.to_string()][party_id.to_string()] = Value::Null;
            }
        }
    }
}
