use crate::server::utility;
use futures_util::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;

//use serde_json::Result;
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

type PartyId = u32;
type ComputationId = u32;

pub struct output_initial {
    pub success: bool,
    pub error: Option<String>,
    pub initialization: Option<initialization_rest>,
    pub party_id: Option<u32>,
}
// maps that store state of computations
pub struct computationMaps<'a> {
    pub clientIds: HashMap<&'a str, Vec<String>>, // { computation_id -> [ party1_id, party2_id, ...] } for only registered/initialized clients
    //spareIds: HashMap<&str, >, // { computation_id -> <interval object> }
    pub maxCount: HashMap<&'a str, u32>, // { computation_id -> <max number of parties allowed> }
    pub keys: HashMap<&'a str, HashMap<&'a str, &'a str>>, // { computation_id -> { party_id -> <public_key> } }
    pub secretKeys: HashMap<&'a str, &'a str>,             // { computation_id -> <privateKey> }
    pub freeParties: HashMap<&'a str, HashMap<&'a str, bool>>, // { computation_id -> { id of every free party -> true } }
}
pub struct restfulAPI<'a> {
    pub mail_box: HashMap<ComputationId, HashMap<PartyId, Vec<String>>>,
    pub computationMaps: computationMaps<'a>,
}


impl<'a> server_trait for restfulAPI<'a> {
    fn send(&mut self, json: String, party_id: PartyId, computation_id: ComputationId) {
        let mut mailbox = &mut self.mail_box;
        mailbox::put_in_mailbox(mailbox, computation_id, party_id, json);
    }


}

impl<'a> restfulAPI<'a> {
    #[tokio::main]
    pub async fn on(instance:   Arc<Mutex<restfulAPI<'static>>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = ([127, 0, 0, 1], 3001).into();
        
        //let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(restfulAPI::listen)) });
        let service = make_service_fn( |_| async { 
            Ok::<_, hyper::Error>(service_fn( |req| 
                
                restfulAPI::listen(req, instance.clone())
            ))
        });
        let server = Server::bind(&addr).serve(service);
        println!("Listening on http://{}", addr);
        server.await.unwrap();
        Ok(())
    }

    async fn listen(
        req: Request<Body>,
        restful_instance: Arc<Mutex<restfulAPI<'static>>>,
    ) -> Result<Response<Body>, hyper::Error> {
        match (req.method(), req.uri().path()) {
            // Convert to uppercase before sending back to client using a stream.
            (&Method::POST, "/poll") => {
                let full_body = hyper::body::to_bytes(req.into_body()).await?;
                //let body = req.into_body();
                let body_string = std::str::from_utf8(&full_body[..]).unwrap();
                let deserialized: Value = serde_json::from_str(body_string).unwrap();
                //let deserialized: JasonMessage_rest = serde_json::from_str(&body_string[..]).unwrap();
                let output = restfulAPI::initializeParty(deserialized, restful_instance);
                //let (receiver_id, msg) =utility::handle_messages(&deserialized, &mut socket_map, addr);
                println!("{}", body_string);
                //println!("{:?}", deserialized);
                Ok(Response::new(Body::from("Success")))
            }

            // Return the 404 Not Found for other routes.
            _ => {
                let mut not_found = Response::default();
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found)
            }
        }
    }

    fn initializeParty(msg: Value, instance: Arc<Mutex<restfulAPI>>) -> output_initial {
        let initialization = &msg["initialization"];
        if *initialization == Value::Null {
            if msg["from_id"] == Value::Null {
                output_initial {
                    success: false,
                    error: Some("cannot determine party id".to_string()),
                    initialization: None,
                    party_id: None,
                }
            } else {
                output_initial {
                    success: true,
                    error: None,
                    initialization: None,
                    party_id: Some(msg["from_id"].as_u64().unwrap() as u32),
                }
            }
        } else {
            let output = handlers::initializeParty(
                instance,
                msg["computation_id"].as_str().unwrap(),
                initialization["party_id"].as_str().unwrap(),
                &initialization["party_count"],
                initialization,
                false,
            );
            output
            //output_initial {success: false, error: Some("cannot determine party id".to_string()), initialization: None, party_id:None}
        }
    }
}
