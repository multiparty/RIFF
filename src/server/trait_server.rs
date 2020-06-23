use async_trait::async_trait;
use hyper::{Body, Request, Response, Server};
#[async_trait]
pub trait server_trait {
    fn send (&mut self, json: String, party_id:u32, computation_id: u32);

    async fn listen(req: Request<Body>) -> Result<Response<Body>, hyper::Error>;
}