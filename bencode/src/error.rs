//! Bencode errors and result type for serialization & deserialization.

use std::fmt::Display;

use serde::{de, ser};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// Catch-all for deserialization & serialization error messages.
    #[error("{0}")]
    Message(Box<str>),

    /// ExpectedInteger occurs, when a signed integer was expected at the position
    /// during deserialization.
    #[error("Expected integer")]
    ExpectedInteger,

    /// ExpectedSignedNumber occurs, when a signed integer was expected at
    /// the position during deserialization.
    #[error("Expected signed number")]
    ExpectedSignedNumber,

    /// ExpectedUnsignedNumber occurs, when an unsigned integer was expected at
    /// the position during deserialization.
    #[error("Expected unsigned number")]
    ExpectedUnsignedNumber,

    /// ExpectedIntegerEnd occurs, when an integer's end was expected at the position
    /// during deserialization.
    #[error("Expected integer end")]
    ExpectedIntegerEnd,

    /// ExpectedStringIntegerLength occurs, when a length of string has not been
    /// specified, or is of an unappropriate type during deserialization.
    #[error("Expected string integer length")]
    ExpectedStringIntegerLength,

    /// InvalidUTF8 occurs, when parsing of a string has failed during
    /// serialization or deserialization (it is not in UTF-8).
    #[error("Invalid UTF-8")]
    InvalidUTF8,

    /// IntegerOverflow occurs, when an integer overflows during deserialization
    /// of a type smaller than integer input.
    #[error{"Integer overflow"}]
    IntegerOverflow,

    /// ExpectedFloat occurs, when parsing string to float has failed during
    /// deserialization.
    #[error("Expected float")]
    ExpectedFloat,

    /// ExpectedList occurs, when a list was expected at the position during
    /// deserialization.
    #[error("Expected list")]
    ExpectedList,

    /// ExpectedListEnd occurs, when a list's end was expected at the position during
    /// deserialization.
    #[error("Expected list end")]
    ExpectedListEnd,

    /// ExpectedDictionary occurs, when a dictionary was expected at the position
    /// during deserialization.
    #[error("Expected dictionary")]
    ExpectedDictionary,

    /// ExpectedDictionaryEnd occurs, when a dictionary's end was expected at the
    /// position during deserialization.
    #[error("Expected dictionary length")]
    ExpectedDictionaryEnd,

    /// ExpectedDictionaryKeyString occurs, when dictionary's key has not been
    /// specified, or is of an unappropriate type during deserialization.
    #[error("Expected dictionary key")]
    ExpectedDictionaryKeyString,

    /// UnknownType occurs, when the data is impossible to infer from during
    /// deserialization.
    #[error("Unknown type")]
    UnknownType,

    /// TrailingCharacter occurs, when the input contains additional trailing
    /// characters after deserializing.
    #[error("Trailing characters")]
    TrailingCharacters,

    /// EOF occurs, when reading from an input hits an unexpected end during
    /// deserialization.
    #[error("Unexpected EOF")]
    EOF,

    /// IO occurs, when caused by a failure to read or write bytes on an IO
    /// stream.
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl de::Error for Error {
    #[cold]
    fn custom<T: Display>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string().into_boxed_str())
    }

    #[cold]
    fn invalid_type(unexp: de::Unexpected, exp: &dyn de::Expected) -> Self {
        if let de::Unexpected::Unit = unexp {
            Error::custom(format_args!("invalid_type: null, expected: {}", exp))
        } else {
            Error::custom(format_args!("invalid type: {}, expected: {}", unexp, exp))
        }
    }
}

impl ser::Error for Error {
    #[cold]
    fn custom<T: Display>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string().into_boxed_str())
    }
}
