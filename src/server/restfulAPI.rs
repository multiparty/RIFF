use futures_util::TryStreamExt;
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, StatusCode};
use crate::server::utility;

//use serde_json::Result;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex},
    thread,
};
use crate::server::mailbox;
use crate::server::trait_server::server_trait;
use async_trait::async_trait;
use crate::server::utility::JasonMessage_rest;

type PartyId = u32;
type ComputationId = u32;


pub struct restfulAPI {
    pub mail_box: HashMap<ComputationId, HashMap<PartyId, Vec<String>>>,
}
#[async_trait]
impl server_trait for restfulAPI {

    fn send (&mut self, json: String, party_id: PartyId, computation_id: ComputationId) {
        let mut mailbox = &mut self.mail_box;
        mailbox::put_in_mailbox(mailbox, computation_id,party_id, json );
    }

    async fn listen(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        match (req.method(), req.uri().path()) {
            
    
            // Convert to uppercase before sending back to client using a stream.
            (&Method::POST, "/poll") => {
                let full_body = hyper::body::to_bytes(req.into_body()).await?;
                //let body = req.into_body();
                let body_string = std::str::from_utf8(&full_body[..]).unwrap().to_string();
                //let deserialized: JasonMessage_rest = serde_json::from_str(&body_string[..]).unwrap();
                //let (receiver_id, msg) =utility::handle_messages(&deserialized, &mut socket_map, addr);
                println!("{}", body_string);
                Ok(Response::new(Body::from(
                    "Success",
                )))
            }
    
    
            // Return the 404 Not Found for other routes.
            _ => {
                let mut not_found = Response::default();
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found)
            }
        }
    }

    
}

impl restfulAPI {
    #[tokio::main]
    pub async fn on (&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
        let addr = ([127, 0, 0, 1], 3001).into();
        let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(restfulAPI::listen)) });
        let server = Server::bind(&addr).serve(service);
        println!("Listening on http://{}", addr);
        server.await?;
        Ok(())
    }
}