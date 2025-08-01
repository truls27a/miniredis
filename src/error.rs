/// An error that can occur in the MiniRedis library.
#[derive(Debug, PartialEq)]
pub enum MiniRedisError {
    /// The key value store is locked.
    StoreLocked,

    /// The command is invalid.
    InvalidCommand{command: String},
    /// The arguments are invalid.
    InvalidArguments{arguments: Vec<String>},

    /// The stream is closed.
    StreamClosed,
    /// The stream is not readable.
    StreamNotReadable,
    /// The stream is not writable.
    StreamNotWritable,
    /// The stream is not connected.
    StreamNotConnected{address: String},
    /// The stream is not flushed.
    StreamNotFlushed,

    /// The stream is not accepted.
    AddressNotBound,
}

impl std::fmt::Display for MiniRedisError {
    /// Formats the error as a string.
    /// 
    /// # Arguments
    /// 
    /// * `f` - The formatter to write the error to.
    /// 
    /// # Errors
    /// 
    /// If the error cannot be formatted, it will return an error.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiniRedisError::StoreLocked => write!(f, "Could not access the key value store as it is locked."),
            MiniRedisError::InvalidCommand{command} => write!(f, "Invalid command: {}. Run 'miniredis --help' for more information.", command),
            MiniRedisError::InvalidArguments{arguments} => write!(f, "Invalid arguments: {:?}. Run 'miniredis --help' for more information.", arguments),
            MiniRedisError::StreamClosed => write!(f, "The stream is closed."),
            MiniRedisError::StreamNotReadable => write!(f, "Could not read from the stream."),
            MiniRedisError::StreamNotWritable => write!(f, "Could not write to the stream."),
            MiniRedisError::StreamNotConnected{address} => write!(f, "Could not connect to the stream at {}.", address),
            MiniRedisError::AddressNotBound => write!(f, "Could not bind to the address."),
            MiniRedisError::StreamNotFlushed => write!(f, "Could not flush the stream."),
        }
    }
}