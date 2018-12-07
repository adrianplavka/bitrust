
//! Bencode decoder.

use std::io::{Cursor, Read, BufRead};
use std::collections::{BTreeMap};
use std::convert::{From};
use crate::error::{Error, Result};

/// Decode is a function, that will decode a slice of string to a bencode value.
/// It will infer the type of the value, based on the first byte it looks at.
///
/// For example, consider this happy path scenario:
/// ```rust
/// use bitrust_bencode::decode;
///
/// let data = "li32ei8e5:helloe";
/// decode(&data);
/// // -> Value::List([32, 8, "hello"])
/// ```
/// This automatically inferred the inputted type as a list, with it's mixed contents.
///
/// However, note that only the first type in the data will be inferred!
/// Consider this edge case scenario:
/// ```rust
/// use bitrust_bencode::decode;
///
/// let data = "i32eli0ee";
/// decode(&data);
/// // -> Value::Int(32)
/// ```
/// This example included two types - integer and a list afterwards.
///
/// As you can see, the data starts with a type integer, that is not interroperable
/// and cannot contain other types (as lists or dictionary can).
///
/// Therefore, the result will always return the first type it can match to.
pub fn decode(data: &str) -> Result<Value> {
    Ok(Decoder::new(&data).decode())?
}

pub fn decode_from<R: std::io::Read + std::io::Seek>(reader: R) -> Result<Value> {
    unimplemented!()
}

#[derive(Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Bytes(Vec<u8>);

impl<'a> From<&'a str> for Bytes {
    fn from(value: &'a str) -> Self {
        Bytes(value.as_bytes().to_vec())
    }
}

/// Value is an enum, holding a decoded bencode value.
/// The value can be of multiple types:
///     - integer
///     - string
///     - list
///     - dictionary
///
/// Integer is implemented by i64.
///
/// String is implemented by a custom struct Bytes, which holds a Vec of bytes.
///
/// List is implemented by a Vec of bencode values.
///
/// Dictionary is implemented by a std::collections::BTreeMap,
/// because the keys have to be sorted as raw strings (not alphanumeric).
/// It's key is represented by series of bytes & the bencode value.
#[derive(Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Str(Bytes),
    List(Vec<Value>),
    Dict(BTreeMap<Bytes, Value>),
    None
}

/// Decoder is a main struct to decode from bencode data into actual values.
/// It is implemented by a std::io::Cursor, which holds data to a byte slice.
///
/// To use this struct, create a Decoder with "new" function, which converts a string
/// slice to a bytes slice.
/// After that, the implementation consists of reading, advancing or peeking into the
/// bytes slice, which holds the data.
/// Decoding of values happen by correctly matching the BitTorrent implementation, which
/// is described @ http://www.bittorrent.org.
#[derive(Debug)]
struct Decoder<'a> {
    data: Cursor<&'a [u8]>
}

impl<'a> Decoder<'a> {
    /// Constructs a new decoder.
    ///
    /// Accepts data as a string slice,
    /// which then converts it to bytes to the underlying cursor.
    pub fn new(data: &str) -> Decoder {
        Decoder{ data: Cursor::new(data.as_bytes()) }
    }

    pub fn decode(&mut self) -> Result<Value> {
        let byte = self.peek_byte()?;
        let value = match byte {
            b'i' => self.decode_int()?,
            b'0'...b'9' => self.decode_str()?,
            b'l' => self.decode_list()?,
            b'd' => self.decode_dict()?,
            _ => { return Err(Error::NonExistingType); }
        };

        Ok(value)
    }

    /// Read and advance from the cursor to the length of a passed buffer
    /// & save the data to it.
    fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        match self.data.read(buf) {
            Ok(n) if n == buf.len() => Ok(()),
            _ => Err(Error::EOF)
        }
    }

    /// Read & advance one byte from the cursor.
    fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read(&mut buf)?;
        Ok(buf[0])
    }

    /// Peeks, without advancing, one byte from the cursor.
    fn peek_byte(&mut self) -> Result<u8> {
        let data = self.read_byte()?;
        let pos = self.data.position();
        self.data.set_position(pos - 1);
        Ok(data)
    }

    /// Reads & advance one byte, with expectation.
    fn expect_byte(&mut self, expect: u8) -> Result<()> {
        let byte = self.read_byte()?;

        if byte == expect {
            Ok(())
        } else {
            Err(Error::UnexpectedSymbol)
        }
    }

    /// Decodes an integer at the cursor's current position.
    /// The position points to the integer delimiter.
    fn decode_int(&mut self) -> Result<Value> {
        // Expect the first byte to represent an 'i' character,
        // then advance to the next byte.
        self.expect_byte(b'i')?;

        // Construct a buffer, from which we will parse the bytes into a number.
        // Keep a flag of the zeroth loop iteration for logic regarding the '0' or '-' character.
        let mut buffer = String::new();
        let mut is_zeroth = true;
        loop {
            let byte = self.read_byte()?;

            match byte {
                // Numbers, besides '0', get pushed to the buffer.
                b'1'...b'9' => buffer.push(byte as char),
                // Character '0' will yield an error, if it happens to be on the beginning,
                // while there are still some numbers left.
                b'0' => {
                    let next = self.peek_byte()?;
                    if next != b'e' && is_zeroth {
                        return Err(Error::DataError);
                    } else {
                        buffer.push(byte as char);
                    }
                },
                // Character '-' will yield an error, if it doesn't appear only at the beginning,
                // or if the next character will be character '0'.
                b'-' => {
                    let next = self.peek_byte()?;
                    if next == b'0' || !is_zeroth {
                        return Err(Error::DataError);
                    } else {
                        buffer.push(byte as char);
                    }
                },
                // Break the loop, if it's the end of integer.
                b'e' => {
                    break;
                },
                // Default case, when something hadn't been covered.
                _ => {
                    return Err(Error::ParseError);
                }
            }

            is_zeroth = false;
        }

        // Parse the buffer into an integer.
        match buffer.parse() {
            Ok(v) => Ok(Value::Int(v)),
            _ => Err(Error::ParseError)
        }
    }

    /// Decodes a string at the cursor's current position.
    /// The position points to the starting length of the string.
    fn decode_str(&mut self) -> Result<Value> {
        // Extract the length of the buffer from the string value.
        let mut buffer_len = String::new();            
        loop {
            let byte = self.read_byte()?;

            match byte {
                // Push any number into the buffer length.
                b'0'...b'9' => buffer_len.push(byte as char),
                // The ending delimiter of the buffer length.
                b':' => { break; },
                // Default case, when something hadn't been covered.
                _ => { return Err(Error::ParseError); }
            }
        }

        // Parse the length of bytes into a number.
        let len: usize;
        match buffer_len.parse::<usize>() {
            Ok(l) => len = l,
            _ => { return Err(Error::ParseError); }
        };

        // Construct a buffer & read until the length of the buffer.
        let mut buffer: Vec<u8> = vec![0u8; len];
        self.read(&mut buffer[..])?;

        Ok(Value::Str(Bytes(buffer.to_vec())))
    }

    /// Decodes a list at the cursor's current position.
    /// The position points to the list delimiter.
    fn decode_list(&mut self) -> Result<Value> {
        // Expect the first byte to represent an 'l' character,
        // then advance to the next byte.
        self.expect_byte(b'l')?;

        // Construct a list, to which we will append new data.
        let mut list: Vec<Value> = Vec::new();
        loop {
            // Do not consume the next byte, but rather look, 
            // which value is currently being looked at.
            let next = self.peek_byte()?;

            let value = match next {
                // If the next byte is an integer delimiter, decode integer.
                b'i' => self.decode_int()?,
                // If the next byte is starting with an integer, decode string.
                b'0'...b'9' => self.decode_str()?,
                // If the next byte is starting with a list delimiter, decode list.
                b'l' => self.decode_list()?,
                // If the next byte is starting with a dictionary delimiter, decode dictionary.
                b'd' => self.decode_dict()?,
                // If the next byte is an end delimiter, advance one byte & break.
                b'e' => { 
                    self.read_byte()?; 
                    break; 
                },
                // Default case, when something hadn't been covered.
                _ => { return Err(Error::ParseError); }
            };

            list.push(value);
        }

        Ok(Value::List(list))
    }

    /// Decodes a dictionary at the cursor's current position.
    /// The position points to the dictionary delimiter.
    fn decode_dict(&mut self) -> Result<Value> {
        // Expect the first byte to represent a 'd' character,
        // then advance to the next byte.
        self.expect_byte(b'd')?;

        // Construct a dictionary, implemented by binary tree map, to which we will
        // append new data.
        let mut dict: BTreeMap<Bytes, Value> = BTreeMap::new();
        loop {
            // Expect a key to be at the first position.
            // The key has to be a string only.
            let next_key = self.peek_byte()?;
            let key = match next_key {
                // If the key starts with numbers, decode a string.
                // Note that the key can't be of a zero length. 
                b'1'...b'9' => self.decode_str()?,
                // If there is an ending delimiter of a dictionary,
                // advance one byte & break.
                b'e' => {
                    self.read_byte()?;
                    break;
                },
                // Default case, when something hadn't been covered.
                _ => { return Err(Error::NonStringKey); }
            };

            // Expect a value to be at the second position.
            // The value can be anything.
            let next_value = self.peek_byte()?;
            let value = match next_value {
                b'i' => self.decode_int()?,
                b'0'...b'9' => self.decode_str()?,
                b'l' => self.decode_list()?,
                b'd' => self.decode_dict()?,
                _ => { return Err(Error::ParseError); }
            };

            // Deconstruct the key from the string value & insert it into the map.
            match key {
                Value::Str(k) => dict.insert(k, value),
                _ => { return Err(Error::ParseError); }
            };
        }

        Ok(Value::Dict(dict))
    }
}

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap};
    use crate::decoder::{Decoder, Value, Bytes};
    use crate::decode;
    use crate::error::{Error};

    /// Tests the reading, advancing & peeking of data.
    #[test]
    fn read_and_peek() {
        let data = "i3784e";
        let mut decoder = Decoder::new(data);
        assert_eq!(decoder.data.position(), 0);

        // Check, if reading of one byte advances the underlying cursor.
        let mut byte = decoder.read_byte().unwrap();
        assert_eq!(byte, b'i');
        assert_eq!(decoder.data.position(), 1);

        // Check, if peeking of one byte doesn't advance the underlying cursor.
        byte = decoder.peek_byte().unwrap();
        assert_eq!(byte, b'3');
        assert_eq!(decoder.data.position(), 1);
    
        // Read until the end & compare the expected with the position.
        let mut buf = [0u8; 5];
        let expected: &[u8] = "3784e".as_bytes();
        decoder.read(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(decoder.data.position() as usize, data.len());

        // Reading & peeking at the end should return an error.
        assert_eq!(decoder.read_byte().unwrap_err(), Error::EOF);
        assert_eq!(decoder.peek_byte().unwrap_err(), Error::EOF);
    }

    /*
        "Integers are represented by an 'i' followed by the number in base 10 followed by an 'e'. 
        For example i3e corresponds to 3 and i-3e corresponds to -3. 
        Integers have no size limitation. 
        i-0e is invalid. 
        All encodings with a leading zero, such as i03e, are invalid,
        other than i0e, which of course corresponds to 0."

        Source: http://www.bittorrent.org/beps/bep_0003.html
    */
    #[test]
    fn decode_int() {
        // Normal cases.
        assert_eq!(decode("i78e").unwrap(), Value::Int(78));
        assert_eq!(decode("i-360e").unwrap(), Value::Int(-360));
        assert_eq!(decode("i0e").unwrap(), Value::Int(0));
        assert_eq!(decode("i7580313e").unwrap(), Value::Int(7580313));

        // Edge cases.
        assert_eq!(decode("x1e").unwrap_err(), Error::NonExistingType);
        assert_eq!(decode("i321f").unwrap_err(), Error::ParseError);
        assert_eq!(decode("i-0e").unwrap_err(), Error::DataError);
        assert_eq!(decode("i8-3e").unwrap_err(), Error::DataError);
        assert_eq!(decode("i0321e").unwrap_err(), Error::DataError);
        assert_eq!(decode("i547").unwrap_err(), Error::EOF);
        assert_eq!(decode("isdfe").unwrap_err(), Error::ParseError);
    }

    /*
        "Strings are length-prefixed base ten followed by a colon and the string. 
        For example 4:spam corresponds to 'spam'."

        Source: http://www.bittorrent.org/beps/bep_0003.html
    */
    #[test]
    fn decode_str() {
        // Normal cases.
        assert_eq!(decode("4:asdf").unwrap(), Value::Str(Bytes::from("asdf")));
        assert_eq!(decode("7:bencode").unwrap(), Value::Str(Bytes::from("bencode")));
        assert_eq!(decode("10:m4k3s5en5e").unwrap(), Value::Str(Bytes::from("m4k3s5en5e")));
        assert_eq!(decode("0:").unwrap(), Value::Str(Bytes(vec![])));

        // Edge cases.
        assert_eq!(decode("4asdf").unwrap_err(), Error::ParseError);
        assert_eq!(decode("10:aa").unwrap_err(), Error::EOF);
        assert_eq!(decode("asdf").unwrap_err(), Error::NonExistingType);
    }

    /*
        "Lists are encoded as an 'l' followed by their elements (also bencoded) followed by an 'e'. 
        For example l4:spam4:eggse corresponds to ['spam', 'eggs']."

        Source: http://www.bittorrent.org/beps/bep_0003.html
    */
    #[test]
    fn decode_list() {
        let mut data: Vec<Value>;

        // Normal cases.
        // General case of strings.
        data = vec![
            Value::Str(Bytes::from("spam")), 
            Value::Str(Bytes::from("eggs"))
        ];
        assert_eq!(decode("l4:spam4:eggse").unwrap(), Value::List(data));

        // Strings with integers in them.
        data = vec![
            Value::Str(Bytes::from("m4k3s5en5e")), 
            Value::Str(Bytes::from("bencode"))
        ];
        assert_eq!(decode("l10:m4k3s5en5e7:bencodee").unwrap(), Value::List(data));

        // Mixed content of string and integers.
        data = vec![
            Value::Str(Bytes::from("mixed")), 
            Value::Int(-40), 
            Value::Str(Bytes::from("content"))
        ];
        assert_eq!(decode("l5:mixedi-40e7:contente").unwrap(), Value::List(data));

        // More complex mixing of inner lists.
        data = vec![
            Value::Str(Bytes::from("more")), 
            Value::List(vec![
                Value::Str(Bytes::from("mixed")), 
                Value::Int(1337)
            ]), 
            Value::Str(Bytes::from("content"))
        ];
        assert_eq!(decode("l4:morel5:mixedi1337ee7:contente").unwrap(), Value::List(data));

        // Empty list should return an empty Vec aswell.
        assert_eq!(decode("le").unwrap(), Value::List(vec![]));

        // Edge cases.
        // The errors of other values inside lists happen.
        assert_eq!(decode("li-0ee").unwrap_err(), Error::DataError);
        assert_eq!(decode("ei783ee").unwrap_err(), Error::NonExistingType);
        assert_eq!(decode("li-0e").unwrap_err(), Error::DataError);
    }

    /*
        "Dictionaries are encoded as a 'd' followed by a list of alternating keys 
        and their corresponding values followed by an 'e'. 
        For example, d3:cow3:moo4:spam4:eggse corresponds to {'cow': 'moo', 'spam': 'eggs'} 
        and d4:spaml1:a1:bee corresponds to {'spam': ['a', 'b']}. 
        Keys must be strings and appear in sorted order (sorted as raw strings, not alphanumerics)."

        Source: http://www.bittorrent.org/beps/bep_0003.html
    */
    #[test]
    fn decode_dict() {
        let mut data: BTreeMap<Bytes, Value> = BTreeMap::new();

        // Normal cases.
        // General case of strings.
        data.insert(
            Bytes::from("key"), 
            Value::Str(Bytes::from("value"))
        );
        assert_eq!(
            decode("d3:key5:valuee").unwrap(), 
            Value::Dict(data)
        );

        // Mixed content, dictionary inside a dictionary.
        data = BTreeMap::new();
        let mut data_mixed: BTreeMap<Bytes, Value> = BTreeMap::new();
        data_mixed.insert(
            Bytes::from("insidemeto"), 
            Value::Int(43)
        );
        data.insert(
            Bytes::from("list"), 
            Value::List(
                vec![Value::Int(3), Value::Int(-83)]
            )
        );
        data.insert(
            Bytes::from("content"),
            Value::Dict(data_mixed)
        );
        assert_eq!(
            decode("d4:listli3ei-83ee7:contentd10:insidemetoi43eee").unwrap(), 
            Value::Dict(data)
        );

        // Empty dictionary should return an empty BTreeMap aswell.
        assert_eq!(decode("de").unwrap(), Value::Dict(BTreeMap::new()));
    
        // Edge cases.
        // A non-string key should return a parse error.
        assert_eq!(decode("di35ee").unwrap_err(), Error::NonStringKey);
        // An empty key in a dictionary should return a parse error.
        assert_eq!(decode("d0:17:iwillnevergetheree").unwrap_err(), Error::NonStringKey);
        // An unfinished dictionary should return an EOF error.
        assert_eq!(decode("d3:hey99:unfinished").unwrap_err(), Error::EOF);
    }

    #[test]
    fn decode_first_type_infers() {
        // Only the first type can be inferred from the string, that contains more than one type,
        // that are not interoperrable.
        assert_eq!(decode("i32eli0ee").unwrap(), Value::Int(32));
    }
}

#[cfg(test)]
mod bench {

}
