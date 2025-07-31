/// An error that can occur in the MiniRedis library.
#[derive(Debug, PartialEq)]
pub enum MiniRedisError {
    /// The key value store is locked.
    StoreLocked,

    /// The command is invalid.
    InvalidCommand,
    /// The arguments are invalid.
    InvalidArguments,

    /// The stream is closed.
    StreamClosed,
    /// The stream is not readable.
    StreamNotReadable,
    /// The stream is not writable.
    StreamNotWritable,
    /// The stream is not connected.
    StreamNotConnected,
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
            MiniRedisError::StoreLocked => write!(f, "The key value store is locked."),
            MiniRedisError::InvalidCommand => write!(f, "The command is invalid."),
            MiniRedisError::InvalidArguments => write!(f, "The arguments are invalid."),
            MiniRedisError::StreamClosed => write!(f, "The stream is closed."),
            MiniRedisError::StreamNotReadable => write!(f, "The stream is not readable."),
            MiniRedisError::StreamNotWritable => write!(f, "The stream is not writable."),
            MiniRedisError::StreamNotConnected => write!(f, "The stream is not connected."),
            MiniRedisError::AddressNotBound => write!(f, "The address is not bound."),
            MiniRedisError::StreamNotFlushed => write!(f, "The stream is not flushed."),
        }
    }
}