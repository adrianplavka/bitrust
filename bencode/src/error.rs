
//! Bencode errors during encoding & decoding, and serialization & deserialization.

use std::error;
use std::fmt::{self, Debug, Display};
use std::io;
use std::result;

/// This type represents all possible errors that can occur.
pub struct Error {
    pub code: ErrorCode
}

impl Error {
    pub fn new(code: ErrorCode) -> Self {
        Error {
            code: code
        }
    }
}

/// Alias for `Result` with an own error implementation.
pub type Result<T> = result::Result<T, Error>;

#[doc(hidden)]
#[derive(Debug, PartialEq)]
pub enum ErrorCode {
    /// ParseError occurs, when a match case hadn't been covered in a parsing stage.
    ParseError,

    /// DataError occurs, when data are semantically incorrect.
    DataError,

    /// UnexpectedSymbol occurs, when a specific symbol hasn't been expected.
    UnexpectedSymbol,

    /// EOF occurs, when reading from a stream of bytes hits an unexpected end.
    EOF
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "bencode error"
    }
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        match e.code {
            ErrorCode::DataError 
            | ErrorCode::ParseError
            | ErrorCode::UnexpectedSymbol => io::Error::new(io::ErrorKind::InvalidData, e),
            ErrorCode::EOF => io::Error::new(io::ErrorKind::UnexpectedEof, e)
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::ParseError => f.write_str("parsing error occured"),
            ErrorCode::DataError => f.write_str("data error occured"),
            ErrorCode::UnexpectedSymbol => f.write_str("unexpected symbol occured"),
            ErrorCode::EOF => f.write_str("unexpected end occured")
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.code, f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error({:?})",
            self.code.to_string()
        )
    }
}
