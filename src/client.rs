use std::net::TcpStream;
use std::io::{self, BufRead, BufReader, Write};

/// A client that connects to a server and sends requests.
/// 
/// # Examples
/// 
/// ```rust
/// use miniredis::client::Client;
/// 
/// let client = Client::new("127.0.0.1:6379");
/// client.run();
/// ```
pub struct Client {
    address: String,
}

impl Client {
    /// Creates a new client.
    /// 
    /// # Arguments
    /// 
    /// * `address` - The address of the server to connect to.
    /// 
    /// # Returns
    /// 
    /// A new client.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use miniredis::client::Client;
    /// 
    /// let client = Client::new("127.0.0.1:6379");
    /// client.run();
    /// ```
    pub fn new(address: &str) -> Self {
        Self { address: address.to_string() }
    }

    /// Runs the client.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use miniredis::client::Client;
    /// 
    /// let client = Client::new("127.0.0.1:6379");
    /// client.run();
    /// ```
    pub fn run(&self) {
        let stream = TcpStream::connect(&self.address).expect("Failed to connect to server");
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = stream;

        println!("Connected to server at {}", self.address);

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                println!("Error reading input");
                continue;
            }

            let trimmed = input.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            if trimmed == "quit" {
                break;
            }

            if let Err(e) = writer.write_all(trimmed.as_bytes()) {
                println!("Failed to send: {}", e);
                break;
            }
            if let Err(err) = writer.write_all(b"\n") {
                println!("Failed to send newline: {}", err);
                break;
            }

            let mut response = String::new();
            if reader.read_line(&mut response).is_err() {
                println!("Error reading response");
                break;
            }

            println!("{}", response);
        }
    }
}