use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::thread;
use std::time::Duration;
use miniredis::server::Server;

/// Helper function to find an available port
fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a port");
    let port = listener
        .local_addr()
        .expect("Failed to get local address")
        .port();
    drop(listener); // Close the listener to free the port
    port
}

/// Helper function to start a test server on a random available port
pub fn start_test_server() -> String {
    let port = find_available_port();
    let address = format!("127.0.0.1:{}", port);
    let server_address = address.clone();

    thread::spawn(move || {
        let server = Server::new(&server_address);
        let _ = server.run();
    });

    // Give the server a moment to start up
    thread::sleep(Duration::from_millis(100));

    // Verify server is actually listening
    for _ in 0..10 {
        if TcpStream::connect(&address).is_ok() {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    address
}

/// Helper function to send a command to the server and get the response
pub fn send_command(address: &str, command: &str) -> Result<String, std::io::Error> {
    let mut stream = TcpStream::connect(address)?;
    let mut reader = BufReader::new(stream.try_clone()?);

    // Send command
    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;

    // Read response
    let mut response = String::new();
    reader.read_line(&mut response)?;

    // Remove trailing newline
    if response.ends_with('\n') {
        response.pop();
    }

    Ok(response)
}