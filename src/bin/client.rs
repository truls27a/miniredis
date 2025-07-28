use miniredis::client::Client;

fn main() {
    let client = Client::new("127.0.0.1:6379");
    client.run();
}