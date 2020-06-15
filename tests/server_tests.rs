use riff::server;

#[test]
fn open_websocket() {
    let s = server::Server{ name: String::from("test_server")};
    s.on();
}
