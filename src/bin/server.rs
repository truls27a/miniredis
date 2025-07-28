use miniredis::server::Server;

fn main() {
    let server = Server::new("127.0.0.1:6379");
    server.run();
}