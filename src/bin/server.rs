use miniredis::server::Server;
use std::env;

/// Runs the server.
///
/// Run gets the environment variables, checks if the user wants to see the help message,
/// and then creates a server from the arguments and runs it.
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        Server::print_help();
        return;
    }

    let server = Server::from_args(&args);

    if let Err(e) = server.run() {
        eprintln!("Server failed: {}", e);
        std::process::exit(1);
    }
}
