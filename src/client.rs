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
        todo!("Implement client run");
    }
}