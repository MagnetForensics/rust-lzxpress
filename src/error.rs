use std::io::Error as IoError;

/// An error produced by an operation on LZXpress data
#[derive(Debug)]
pub enum LzxpressError {
    /// Failed Memory Allocation
    Mem,
    /// Memory limit would be violated
    MemLimit,
    /// XZ magic bytes weren't found
    Format,
    /// Unsupported compression options
    Options,
    /// Corrupt data
    Data,
    /// Data looks truncated
    Buf,
    /// std::io::Error
    Io(IoError),
    /// An unknown error
    Other,
}