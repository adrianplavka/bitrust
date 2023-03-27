//! Bencode tokens & charsets.

pub const INTEGER_START: u8 = b'i';
pub const LIST_START: u8 = b'l';
pub const MAP_START: u8 = b'd';
pub const BYTES_DELIMITER: u8 = b':';
pub const END: u8 = b'e';

pub const SIGNED_NUMBER_CHARSET: &[u8; 11] = b"-1234567890";
pub const UNSIGNED_NUMBER_CHARSET: &[u8; 10] = b"1234567890";
