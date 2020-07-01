use async_trait::async_trait;
use hyper::{Body, Request, Response, Server};
use crate::server::restfulAPI::computationMaps;

use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex},
    thread,
};
//#[async_trait]
pub trait server_trait {
    fn send (&mut self, json: String, party_id:u64, computation_id: String);

    //async fn listen(req: Request<Body>, computationMaps: computationMaps) -> Result<Response<Body>, hyper::Error>;
}