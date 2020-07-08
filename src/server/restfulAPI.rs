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
                let deserialized: Value = serde_json::from_str(body_string).unwrap();
                //let deserialized: JasonMessage_rest = serde_json::from_str(&body_string[..]).unwrap();
                println!("{:?}", deserialized);
                let output = restfulAPI::initializeParty(deserialized, restful_instance.clone());
                println!("{}", body_string);
                if !output["success"].as_bool().unwrap() {
                    return Ok(Response::new(Body::from(output.to_string())))
                }
                // Third: handle given messages / operations.
                //let (receiver_id, msg) =utility::handle_messages(&deserialized, &mut socket_map, addr);
                //println!("{}", body_string);
                // Execute end hooks
                Ok(Response::new(Body::from(output.to_string())))
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

        output["party_id"] = output["message"]["party_id"].clone();
        output["message"] = json!(output["message"].to_string());
            //output_initial {success: false, error: Some("cannot determine party id".to_string()), initialization: None, party_id:None}
        return output
    }

    pub fn initComputation (&mut self, computation_id: &Value, party_id: &Value, party_count: &Value) {
        if self.computationMaps.clientIds[computation_id.as_str().unwrap()] == Value::Null {
            self.computationMaps.clientIds.as_object_mut().unwrap().insert(computation_id.to_string(), json!([]));
            self.computationMaps.maxCount.as_object_mut().unwrap().insert(computation_id.to_string(), party_count.clone());
            self.computationMaps.freeParties.as_object_mut().unwrap().insert(computation_id.to_string(), json!({}));
            self.computationMaps.keys.as_object_mut().unwrap().insert(computation_id.to_string(), json!({}));
            self.mail_box.as_object_mut().unwrap().insert(computation_id.to_string(), json!({}));
 
        }
        if !self.computationMaps.clientIds[computation_id.to_string()].as_array_mut().unwrap().contains(&party_id) {
            self.computationMaps.clientIds[computation_id.to_string()].as_array_mut().unwrap().push(party_id.clone());
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
        if to_id == 999 {
            return
        }

        let store_id = mailbox::put_in_mailbox(self, label, msg, computation_id, to_id);

        // store message in mailbox so that it can be resent in case of failure.

    }
}
