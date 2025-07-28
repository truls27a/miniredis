use crate::kv_store::KVStore;
use std::sync::Arc;

/// A server that listens for client connections and handles requests.
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
        Self { address: address.to_string(), store: Arc::new(KVStore::new()) }
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
        todo!("Implement server run");
    }
}
