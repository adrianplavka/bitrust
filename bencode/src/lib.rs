
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
        #[ignore]
        fn parse_num() {
            unimplemented!();
        }

        /*
            "Strings are length-prefixed base ten followed by a colon and the string. 
            For example 4:spam corresponds to 'spam'."

            Source: http://www.bittorrent.org/beps/bep_0003.html
        */
        #[test]
        #[ignore]
        fn parse_str() {
            unimplemented!();
        }

        /*
            "Lists are encoded as an 'l' followed by their elements (also bencoded) followed by an 'e'. 
            For example l4:spam4:eggse corresponds to ['spam', 'eggs']."

            Source: http://www.bittorrent.org/beps/bep_0003.html
        */
        #[test]
        #[ignore]
        fn parse_list() {
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
        fn parse_dict() {
            unimplemented!();
        }
    }

    #[cfg(test)]
    mod bench {

    }
}
