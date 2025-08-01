use miniredis::client::client::Client;

fn main() {
    let client = Client::new("127.0.0.1:6379");

    if let Err(e) = client.run() {
        eprintln!("Client failed: {}", e);
        std::process::exit(1);
    }
}
