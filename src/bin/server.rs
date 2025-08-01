use miniredis::server::server::Server;

fn main() {
    let server = Server::new("127.0.0.1:6379");

    if let Err(e) = server.run() {
        eprintln!("Server failed: {}", e);
        std::process::exit(1);
    }
}
