
#![feature(test)]

pub mod bencode {
    use std;
    use std::collections::HashMap;
    use std::io::Read;

    const NUM_DELIMITER: char   = 'i';
    const DICT_DELIMITER: char  = 'd';
    const LIST_DELIMITER: char  = 'l';
    const END_DELIMITER: char   = 'e';
    const COL_DELIMITER: char   = ':';

    #[derive(Debug, PartialEq)]
    pub enum Error {
        Invalid,
        UnexpectedSymbol,
        UnexpectedEnd,
        Unknown
    }

    pub type Result<T> = std::result::Result<T, Error>;

    pub type Bytes = Vec<u8>;

    #[derive(Debug, PartialEq)]
    pub enum Type {
        Num(i64),
        Str(Bytes),
        List(Vec<Type>),
        Dict(HashMap<Bytes, Type>),
        None
    }

    #[derive(Debug, PartialEq)]
    enum ParseState {
        AtNum,
        AtStr,
        AtList,
        AtDict,
        None
    }

    #[derive(Debug)]
    pub struct Parser {
        pos: u64,
        state: ParseState
    }

    impl Parser {
        pub fn decode_str(data: &str) -> Result<Type> {
            unimplemented!()
        }

        pub fn decode_from<R>(reader: R) -> () where R: Read {
            unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use std;
    use std::fs::File;
    use ::bencode::{Parser, Type, Error};

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
    fn parse_num() {
        assert_eq!(Parser::decode_str("i0e").unwrap(), Type::Num(0));
        assert_eq!(Parser::decode_str("i-1e").unwrap(), Type::Num(-1));
        assert_eq!(Parser::decode_str("i28e").unwrap(), Type::Num(28));
        assert_eq!(Parser::decode_str("i489e").unwrap(), Type::Num(489));
        assert_eq!(Parser::decode_str("i981795470e").unwrap(), Type::Num(981795470));

        assert_eq!(Parser::decode_str("i-0e").unwrap_err(), Error::Unknown);
        assert_eq!(Parser::decode_str("i098e").unwrap_err(), Error::Unknown);
    }

    /*
        "Strings are length-prefixed base ten followed by a colon and the string. 
        For example 4:spam corresponds to 'spam'."

        Source: http://www.bittorrent.org/beps/bep_0003.html
    */
    #[test]
    fn parse_str() {
        unimplemented!();
    }

    /*
        "Lists are encoded as an 'l' followed by their elements (also bencoded) followed by an 'e'. 
        For example l4:spam4:eggse corresponds to ['spam', 'eggs']."

        Source: http://www.bittorrent.org/beps/bep_0003.html
    */
    #[test]
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
    fn parse_dict() {
        unimplemented!();
    }

    #[bench]
    fn bench_parse_num(b: &mut test::Bencher) {
        b.iter(|| Parser::decode_str("i175476243e").unwrap());
    }
}
