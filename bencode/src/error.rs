
//! Bencode errors during encoding & decoding, and serialization & deserialization.

use std::error;
use std::fmt::{self, Debug, Display};
use std::io;
use std::result;

/// This type represents all possible errors that can occur.
#[derive(PartialEq)]
pub enum Error {
    /// ParseError occurs, when a match case hadn't been covered in a parsing stage.
    ParseError,

    /// DataError occurs, when data are semantically incorrect.
    DataError,

    /// UnexpectedSymbol occurs, when a specific symbol hadn't been expected.
    UnexpectedSymbol,

    /// NonStringKey occurs, when a key in a dictionary is not a string.
    NonStringKey,

    /// EOF occurs, when reading from a stream of bytes hits an unexpected end.
    EOF
}

/// Alias for `Result` with an own error implementation.
pub type Result<T> = result::Result<T, Error>;

impl error::Error for Error {
    fn description(&self) -> &str {
        "bencode error"
    }
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::DataError 
            | Error::ParseError
            | Error::UnexpectedSymbol
            | Error::NonStringKey => io::Error::new(io::ErrorKind::InvalidData, e),
            Error::EOF => io::Error::new(io::ErrorKind::UnexpectedEof, e)
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError => f.write_str("parsing error occured"),
            Error::DataError => f.write_str("data error occured"),
            Error::UnexpectedSymbol => f.write_str("unexpected symbol occured"),
            Error::NonStringKey => f.write_str("non string key in a dictionary occured"),
            Error::EOF => f.write_str("unexpected end occured")
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error({:?})",
            self.to_string()
        )
    }
}
