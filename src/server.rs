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

    /// Runs the server.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
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
    fn handle_client(mut stream: TcpStream, store: Arc<KVStore>) {
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let mut line = String::new();

        loop {
            line.clear();
            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }

            let (command, args) = match Self::parse_command(&line) {
                Some((command, args)) => (command, args),
                None => continue,
            };

            let response = Self::handle_command(&command, args, &store);

            stream.write_all(response.as_bytes()).unwrap();
            stream.write_all(b"\n").unwrap();
        }
    }

    /// Parses a command from a stream.
    ///
    /// # Arguments
    ///
    /// * `line` - The line to read the command from.
    ///
    /// # Returns
    ///
    /// A optional tuple containing the command and its arguments. If the command is empty or the line is empty, None is returned.
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
        assert!(server.store.get("nonexistent_key").is_none());
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
        store.set("testkey", "testvalue");

        let response = Server::handle_command("GET", vec!["testkey".to_string()], &store);
        assert_eq!("testvalue", response);
    }

    #[test]
    fn handle_command_get_returns_nil_when_key_does_not_exist() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("GET", vec!["nonexistent".to_string()], &store);
        assert_eq!("nil", response);
    }

    #[test]
    fn handle_command_get_returns_error_with_no_arguments() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("GET", vec![], &store);
        assert_eq!("ERR wrong number of arguments", response);
    }

    #[test]
    fn handle_command_set_stores_value_and_returns_ok() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command(
            "SET",
            vec!["testkey".to_string(), "testvalue".to_string()],
            &store,
        );
        assert_eq!("OK", response);
        assert_eq!(Some("testvalue".to_string()), store.get("testkey"));
    }

    #[test]
    fn handle_command_set_overwrites_existing_value() {
        let store = Arc::new(KVStore::new());
        store.set("testkey", "oldvalue");

        let response = Server::handle_command(
            "SET",
            vec!["testkey".to_string(), "newvalue".to_string()],
            &store,
        );
        assert_eq!("OK", response);
        assert_eq!(Some("newvalue".to_string()), store.get("testkey"));
    }

    #[test]
    fn handle_command_set_returns_error_with_no_value() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("SET", vec!["testkey".to_string()], &store);
        assert_eq!("ERR wrong number of arguments for 'set' command", response);
    }

    #[test]
    fn handle_command_set_returns_error_with_no_arguments() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("SET", vec![], &store);
        assert_eq!("ERR wrong number of arguments", response);
    }

    #[test]
    fn handle_command_del_removes_key_and_returns_ok() {
        let store = Arc::new(KVStore::new());
        store.set("testkey", "testvalue");

        let response = Server::handle_command("DEL", vec!["testkey".to_string()], &store);
        assert_eq!("OK", response);
        assert_eq!(None, store.get("testkey"));
    }

    #[test]
    fn handle_command_del_returns_ok_even_if_key_does_not_exist() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("DEL", vec!["nonexistent".to_string()], &store);
        assert_eq!("OK", response);
    }

    #[test]
    fn handle_command_del_returns_error_with_no_arguments() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("DEL", vec![], &store);
        assert_eq!("ERR wrong number of arguments", response);
    }

    #[test]
    fn handle_command_returns_error_for_unknown_command() {
        let store = Arc::new(KVStore::new());

        let response = Server::handle_command("UNKNOWN", vec!["arg".to_string()], &store);
        assert_eq!("ERR unknown command", response);
    }
}
