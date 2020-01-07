//! Bencode errors during encoding & decoding, and serialization & deserialization.

use serde::de;
use serde::ser;
use std::error;
use std::fmt::{self, Debug, Display};
use std::io;
use std::result;

/// This type represents all possible errors that can occur during serialization & deserialization.
#[derive(PartialEq)]
pub enum Error {
    /// Catchall for deserialization & serialization error messages.
    Message(Box<str>),

    /// ExpectedInteger occurs, when an unsigned integer was expected during serialization or deserialization.
    ExpectedInteger,

    ExpectedStringIntegerLength,

    ExpectedListEnd,

    ExpectedMapEnd,

    ExpectedMapKeyString,

    /// ParseError occurs, when a match case hadn't been covered in a parsing stage.
    ParseError,

    ParseStringIntegerLengthError,

    /// DataError occurs, when data are semantically incorrect.
    DataError,

    // NonExistingType occurs, when the data is impossible to infer from.
    NonExistingType,

    /// UnexpectedSymbol occurs, when a specific symbol hadn't been expected.
    UnexpectedSymbol,

    /// NonStringKey occurs, when a key in a dictionary is not a string.
    NonStringKey,

    /// TrailingCharacter occurs, when the input in deserializing contains additional trailing characters.
    TrailingCharacters,

    /// EOF occurs, when reading from a stream of bytes hits an unexpected end.
    EOF,
}

/// Typedef for `Result` with an own error implementation.
pub type Result<T> = result::Result<T, Error>;

impl error::Error for Error {
    fn description(&self) -> &str {
        "bitrust_bencode error"
    }
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Message(_)
            | Error::ExpectedInteger
            | Error::ParseError
            | Error::DataError
            | Error::ExpectedStringIntegerLength
            | Error::ParseStringIntegerLengthError
            | Error::ExpectedListEnd
            | Error::ExpectedMapEnd
            | Error::ExpectedMapKeyString
            | Error::NonExistingType
            | Error::UnexpectedSymbol
            | Error::TrailingCharacters
            | Error::NonStringKey => io::Error::new(io::ErrorKind::InvalidData, e),
            Error::EOF => io::Error::new(io::ErrorKind::UnexpectedEof, e),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Message(ref m) => f.write_str(m),
            Error::ExpectedInteger => f.write_str("expected integer error occured"),
            Error::ParseError => f.write_str("parsing error occured"),
            Error::DataError => f.write_str("data error occured"),
            Error::ExpectedStringIntegerLength => f.write_str("expected string integer length"),
            Error::ParseStringIntegerLengthError => {
                f.write_str("parse of string integer length error")
            }
            Error::ExpectedListEnd => f.write_str("expected list end"),
            Error::ExpectedMapEnd => f.write_str("expected map end"),
            Error::ExpectedMapKeyString => f.write_str("expected map key string"),
            Error::NonExistingType => f.write_str("non existing type occured"),
            Error::UnexpectedSymbol => f.write_str("unexpected symbol occured"),
            Error::NonStringKey => f.write_str("non string key in a dictionary occured"),
            Error::TrailingCharacters => f.write_str("trailing characters occurder"),
            Error::EOF => f.write_str("unexpected end occured"),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error({:?})", self.to_string())
    }
}

impl de::Error for Error {
    #[cold]
    fn custom<T: Display>(msg: T) -> Error {
        Error::Message(msg.to_string().into_boxed_str())
    }

    #[cold]
    fn invalid_type(unexp: de::Unexpected, exp: &dyn de::Expected) -> Self {
        if let de::Unexpected::Unit = unexp {
            Error::custom(format_args!("invalid type: null, expected {}", exp))
        } else {
            Error::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
        }
    }
}

impl ser::Error for Error {
    #[cold]
    fn custom<T: Display>(msg: T) -> Error {
        Error::Message(msg.to_string().into_boxed_str())
    }
}
