use std::net::TcpStream;
use std::io::{self, BufRead, BufReader, Write};
use crate::error::MiniRedisError;

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
    /// Run starts the client and connects to the server.
    /// It will then enter a loop where it reads input from the user,
    /// sends it to the server, and prints the response.
    /// 
    /// # Panics
    /// 
    /// If the client fails to connect to the server,
    /// read from the stream, or write to the stream, it will panic.
    ///
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use miniredis::client::Client;
    /// 
    /// let client = Client::new("127.0.0.1:6379");
    /// client.run();
    /// ```
    pub fn run(&self) -> Result<(), MiniRedisError> {
        let mut stream = TcpStream::connect(&self.address).map_err(|_| MiniRedisError::StreamNotConnected)?;
        let mut reader = BufReader::new(stream.try_clone().map_err(|_| MiniRedisError::StreamClosed)?);

        println!("Connected to server at {}", self.address);

        loop {
            print!("> ");
            io::stdout().flush().map_err(|_| MiniRedisError::StreamNotFlushed)?;

            let input = self.read_input()?;

            if input.is_empty() {
                continue;
            }
            
            if input == "quit" {
                break;
            }

            if let Err(_) = self.send_input(&input, &mut stream) {
                break;
            }

            let response = match self.read_response(&mut reader) {
                Ok(response) => response,
                Err(_) => break,
            };

            println!("{}", response);
        }

        Ok(())
    }

    /// Reads input from the user.
    /// 
    /// # Returns
    /// 
    /// A string containing the input from the user.
    fn read_input(&self) -> Result<String, MiniRedisError> {
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|_| MiniRedisError::StreamNotReadable)?;
        Ok(input)
    }

    /// Sends input to the server.
    /// 
    /// # Arguments
    /// 
    /// * `input` - The input to send to the server.
    /// * `stream` - The stream to send the input to.
    /// 
    /// # Returns
    /// 
    /// A result indicating whether the input was sent successfully.
    /// 
    fn send_input(&self, input: &str, stream: &mut TcpStream) -> Result<(), MiniRedisError> {
        stream.write_all(input.as_bytes()).map_err(|_| MiniRedisError::StreamNotWritable)?;
        stream.write_all(b"\n").map_err(|_| MiniRedisError::StreamNotWritable)?;
        Ok(())
    }

    /// Reads a response from the server.
    /// 
    /// # Arguments
    /// 
    /// * `reader` - The reader to read the response from.
    /// 
    /// # Returns
    /// 
    /// A result containing the response from the server.
    /// If the response is empty, an error is returned.
    fn read_response(&self, reader: &mut BufReader<TcpStream>) -> Result<String, MiniRedisError> {
        let mut response = String::new();
        reader.read_line(&mut response).map_err(|_| MiniRedisError::StreamNotReadable)?;
        Ok(response)
    }
}