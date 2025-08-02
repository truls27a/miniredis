use crate::error::MiniRedisError;
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
/// ```rust,no_run
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

    /// Creates a new server from command line arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - The command line arguments.
    ///
    /// # Returns
    ///
    /// A new server.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use miniredis::server::Server;
    ///
    /// let server = Server::from_args(&["miniredis".to_string(), "127.0.0.1:6379".to_string()]);
    /// server.run();
    /// ```
    pub fn from_args(args: &[String]) -> Self {
        let address = if args.len() > 1 {
            &args[1]
        } else {
            "127.0.0.1:6379"
        };
        Self::new(address)
    }

    /// Runs the server.
    ///
    /// Run starts the server and listens for client connections.
    /// When receiving a client connection, it will spawn a new thread.
    /// It will then handle the client messages in a loop.
    /// Each message is parsed and then executed through the key value store,
    /// and the response is written back to the client.
    ///
    /// # Returns
    ///
    /// A result indicating whether the server was started successfully.
    ///
    /// # Errors
    ///
    /// If the server fails to bind to the address,
    /// read from the stream, or write to the stream, it will return an error.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// use miniredis::server::Server;
    ///
    /// let server = Server::new("127.0.0.1:6379");
    /// server.run();
    /// ```
    pub fn run(&self) -> Result<(), MiniRedisError> {
        let listener =
            TcpListener::bind(&self.address).map_err(|_| MiniRedisError::AddressNotBound)?;
        println!("MiniRedis is running on {}", self.address);

        for stream in listener.incoming() {
            let stream = stream.map_err(|_| MiniRedisError::StreamNotConnected {
                address: self.address.clone(),
            })?;
            let store = Arc::clone(&self.store);
            thread::spawn(move || Self::handle_client(stream, store));
        }
        Ok(())
    }

    /// Prints the help message.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use miniredis::server::Server;
    ///
    /// Server::print_help();
    /// ```
    pub fn print_help() {
        println!("MiniRedis Server");
        println!();
        println!("Starts the MiniRedis server and listens for client connections.");
        println!();
        println!("USAGE:");
        println!("    miniredis server <ADDRESS>");
        println!();
        println!("ARGS:");
        println!("    <ADDRESS>    The address to listen on [default: 127.0.0.1:6379]");
        println!();
        println!("EXAMPLES:");
        println!("    miniredis server 127.0.0.1:6379");
        println!("    miniredis server --help");
    }

    /// Handles a client connection.
    ///
    /// handle_client reads commands from a stream, parses them,
    /// executes them, and writes the responses back to the stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - The client stream.
    /// * `store` - The shared key-value store.
    ///
    /// # Returns
    ///
    /// A result indicating whether the client was handled successfully.
    ///
    /// # Errors
    ///
    /// If the stream is not readable, writable, or closed, it will return an error.
    fn handle_client(mut stream: TcpStream, store: Arc<KVStore>) -> Result<(), MiniRedisError> {
        let mut reader = BufReader::new(
            stream
                .try_clone()
                .map_err(|_| MiniRedisError::StreamClosed)?,
        );

        let mut line = String::new();

        loop {
            line.clear();
            if reader
                .read_line(&mut line)
                .map_err(|_| MiniRedisError::StreamNotReadable)?
                == 0
            {
                break;
            }

            let (command, args) = match Self::parse_command(&line) {
                Some((command, args)) => (command, args),
                None => continue,
            };

            let response = match Self::handle_command(&command, args, &store) {
                Ok(response) => response,
                Err(e) => e.to_string(),
            };

            stream
                .write_all(response.as_bytes())
                .map_err(|_| MiniRedisError::StreamNotWritable)?;
            stream
                .write_all(b"\n")
                .map_err(|_| MiniRedisError::StreamNotWritable)?;
        }
        Ok(())
    }

    /// Parses a command from a stream.
    ///
    /// # Arguments
    ///
    /// * `line` - The line to read the command from.
    ///
    /// # Returns
    ///
    /// A optional tuple containing the command and its arguments.
    /// If the command is empty or the line is empty, None is returned.
    fn parse_command(line: &str) -> Option<(String, Vec<String>)> {
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
    /// A string containing the response to the command.
    /// Can either be an error message or a response to the command.
    ///
    /// # Errors
    ///
    /// If the command is invalid, the arguments are invalid,
    /// or the key is not found, it will return an error.
    fn handle_command(
        command: &str,
        args: Vec<String>,
        store: &Arc<KVStore>,
    ) -> Result<String, MiniRedisError> {
        let key: Option<&String> = args.get(0);
        let value: Option<&String> = args.get(1);
        let args_len = args.len();

        match command {
            "GET" => {
                if args_len != 1 {
                    return Err(MiniRedisError::InvalidArguments { arguments: args });
                }
                match key {
                    Some(key) => match store.get(key) {
                        Ok(Some(value)) => Ok(value),
                        Ok(None) => Ok("nil".to_string()),
                        Err(e) => Err(e),
                    },
                    None => Err(MiniRedisError::InvalidArguments { arguments: args }),
                }
            }
            "SET" => {
                if args_len != 2 {
                    return Err(MiniRedisError::InvalidArguments { arguments: args });
                }
                match key {
                    Some(key) => match value {
                        Some(value) => {
                            store.set(key, value)?;
                            Ok("OK".to_string())
                        }
                        None => Err(MiniRedisError::InvalidArguments { arguments: args }),
                    },
                    None => Err(MiniRedisError::InvalidArguments { arguments: args }),
                }
            }
            "DEL" => {
                if args_len != 1 {
                    return Err(MiniRedisError::InvalidArguments { arguments: args });
                }
                match key {
                    Some(key) => {
                        store.del(key)?;
                        Ok("OK".to_string())
                    }
                    None => Err(MiniRedisError::InvalidArguments { arguments: args }),
                }
            }
            _ => Err(MiniRedisError::InvalidCommand {
                command: command.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_server_with_address() {
        let address = "127.0.0.1:0";
        let server = Server::new(address);
        assert_eq!(address, server.address);
    }

    #[test]
    fn new_creates_server_with_empty_store() {
        let server = Server::new("127.0.0.1:0");
        assert!(server.store.get("nonexistent_key").unwrap().is_none());
    }

    #[test]
    fn from_args_uses_default_address_when_no_args_provided() {
        let args = vec!["miniredis".to_string()];
        let server = Server::from_args(&args);
        assert_eq!("127.0.0.1:6379", server.address);
    }

    #[test]
    fn from_args_uses_provided_address_when_args_given() {
        let expected_address = "localhost:9999";
        let args = vec!["miniredis".to_string(), expected_address.to_string()];
        let server = Server::from_args(&args);
        assert_eq!(expected_address.to_string(), server.address);
    }

    #[test]
    fn from_args_uses_first_argument_as_address() {
        let expected_address = "test.example.com:1234";
        let args = vec![
            "miniredis".to_string(),
            expected_address.to_string(),
            "ignored_arg".to_string(),
        ];
        let server = Server::from_args(&args);
        assert_eq!(expected_address.to_string(), server.address);
    }
    #[test]
    fn parse_command_parses_get_command() {
        let line = "GET mykey\n";
        let result = Server::parse_command(line);
        assert_eq!(Some(("GET".to_string(), vec!["mykey".to_string()])), result);
    }

    #[test]
    fn parse_command_parses_set_command() {
        let line = "SET mykey myvalue\n";
        let result = Server::parse_command(line);
        assert_eq!(
            Some((
                "SET".to_string(),
                vec!["mykey".to_string(), "myvalue".to_string()]
            )),
            result
        );
    }

    #[test]
    fn parse_command_parses_del_command() {
        let line = "DEL mykey\n";
        let result = Server::parse_command(line);
        assert_eq!(Some(("DEL".to_string(), vec!["mykey".to_string()])), result);
    }

    #[test]
    fn parse_command_handles_lowercase_commands() {
        let line = "get mykey\n";
        let result = Server::parse_command(line);
        assert_eq!(Some(("GET".to_string(), vec!["mykey".to_string()])), result);
    }

    #[test]
    fn parse_command_handles_mixed_case_commands() {
        let line = "GeT mykey\n";
        let result = Server::parse_command(line);
        assert_eq!(Some(("GET".to_string(), vec!["mykey".to_string()])), result);
    }

    #[test]
    fn parse_command_handles_extra_whitespace() {
        let line = "  SET   mykey   myvalue  \n";
        let result = Server::parse_command(line);
        assert_eq!(
            Some((
                "SET".to_string(),
                vec!["mykey".to_string(), "myvalue".to_string()]
            )),
            result
        );
    }

    #[test]
    fn parse_command_returns_none_for_empty_line() {
        let line = "\n";
        let result = Server::parse_command(line);
        assert_eq!(None, result);
    }

    #[test]
    fn parse_command_returns_none_for_whitespace_only() {
        let line = "   \n";
        let result = Server::parse_command(line);
        assert_eq!(None, result);
    }

    #[test]
    fn handle_command_get_returns_value_when_key_exists() {
        let store = Arc::new(KVStore::new());
        store.set("testkey", "testvalue").unwrap();

        let response = Server::handle_command("GET", vec!["testkey".to_string()], &store);
        assert_eq!("testvalue", response.unwrap());
    }

    #[test]
    fn handle_command_get_returns_nil_when_key_does_not_exist() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("GET", vec!["nonexistent".to_string()], &store);
        assert_eq!("nil", response.unwrap());
    }

    #[test]
    fn handle_command_get_returns_error_with_no_arguments() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("GET", vec![], &store);
        assert!(response.is_err());
    }

    #[test]
    fn handle_command_set_stores_value_and_returns_ok() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command(
            "SET",
            vec!["testkey".to_string(), "testvalue".to_string()],
            &store,
        );
        assert_eq!("OK", response.unwrap());
        assert_eq!(Some("testvalue".to_string()), store.get("testkey").unwrap());
    }

    #[test]
    fn handle_command_set_overwrites_existing_value() {
        let store = Arc::new(KVStore::new());
        store.set("testkey", "oldvalue").unwrap();

        let response = Server::handle_command(
            "SET",
            vec!["testkey".to_string(), "newvalue".to_string()],
            &store,
        );
        assert_eq!("OK", response.unwrap());
        assert_eq!(Some("newvalue".to_string()), store.get("testkey").unwrap());
    }

    #[test]
    fn handle_command_set_returns_error_with_no_value() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("SET", vec!["testkey".to_string()], &store);

        assert!(response.is_err());
        assert_eq!(
            MiniRedisError::InvalidArguments {
                arguments: vec!["testkey".to_string()]
            },
            response.unwrap_err()
        );
    }

    #[test]
    fn handle_command_set_returns_error_with_no_arguments() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("SET", vec![], &store);

        assert!(response.is_err());
        assert_eq!(
            MiniRedisError::InvalidArguments { arguments: vec![] },
            response.unwrap_err()
        );
    }

    #[test]
    fn handle_command_del_removes_key_and_returns_ok() {
        let store = Arc::new(KVStore::new());
        store.set("testkey", "testvalue").unwrap();

        let response = Server::handle_command("DEL", vec!["testkey".to_string()], &store);

        assert_eq!("OK", response.unwrap());
        assert_eq!(None, store.get("testkey").unwrap());
    }

    #[test]
    fn handle_command_del_returns_ok_even_if_key_does_not_exist() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("DEL", vec!["nonexistent".to_string()], &store);

        assert_eq!("OK", response.unwrap());
    }

    #[test]
    fn handle_command_del_returns_error_with_no_arguments() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("DEL", vec![], &store);

        assert!(response.is_err());
        assert_eq!(
            MiniRedisError::InvalidArguments { arguments: vec![] },
            response.unwrap_err()
        );
    }

    #[test]
    fn handle_command_returns_error_for_unknown_command() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("UNKNOWN", vec!["arg".to_string()], &store);

        assert!(response.is_err());
        assert_eq!(
            MiniRedisError::InvalidCommand {
                command: "UNKNOWN".to_string()
            },
            response.unwrap_err()
        );
    }

    #[test]
    fn handle_command_returns_error_for_extra_arguments() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command(
            "GET",
            vec!["testkey".to_string(), "extra".to_string()],
            &store,
        );

        assert!(response.is_err());
        assert_eq!(
            MiniRedisError::InvalidArguments {
                arguments: vec!["testkey".to_string(), "extra".to_string()]
            },
            response.unwrap_err()
        );

        let response = Server::handle_command(
            "SET",
            vec![
                "testkey".to_string(),
                "testvalue".to_string(),
                "extra".to_string(),
            ],
            &store,
        );
        assert!(response.is_err());
        assert_eq!(
            MiniRedisError::InvalidArguments {
                arguments: vec![
                    "testkey".to_string(),
                    "testvalue".to_string(),
                    "extra".to_string()
                ]
            },
            response.unwrap_err()
        );

        let response = Server::handle_command(
            "DEL",
            vec!["testkey".to_string(), "extra".to_string()],
            &store,
        );
        assert!(response.is_err());
        assert_eq!(
            MiniRedisError::InvalidArguments {
                arguments: vec!["testkey".to_string(), "extra".to_string()]
            },
            response.unwrap_err()
        );
    }
}
