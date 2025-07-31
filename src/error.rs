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
    /// The stream is not accepted.
    AddressNotBound,

    /// The stream is not flushed.
    StreamNotFlushed,
}