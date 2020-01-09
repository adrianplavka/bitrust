//! Bencode errors and result type for serialization & deserialization.

use std::error;
use std::fmt::{self, Debug, Display};
use std::io;
use std::result;

use serde::de;
use serde::ser;

/// This type represents all possible errors that can occur during Bencode
/// serialization & deserialization.
#[derive(PartialEq)]
pub enum Error {
    /// Catch-all for deserialization & serialization error messages.
    Message(Box<str>),

    /// ExpectedInteger occurs, when a signed integer was expected at the position
    /// during deserialization.
    ExpectedInteger,

    /// ExpectedUnsignedInteger occurs, when an unsigned integer was expected at
    /// the position during deserialization.
    ExpectedUnsignedInteger,

    /// IntegerOverflow occurs, when an integer overflows during deserialization
    /// of a type smaller than integer input.
    IntegerOverflow,

    /// ExpectedStringIntegerLength occurs, when a length of string has not been
    /// specified, or is of an unappropriate type during deserialization.
    ExpectedStringIntegerLength,

    /// ParseStringIntegerLengthError occurs, when parsing length of string has
    /// failed during deserialization.
    ParseStringIntegerLengthError,

    /// ExpectedList occurs, when a list was expected at the position during
    /// deserialization.
    ExpectedList,

    /// ExpectedListEnd occurs, when a list's end was expected at the position during
    /// deserialization.
    ExpectedListEnd,

    /// ExpectedDictionary occurs, when a dictionary was expected at the position
    /// during deserialization.
    ExpectedDictionary,

    /// ExpectedDictionaryEnd occurs, when a dictionary's end was expected at the
    /// position during deserialization.
    ExpectedDictionaryEnd,

    /// ExpectedDictionaryKeyString occurs, when dictionary's key has not been
    /// specified, or is of an unappropriate type during deserialization.
    ExpectedDictionaryKeyString,

    /// UnknownType occurs, when the data is impossible to infer from during
    /// deserialization.
    UnknownType,

    /// TrailingCharacter occurs, when the input contains additional trailing
    /// characters after deserializing.
    TrailingCharacters,

    /// EOF occurs, when reading from an input hits an unexpected end during
    /// deserialization.
    EOF,
}

/// Typedef for `Result` with an own error implementation.
pub type Result<T> = result::Result<T, Error>;

impl error::Error for Error {}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Message(_)
            | Error::ExpectedInteger
            | Error::ExpectedUnsignedInteger
            | Error::IntegerOverflow
            | Error::ExpectedStringIntegerLength
            | Error::ParseStringIntegerLengthError
            | Error::ExpectedList
            | Error::ExpectedListEnd
            | Error::ExpectedDictionary
            | Error::ExpectedDictionaryEnd
            | Error::ExpectedDictionaryKeyString
            | Error::UnknownType
            | Error::TrailingCharacters => io::Error::new(io::ErrorKind::InvalidData, e),
            Error::EOF => io::Error::new(io::ErrorKind::UnexpectedEof, e),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Message(ref m) => f.write_str(m),
            Error::ExpectedInteger => f.write_str("[bitrust_bencode error]: expected integer"),
            Error::ExpectedUnsignedInteger => {
                f.write_str("[bitrust_bencode error]: expected unsigned integer")
            }
            Error::IntegerOverflow => f.write_str("[bitrust_bencode error]: integer overflow"),
            Error::ExpectedStringIntegerLength => {
                f.write_str("[bitrust_bencode error]: expected string's integer length")
            }
            Error::ParseStringIntegerLengthError => {
                f.write_str("[bitrust_bencode error]: unable to parse string's integer length")
            }
            Error::ExpectedList => f.write_str("[bitrust_bencode error]: expected list"),
            Error::ExpectedListEnd => f.write_str("[bitrust_bencode error]: expected list's end"),
            Error::ExpectedDictionary => {
                f.write_str("[bitrust_bencode error]: expected dictionary")
            }
            Error::ExpectedDictionaryEnd => {
                f.write_str("[bitrust_bencode error]: expected dictionary's end")
            }
            Error::ExpectedDictionaryKeyString => {
                f.write_str("[bitrust_bencode error]: expected dictionary's key string")
            }
            Error::UnknownType => f.write_str("[bitrust_bencode error]: unknown type"),
            Error::TrailingCharacters => {
                f.write_str("[bitrust_bencode error]: trailing characters")
            }
            Error::EOF => f.write_str("[bitrust_bencode error]: unexpected end"),
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
