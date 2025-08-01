use std::env;
use miniredis::client::client::Client;

fn main() {
    let args: Vec<String> = env::args().collect();
    let client = Client::from_args(&args);

    if let Err(e) = client.run() {
        eprintln!("Client failed: {}", e);
        std::process::exit(1);
    }
}
