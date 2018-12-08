
//! Bencode errors during encoding & decoding, and serialization & deserialization.

use std::error;
use std::fmt::{self, Debug, Display};
use std::io;
use std::result;

use serde::de;

/// This type represents all possible errors that can occur.
#[derive(PartialEq)]
pub enum Error {
    /// Catchall for deserialization & serialization error messages.
    Message(Box<str>),

    /// ParseError occurs, when a match case hadn't been covered in a parsing stage.
    ParseError,

    /// DataError occurs, when data are semantically incorrect.
    DataError,

    // NonExistingType occurs, when the data are impossible to infer from.
    NonExistingType,

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
            Error::Message(_)
            | Error::ParseError
            | Error::DataError
            | Error::NonExistingType
            | Error::UnexpectedSymbol
            | Error::NonStringKey => io::Error::new(io::ErrorKind::InvalidData, e),
            Error::EOF => io::Error::new(io::ErrorKind::UnexpectedEof, e)
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Message(ref m) => f.write_str(m),
            Error::ParseError => f.write_str("parsing error occured"),
            Error::DataError => f.write_str("data error occured"),
            Error::NonExistingType => f.write_str("non existing type occured"),
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

impl de::Error for Error {
    #[cold]
    fn custom<T: Display>(msg: T) -> Error {
        Error::Message(msg.to_string().into_boxed_str())
    }

    #[cold]
    fn invalid_type(unexp: de::Unexpected, exp: &de::Expected) -> Self {
        if let de::Unexpected::Unit = unexp {
            Error::custom(format_args!("invalid type: null, expected {}", exp))
        } else {
            Error::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
        }
    }
}
