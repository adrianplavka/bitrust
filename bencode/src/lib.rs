
#![feature(test)]

// TODO: Remove these after the successful implementation.
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod bencode {
    use std;
    use std::result::{Result};
    use std::io::{Read};

    pub fn decode(data: &str) -> Result<Value, DecodeError> {
        // TODO: Make use of decode functions.
        unimplemented!()
    }

    pub fn decode_from<R: std::io::Read + std::io::Seek>(reader: R) -> Result<Value, DecodeError> {
        unimplemented!()
    }

    #[derive(Debug, PartialEq)]
    pub enum DecodeError {
        Invalid,
        UnexpectedSymbol,
        UnexpectedEnd,
        UnsupportedType,
        EOF,
        ParseError,
        Unknown
    }

    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct Byte(u8);
    
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct Bytes(Vec<Byte>);

    #[derive(Debug, PartialEq)]
    pub enum Value {
        Int(i64),
        Str(Bytes),
        List(Vec<Value>),
        Dict(std::collections::BTreeMap<Bytes, Value>),
        None
    }

    #[derive(Debug)]
    struct Decoder<'a> {
        data: std::io::Cursor<&'a [u8]>
    }

    impl<'a> Decoder<'a> {
        // Constructs a new decoder.
        // Accepts data as a string slice,
        // which then converts it to bytes to the underlying cursor.
        pub fn new(data: &str) -> Decoder {
            Decoder{ data: std::io::Cursor::new(data.as_bytes()) }
        }

        // Read and advance from the cursor to the length of a passed buffer
        // & save the data to it.
        fn read(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
            match self.data.read(buf) {
                Ok(n) if n == buf.len() => Ok(()),
                _ => Err(DecodeError::EOF)
            }
        }

        // Read & advance one byte from the cursor.
        fn read_byte(&mut self) -> Result<u8, DecodeError> {
            let mut buf = [0u8];
            try!(self.read(&mut buf));
            Ok(buf[0])
        }

        // Peeks, without advancing, one byte from the cursor.
        fn peek_byte(&mut self) -> Result<u8, DecodeError> {
            let data = self.read_byte()?;
            let pos = self.data.position();
            self.data.set_position(pos - 1);
            Ok(data)
        }

        // Reads & advance one byte, with expectation.
        fn expect_byte(&mut self, expect: u8) -> Result<(), DecodeError> {
            let byte = self.read_byte()?;

            if byte == expect {
                Ok(())
            } else {
                Err(DecodeError::UnexpectedSymbol)
            }
        }

        // Decodes an integer of the cursor's current position.
        // The position points to the 'i' character.
        fn decode_int(&mut self) -> Result<Value, DecodeError> {
            // Expect the first byte to represent an 'i' character.
            self.expect_byte('i' as u8)?;

            // Construct a buffer, from which we will parse the bytes into a number.
            // Error represents something, in which the parsing failed somehow.
            // Keep an index of the current loop iteration for logic regarding the '0' or '-' character.
            let mut buffer = String::new();
            let mut error: Option<DecodeError> = None;
            let mut i = 0;
            loop {
                let byte = self.read_byte()?;

                match byte {
                    // Numbers, besides '0', get pushed to the buffer.
                    b'1'...b'9' => buffer.push(byte as char),
                    // Character '0' will yield an error, if it happens to be on the beginning,
                    // while there are still some numbers left.
                    b'0' => {
                        let next = self.peek_byte()?;
                        if next != b'e' && i == 0 {
                            error = Some(DecodeError::ParseError);
                            break;
                        } else {
                            buffer.push(byte as char);
                        }
                    },
                    // Character '-' will yield an error, if it doesn't appear only at the beginning,
                    // or if the next character will be character '0'.
                    b'-' => {
                        let next = self.peek_byte()?;
                        if i != 0 || next == b'0' {
                            error = Some(DecodeError::ParseError);
                            break;
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
                        error = Some(DecodeError::ParseError);
                        break;
                    }
                }

                i += 1;
            }

            // Parse the buffer into an integer & return it, only if no error occured.
            match (buffer.parse(), error) {
                (Ok(v), None) => Ok(Value::Int(v)),
                _ => Err(DecodeError::ParseError)
            }
        }
    }


    #[cfg(test)]
    mod test {
        extern crate test;

        use std;
        use bencode::{Decoder, DecodeError, Value};

        /*
            Tests the reading, advancing & peeking of data.
        */
        #[test]
        fn read_and_peek() {
            let data = "i3784e";
            let mut decoder = Decoder::new(data);
            assert_eq!(decoder.data.position(), 0);

            // Check, if reading of one byte advances the underlying cursor.
            let mut byte = decoder.read_byte().unwrap();
            assert_eq!(byte, 'i' as u8);
            assert_eq!(decoder.data.position(), 1);

            // Check, if peeking of one byte doesn't advance the underlying cursor.
            byte = decoder.peek_byte().unwrap();
            assert_eq!(byte, '3' as u8);
            assert_eq!(decoder.data.position(), 1);
        
            // Read until the end & compare the expected with the position.
            let mut buf = [0u8; 5];
            let expected: &[u8] = "3784e".as_bytes();
            decoder.read(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(decoder.data.position(), data.len() as u64);

            // Reading & peeking at the end should return an error.
            assert_eq!(decoder.read_byte().unwrap_err(), DecodeError::EOF);
            assert_eq!(decoder.peek_byte().unwrap_err(), DecodeError::EOF);
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
        fn decode_num() {
            // Normal cases.
            assert_eq!(Decoder::new("i78e").decode_int().unwrap(), Value::Int(78));
            assert_eq!(Decoder::new("i-360e").decode_int().unwrap(), Value::Int(-360));
            assert_eq!(Decoder::new("i0e").decode_int().unwrap(), Value::Int(0));
            assert_eq!(Decoder::new("i7580313e").decode_int().unwrap(), Value::Int(7580313));

            // Edge cases.
            assert_eq!(Decoder::new("x1e").decode_int().unwrap_err(), DecodeError::UnexpectedSymbol);
            assert_eq!(Decoder::new("i321f").decode_int().unwrap_err(), DecodeError::ParseError);
            assert_eq!(Decoder::new("i-0e").decode_int().unwrap_err(), DecodeError::ParseError);
            assert_eq!(Decoder::new("i8-3e").decode_int().unwrap_err(), DecodeError::ParseError);
            assert_eq!(Decoder::new("i0321e").decode_int().unwrap_err(), DecodeError::ParseError);
            assert_eq!(Decoder::new("i547").decode_int().unwrap_err(), DecodeError::EOF);
            assert_eq!(Decoder::new("isdfe").decode_int().unwrap_err(), DecodeError::ParseError);
        }

        /*
            "Strings are length-prefixed base ten followed by a colon and the string. 
            For example 4:spam corresponds to 'spam'."

            Source: http://www.bittorrent.org/beps/bep_0003.html
        */
        #[test]
        #[ignore]
        fn decode_str() {
            unimplemented!();
        }

        /*
            "Lists are encoded as an 'l' followed by their elements (also bencoded) followed by an 'e'. 
            For example l4:spam4:eggse corresponds to ['spam', 'eggs']."

            Source: http://www.bittorrent.org/beps/bep_0003.html
        */
        #[test]
        #[ignore]
        fn decode_list() {
            unimplemented!();
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
        #[ignore]
        fn decode_dict() {
            unimplemented!();
        }
    }

    #[cfg(test)]
    mod bench {

    }
}
