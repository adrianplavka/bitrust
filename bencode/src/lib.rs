
#![feature(test)]

pub mod bencode {
    use std::collections::HashMap;

    const NUM_DELIMITER: char   = 'i';
    const DICT_DELIMITER: char  = 'd';
    const LIST_DELIMITER: char  = 'l';
    const END_DELIMITER: char   = 'e';

    pub type Bytes = Vec<u8>;

    #[derive(Debug, PartialEq)]
    pub enum Type {
        None,
        Num(f64),
        Str(Bytes),
        List(Vec<Type>),
        Dict(HashMap<Bytes, Type>)
    }

    #[derive(Debug, PartialEq)]
    enum ParseState {
        None,
        AtNum,
        AtStr,
        AtList,
        AtDict
    }

    #[derive(Debug)]
    pub struct Parser {
        pos: u64,
        state: ParseState
    }

    impl Parser {
        pub fn decode(data: &str) -> Type {
            Type::None
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::bencode::{Parser, Type};

    #[test]
    fn parse_numbers() {
        assert_eq!(Parser::decode("i0e"), Type::Num(0.0));
        assert_eq!(Parser::decode("i28e"), Type::Num(28.0));
        assert_eq!(Parser::decode("i489e"), Type::Num(489.0));
        assert_eq!(Parser::decode("i981795470e"), Type::Num(981795470.0));
        // Edge cases.
        assert_eq!(Parser::decode("i-1e"), Type::Num(-1.0));
        assert_eq!(Parser::decode("i-0e"), Type::Num(0.0));
        assert_eq!(Parser::decode("i098e"), Type::Num(98.0));
    }

    #[bench]
    fn bench_parse_numbers(b: &mut test::Bencher) {
        b.iter(|| Parser::decode("i975476243e"));
    }
}
