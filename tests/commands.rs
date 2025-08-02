use miniredis::server::Server;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

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
fn start_test_server() -> String {
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
fn send_command(address: &str, command: &str) -> Result<String, std::io::Error> {
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

#[test]
fn get_command_returns_nil_for_non_existing_key() {
    let address = start_test_server();

    let response =
        send_command(&address, "GET nonexistent_key").expect("Failed to send GET command");

    assert_eq!(response, "nil");
}

#[test]
fn set_command_stores_value_and_returns_ok() {
    let address = start_test_server();

    let response =
        send_command(&address, "SET test_key test_value").expect("Failed to send SET command");

    assert_eq!(response, "OK");
}

#[test]
fn get_command_returns_stored_value() {
    let address = start_test_server();

    // First set a value
    let set_response =
        send_command(&address, "SET my_key my_value").expect("Failed to send SET command");
    assert_eq!(set_response, "OK");

    // Then get the value
    let get_response = send_command(&address, "GET my_key").expect("Failed to send GET command");
    assert_eq!(get_response, "my_value");
}

#[test]
fn set_command_overwrites_existing_value() {
    let address = start_test_server();

    // Set initial value
    send_command(&address, "SET overwrite_key initial_value")
        .expect("Failed to send first SET command");

    // Overwrite with new value
    let set_response = send_command(&address, "SET overwrite_key new_value")
        .expect("Failed to send second SET command");
    assert_eq!(set_response, "OK");

    // Verify new value is stored
    let get_response =
        send_command(&address, "GET overwrite_key").expect("Failed to send GET command");
    assert_eq!(get_response, "new_value");
}

#[test]
fn del_command_removes_key_and_returns_ok() {
    let address = start_test_server();

    // First set a value
    send_command(&address, "SET delete_me some_value").expect("Failed to send SET command");

    // Verify it exists
    let get_response = send_command(&address, "GET delete_me").expect("Failed to send GET command");
    assert_eq!(get_response, "some_value");

    // Delete the key
    let del_response = send_command(&address, "DEL delete_me").expect("Failed to send DEL command");
    assert_eq!(del_response, "OK");

    // Verify it no longer exists
    let get_response_after_del =
        send_command(&address, "GET delete_me").expect("Failed to send GET command after deletion");
    assert_eq!(get_response_after_del, "nil");
}

#[test]
fn del_command_returns_ok_for_non_existing_key() {
    let address = start_test_server();

    let response =
        send_command(&address, "DEL non_existing_key").expect("Failed to send DEL command");

    assert_eq!(response, "OK");
}

#[test]
fn commands_are_case_insensitive() {
    let address = start_test_server();

    // Test lowercase commands
    let set_response = send_command(&address, "set case_key case_value")
        .expect("Failed to send lowercase SET command");
    assert_eq!(set_response, "OK");

    let get_response =
        send_command(&address, "get case_key").expect("Failed to send lowercase GET command");
    assert_eq!(get_response, "case_value");

    let del_response =
        send_command(&address, "del case_key").expect("Failed to send lowercase DEL command");
    assert_eq!(del_response, "OK");
}

#[test]
fn invalid_command_returns_error() {
    let address = start_test_server();

    let response =
        send_command(&address, "INVALID_COMMAND some_arg").expect("Failed to send invalid command");

    // The error message should contain information about the invalid command
    assert!(response.contains("Invalid command"));
}

#[test]
fn get_with_wrong_number_of_arguments_returns_error() {
    let address = start_test_server();

    // GET with no arguments
    let response = send_command(&address, "GET").expect("Failed to send GET with no args");
    assert!(response.contains("Invalid arguments"));

    // GET with too many arguments
    let response =
        send_command(&address, "GET key1 key2").expect("Failed to send GET with too many args");
    assert!(response.contains("Invalid arguments"));
}

#[test]
fn set_with_wrong_number_of_arguments_returns_error() {
    let address = start_test_server();

    // SET with no arguments
    let response = send_command(&address, "SET").expect("Failed to send SET with no args");
    assert!(response.contains("Invalid arguments"));

    // SET with only key (missing value)
    let response =
        send_command(&address, "SET only_key").expect("Failed to send SET with only key");
    assert!(response.contains("Invalid arguments"));

    // SET with too many arguments
    let response = send_command(&address, "SET key value extra")
        .expect("Failed to send SET with too many args");
    assert!(response.contains("Invalid arguments"));
}

#[test]
fn del_with_wrong_number_of_arguments_returns_error() {
    let address = start_test_server();

    // DEL with no arguments
    let response = send_command(&address, "DEL").expect("Failed to send DEL with no args");
    assert!(response.contains("Invalid arguments"));

    // DEL with too many arguments
    let response =
        send_command(&address, "DEL key1 key2").expect("Failed to send DEL with too many args");
    assert!(response.contains("Invalid arguments"));
}

#[test]
fn server_handles_commands_with_extra_whitespace() {
    let address = start_test_server();

    // Test commands with extra spaces
    let response = send_command(&address, "  SET   space_key   space_value  ")
        .expect("Failed to send SET with extra spaces");
    assert_eq!(response, "OK");

    let response = send_command(&address, "  GET   space_key  ")
        .expect("Failed to send GET with extra spaces");
    assert_eq!(response, "space_value");
}
