mod server;
pub use crate::server::Server;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        //assert_eq!(2 + 2, 4);
        let server = super::Server {
            name: String::from("server1"),
        };

        server.on();
    }
}
