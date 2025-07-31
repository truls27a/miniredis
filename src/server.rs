use crate::kv_store::KVStore;
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

/// A server that listens for client connections and handles requests.
///
/// # Examples
///
/// ```rust
/// use miniredis::server::Server;
///
/// let server = Server::new("127.0.0.1:6379");
/// server.run();
/// ```
pub struct Server {
    address: String,
    store: Arc<KVStore>,
}

impl Server {
    /// Creates a new server.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to listen on.
    ///
    /// # Returns
    ///
    /// A new server.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use miniredis::server::Server;
    ///
    /// let server = Server::new("127.0.0.1:6379");
    /// ```
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            store: Arc::new(KVStore::new()),
        }
    }

    /// Runs the server.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use miniredis::server::Server;
    ///
    /// let server = Server::new("127.0.0.1:6379");
    /// server.run();
    /// ```
    pub fn run(&self) {
        let listener = TcpListener::bind(&self.address).expect("Failed to bind to address");
        println!("MiniRedis is running on {}", self.address);

        for stream in listener.incoming() {
            let stream = stream.expect("Failed to accept connection");
            let store = Arc::clone(&self.store);
            thread::spawn(move || Self::handle_client(stream, store));
        }
    }

    /// Handles a client connection.
    ///
    /// handle_client reads commands from a stream, parses them, executes them, and writes the responses back to the stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - The client stream.
    /// * `store` - The shared key-value store.
    fn handle_client(stream: TcpStream, store: Arc<KVStore>) {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = stream;

        let mut line = String::new();

        loop {
            line.clear();
            let (command, args) = match Self::parse_command(&mut reader, &mut line) {
                Some((command, args)) => (command, args),
                None => continue,
            };

            let response = Self::handle_command(&command, args, &store);

            writer.write_all(response.as_bytes()).unwrap();
        }
    }

    /// Parses a command from a stream.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read the command from.
    /// * `line` - The line to read the command from.
    ///
    /// # Returns
    ///
    /// A optional tuple containing the command and its arguments. If the command is empty or the line is empty, None is returned.
    fn parse_command(
        reader: &mut BufReader<TcpStream>,
        line: &mut String,
    ) -> Option<(String, Vec<String>)> {
        line.clear();
        if reader.read_line(line).unwrap() == 0 {
            return None;
        }

        let mut parts = line.split_whitespace();
        let command = match parts.next() {
            Some(command) => command.to_uppercase(),
            None => return None,
        };
        let args = parts.map(|s| s.to_string()).collect::<Vec<String>>();
        Some((command, args))
    }

    /// Handles a command.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to handle.
    /// * `args` - The arguments to the command.
    /// * `store` - The shared key-value store.
    ///
    /// # Returns
    ///
    /// A string containing the response to the command. Can either be an error message or a response to the command.
    fn handle_command(command: &str, args: Vec<String>, store: &Arc<KVStore>) -> String {
        let key = match args.get(0) {
            Some(key) => key,
            None => return "ERR wrong number of arguments".to_string(),
        };
        let value: Option<&String> = args.get(1);
        match command {
            "GET" => match store.get(key) {
                Some(value) => value,
                None => "nil".to_string(),
            },
            "SET" => {
                match value {
                    Some(value) => store.set(key, value),
                    None => return "ERR wrong number of arguments for 'set' command".to_string(),
                }
                "OK".to_string()
            }
            "DEL" => {
                store.del(key);
                "OK".to_string()
            }
            _ => "ERR unknown command".to_string(),
        }
    }
}
