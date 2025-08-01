use crate::error::MiniRedisError;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

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
        Self {
            address: address.to_string(),
        }
    }

    /// Creates a new client from command line arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - The command line arguments.
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
    /// let client = Client::from_args(&["miniredis".to_string(), "127.0.0.1:6379".to_string()]);
    /// client.run();
    /// ```
    pub fn from_args(args: &[String]) -> Self {
        let address = if args.len() > 1 {
            &args[1]
        } else {
            "127.0.0.1:6379"
        };

        Self::new(address)
    }

    /// Runs the client.
    ///
    /// Run starts the client and connects to the server.
    /// It will then enter a loop where it reads input from the user,
    /// sends it to the server, and prints the response.
    ///
    /// # Returns
    ///
    /// A result indicating whether the client was run successfully.
    ///
    /// # Errors
    ///
    /// If the client fails to connect to the server,
    /// read from the stream, or write to the stream, it will return an error.
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
        let mut stream =
            TcpStream::connect(&self.address).map_err(|_| MiniRedisError::StreamNotConnected {
                address: self.address.clone(),
            })?;
        let mut reader = BufReader::new(
            stream
                .try_clone()
                .map_err(|_| MiniRedisError::StreamClosed)?,
        );
        let mut terminal_reader = BufReader::new(io::stdin());

        println!("Connected to server at {}", self.address);

        loop {
            print!("> ");
            io::stdout()
                .flush()
                .map_err(|_| MiniRedisError::StreamNotFlushed)?;

            let input = self.read_input(&mut terminal_reader)?;

            if input.is_empty() {
                continue;
            }

            if input == "quit" {
                break;
            }

            self.send_input(&input, &mut stream)?;

            let response = self.read_response(&mut reader)?;

            println!("{}", response);
        }

        Ok(())
    }

    /// Prints the help message.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use miniredis::client::Client;
    ///
    /// Client::print_help();
    /// ```
    pub fn print_help() {
        println!("MiniRedis Client");
        println!();
        println!("Connects to a MiniRedis server and sends commands to it.");
        println!();
        println!("USAGE:");
        println!("    miniredis client <ADDRESS>");
        println!();
        println!("ARGS:");
        println!(
            "    <ADDRESS>    The address of the server to connect to [default: 127.0.0.1:6379]"
        );
        println!();
        println!("EXAMPLES:");
        println!("    miniredis client 127.0.0.1:6379");
        println!("    miniredis client --help");
        println!();
        println!("COMMANDS IN THE CLIENT:");
        println!("    GET <KEY>             Get the value of a key");
        println!("    SET <KEY> <VALUE>     Set the value of a key");
        println!("    DEL <KEY>             Delete a key");
    }

    /// Reads input from the user.
    ///
    /// # Returns
    ///
    /// A string containing the input from the user.
    ///
    /// # Errors
    ///
    /// If the input cannot be read, it will return an error.
    fn read_input<R: BufRead>(&self, reader: &mut R) -> Result<String, MiniRedisError> {
        let mut input = String::new();
        reader
            .read_line(&mut input)
            .map_err(|_| MiniRedisError::StreamNotReadable)?;
        Ok(input)
    }

    /// Sends input to the server.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to send to the server.
    /// * `writer` - The writer to send the input to.
    ///
    /// # Returns
    ///
    /// A result indicating whether the input was sent successfully.
    ///
    /// # Errors
    ///
    /// If the input cannot be written to the writer, it will return an error.
    fn send_input<W: Write>(&self, input: &str, writer: &mut W) -> Result<(), MiniRedisError> {
        writer
            .write_all(input.as_bytes())
            .map_err(|_| MiniRedisError::StreamNotWritable)?;
        writer
            .write_all(b"\n")
            .map_err(|_| MiniRedisError::StreamNotWritable)?;
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
    ///
    /// # Errors
    ///
    /// If the response cannot be read, it will return an error.
    fn read_response<R: BufRead>(&self, reader: &mut R) -> Result<String, MiniRedisError> {
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(|_| MiniRedisError::StreamNotReadable)?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_client_with_given_address() {
        let address = "192.168.1.1:8080";
        let client = Client::new(address);

        assert_eq!(address.to_string(), client.address);
    }

    #[test]
    fn from_args_uses_default_address_when_no_args_provided() {
        let args = vec!["miniredis".to_string()];
        let client = Client::from_args(&args);

        assert_eq!("127.0.0.1:6379".to_string(), client.address);
    }

    #[test]
    fn from_args_uses_provided_address_when_args_given() {
        let expected_address = "localhost:9999";
        let args = vec!["miniredis".to_string(), expected_address.to_string()];
        let client = Client::from_args(&args);

        assert_eq!(expected_address.to_string(), client.address);
    }

    #[test]
    fn from_args_uses_first_argument_as_address() {
        let expected_address = "test.example.com:1234";
        let args = vec![
            "miniredis".to_string(),
            expected_address.to_string(),
            "ignored_arg".to_string(),
        ];
        let client = Client::from_args(&args);

        assert_eq!(expected_address.to_string(), client.address);
    }

    #[test]
    fn read_input_reads_line_from_reader() {
        use std::io::Cursor;

        let client = Client::new("127.0.0.1:6379");
        let input_data = "test input\n";
        let cursor = Cursor::new(input_data.as_bytes());
        let mut reader = BufReader::new(cursor);

        let result = client.read_input(&mut reader).unwrap();

        assert_eq!("test input\n".to_string(), result);
    }

    #[test]
    fn send_input_writes_input_with_newline() {
        let client = Client::new("127.0.0.1:6379");
        let mut output = Vec::new();
        let input = "SET key value";

        client.send_input(input, &mut output).unwrap();

        assert_eq!("SET key value\n".as_bytes(), output.as_slice());
    }

    #[test]
    fn send_input_handles_empty_input() {
        let client = Client::new("127.0.0.1:6379");
        let mut output = Vec::new();
        let input = "";

        client.send_input(input, &mut output).unwrap();

        assert_eq!("\n".as_bytes(), output.as_slice());
    }

    #[test]
    fn read_response_reads_line_from_reader() {
        use std::io::Cursor;

        let client = Client::new("127.0.0.1:6379");
        let response_data = "OK\n";
        let cursor = Cursor::new(response_data.as_bytes());
        let mut reader = BufReader::new(cursor);

        let result = client.read_response(&mut reader).unwrap();

        assert_eq!("OK\n".to_string(), result);
    }

    #[test]
    fn read_response_handles_multiline_response() {
        use std::io::Cursor;

        let client = Client::new("127.0.0.1:6379");
        let response_data = "value with spaces\nsecond line\n";
        let cursor = Cursor::new(response_data.as_bytes());
        let mut reader = BufReader::new(cursor);

        let result = client.read_response(&mut reader).unwrap();

        assert_eq!("value with spaces\n".to_string(), result);
    }
}