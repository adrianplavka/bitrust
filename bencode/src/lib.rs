
#![feature(test)]

pub mod bencode {
    use std::collections::HashMap;

    const NUM_DELIMITER: char = 'i';
    const DICT_DELIMITER: char = 'd';
    const LIST_DELIMITER: char = 'l';
    const DELIMITER: char = 'e';

    #[derive(Debug, PartialEq)]
    pub enum Type {
        None,
        Num(f64),
        Str(Vec<u8>),
        List(Vec<Type>),
        Dict(HashMap<Vec<u8>, Type>)
    }

    #[derive(Debug, PartialEq)]
    enum ParseState {
        None,
        AtNum,
        AtStr,
        AtList,
        AtDict
    }

    pub struct Parser {
        pos: u64,
        state: ParseState,
        data: P
    }

    impl Parser {
        pub fn decode(data: &str) {

        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::bencode::Parser;
    use super::bencode::Type;

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
