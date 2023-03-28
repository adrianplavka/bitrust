#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;
    use serde_derive::Deserialize;

    use bitrust_bencode::{from_slice, from_str, Error};

    macro_rules! integer_test {
        ($method: ident, $type:ty) => {
            #[quickcheck]
            fn $method(value: $type) {
                assert_eq!(value, from_str(&format!("i{}e", value)).unwrap())
            }
        };
    }

    integer_test!(u8_integers, u8);
    integer_test!(u16_integers, u16);
    integer_test!(u32_integers, u32);
    integer_test!(u64_integers, u64);
    integer_test!(usize_integers, usize);

    integer_test!(i8_integers, i8);
    integer_test!(i16_integers, i16);
    integer_test!(i32_integers, i32);
    integer_test!(i64_integers, i64);
    integer_test!(isize_integers, isize);

    #[test]
    fn integers_edge_cases() {
        assert!(matches!(
            from_str::<usize>(r#"ie"#),
            Err(Error::ExpectedUnsignedNumber)
        ));

        assert!(matches!(
            from_str::<usize>(r#"i1-23e"#),
            Err(Error::ExpectedIntegerEnd)
        ));

        assert!(matches!(
            from_str::<usize>(r#"iasdfe"#),
            Err(Error::ExpectedUnsignedNumber)
        ));

        assert!(matches!(
            from_str::<usize>(r#"i e"#),
            Err(Error::ExpectedUnsignedNumber)
        ));

        assert!(matches!(
            from_str::<u8>(r#"i-100e"#),
            Err(Error::ExpectedUnsignedNumber)
        ));

        assert!(matches!(
            from_str::<usize>(r#"i123"#),
            Err(Error::ExpectedIntegerEnd)
        ));

        assert!(matches!(
            from_str::<usize>(r#"i123.456e"#),
            Err(Error::ExpectedIntegerEnd)
        ));

        assert!(matches!(
            from_str::<isize>(r#"i-1.034e"#),
            Err(Error::ExpectedIntegerEnd)
        ));

        assert!(matches!(
            from_str::<usize>(r#"4:asdf"#),
            Err(Error::ExpectedInteger)
        ));

        assert!(matches!(
            from_str::<usize>(r#"li123ee"#),
            Err(Error::ExpectedInteger)
        ));

        assert!(matches!(
            from_str::<usize>(r#"d1:ai323ee"#),
            Err(Error::ExpectedInteger)
        ));

        assert!(matches!(
            from_str::<usize>(r#"i123etrailing"#),
            Err(Error::TrailingCharacters)
        ));
    }

    #[test]
    fn integers_near_bounds() {
        // Happy paths.
        assert_eq!(
            std::u8::MAX,
            from_str(format!("i{}e", std::u8::MAX).as_str()).unwrap()
        );
        assert_eq!(
            std::u16::MAX,
            from_str(format!("i{}e", std::u16::MAX).as_str()).unwrap()
        );
        assert_eq!(
            std::u32::MAX,
            from_str(format!("i{}e", std::u32::MAX).as_str()).unwrap()
        );
        assert_eq!(
            std::u64::MAX,
            from_str(format!("i{}e", std::u64::MAX).as_str()).unwrap()
        );
        assert_eq!(
            std::i8::MAX,
            from_str(format!("i{}e", std::i8::MAX).as_str()).unwrap()
        );
        assert_eq!(
            std::i16::MAX,
            from_str(format!("i{}e", std::i16::MAX).as_str()).unwrap()
        );
        assert_eq!(
            std::i32::MAX,
            from_str(format!("i{}e", std::i32::MAX).as_str()).unwrap()
        );
        assert_eq!(
            std::i64::MAX,
            from_str(format!("i{}e", std::i64::MAX).as_str()).unwrap()
        );

        // Unhappy paths.
        assert!(matches!(
            from_str::<u8>(format!("i{}0e", std::u8::MAX).as_str()),
            Err(Error::IntegerOverflow),
        ));

        assert!(matches!(
            from_str::<u16>(format!("i{}0e", std::u16::MAX).as_str()),
            Err(Error::IntegerOverflow),
        ));

        assert!(matches!(
            from_str::<u32>(format!("i{}0e", std::u32::MAX).as_str()),
            Err(Error::IntegerOverflow),
        ));

        assert!(matches!(
            from_str::<u64>(format!("i{}0e", std::u64::MAX).as_str()),
            Err(Error::IntegerOverflow),
        ));

        assert!(matches!(
            from_str::<i8>(format!("i{}0e", std::i8::MAX).as_str()),
            Err(Error::IntegerOverflow),
        ));

        assert!(matches!(
            from_str::<i16>(format!("i{}0e", std::i16::MAX).as_str()),
            Err(Error::IntegerOverflow),
        ));

        assert!(matches!(
            from_str::<i32>(format!("i{}0e", std::i32::MAX).as_str()),
            Err(Error::IntegerOverflow),
        ));

        assert!(matches!(
            from_str::<i64>(format!("i{}0e", std::i64::MAX).as_str()),
            Err(Error::IntegerOverflow),
        ));
    }

    #[quickcheck]
    fn strings(value: String) {
        assert_eq!(
            value,
            from_str::<&str>(&format!("{}:{}", value.len(), value)).unwrap()
        );
    }

    #[test]
    fn bools() {
        assert_eq!(true, from_str::<bool>("4:true").unwrap());
        assert_eq!(false, from_str::<bool>("5:false").unwrap());
    }

    #[test]
    fn strings_edge_cases() {
        assert!(matches!(from_str::<&str>(r#"4:EOF"#), Err(Error::EOF)));

        assert!(matches!(
            from_str::<&str>(r#"string"#),
            Err(Error::ExpectedUnsignedNumber)
        ));

        assert!(matches!(
            from_str::<&str>(r#"nointeger:value"#),
            Err(Error::ExpectedUnsignedNumber)
        ));

        assert!(matches!(
            from_str::<&str>(r#"i123e"#),
            Err(Error::ExpectedUnsignedNumber),
        ));

        assert!(matches!(
            from_str::<&str>(r#"l2:abe"#),
            Err(Error::ExpectedUnsignedNumber),
        ));

        assert!(matches!(
            from_str::<&str>(r#"d1:ae"#),
            Err(Error::ExpectedUnsignedNumber),
        ));

        assert!(matches!(
            from_str::<&str>(r#"3:keytrailing"#),
            Err(Error::TrailingCharacters)
        ));
    }

    macro_rules! float_test {
        ($method: ident, $type:ty) => {
            #[quickcheck]
            fn $method(value: $type) {
                assert_eq!(
                    value,
                    from_str(&format!("{}:{}", value.to_string().len(), value)).unwrap()
                )
            }
        };
    }

    float_test!(f32_floats, f32);
    float_test!(f64_floats, f64);

    #[test]
    fn floats_edge_cases() {
        assert!(matches!(
            from_str::<f64>(r#"7:invalid"#),
            Err(Error::ExpectedFloat)
        ));

        assert!(matches!(
            from_str::<f64>(r#"3:-0a"#),
            Err(Error::ExpectedFloat)
        ));

        assert!(matches!(
            from_str::<f64>(r#"0:"#),
            Err(Error::ExpectedFloat)
        ));
    }

    #[quickcheck]
    fn bytes(value: String) {
        assert_eq!(
            value.as_bytes(),
            from_slice::<&[u8]>(format!("{}:{}", value.len(), value).as_bytes()).unwrap()
        );
    }

    #[test]
    fn bytes_edge_cases() {
        // Check for a valid conversion from byte slice.
        // This sequence would translate to: `6:He?llo`.
        //
        // Since this conversion is raw & doesn't translate to UTF-8, it should
        // unwrap without an error (even though there is an invalid code point).
        assert_eq!(
            &[0x48, 0x65, 0xf0, 0x6c, 0x6c, 0x6f],
            from_slice::<&[u8]>(&[0x36, 0x3a, 0x48, 0x65, 0xf0, 0x6c, 0x6c, 0x6f]).unwrap()
        );

        // Check for an invalid conversion from byte slice to an UTF-8 `&str`.
        // This sequence would translate to: `6:He?llo`.
        //
        // This sequence has an invalid code point 0xf0, therefore it should fail.
        assert!(matches!(
            from_slice::<&str>(&[0x36, 0x3a, 0x48, 0x65, 0xf0, 0x6c, 0x6c, 0x6f]),
            Err(Error::InvalidUTF8)
        ));
    }

    #[test]
    fn structs() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct IntegerTest {
            integer: i32,
            integers: Vec<i32>,
        }

        assert_eq!(
            IntegerTest {
                integer: 1995,
                integers: vec!(1, 2, 3)
            },
            from_str::<IntegerTest>(r#"d7:integeri1995e8:integersli1ei2ei3eee"#).unwrap()
        );

        #[derive(Deserialize, PartialEq, Debug)]
        struct StringTest<'a> {
            string: String,
            strings: Vec<String>,
            string_slice: &'a str,
            string_slices: Vec<&'a str>,
        }

        assert_eq!(
            StringTest {
                string: String::from("somestring"),
                strings: vec!(String::from("a"), String::from("b"), String::from("c")),
                string_slice: "longstring".repeat(10).as_str(),
                string_slices: vec!("d", "e", "f", "g")
            },
            from_str::<StringTest>(
                r#"d6:string10:somestring7:stringsl1:a1:b1:ce12:string_slice100:longstringlongstringlongstringlongstringlongstringlongstringlongstringlongstringlongstringlongstring13:string_slicesl1:d1:e1:f1:gee"#
            )
            .unwrap()
        );

        #[derive(Deserialize, PartialEq, Debug)]
        struct InnerMixedStructTest<'a> {
            string: &'a str,
        }

        #[derive(Deserialize, PartialEq, Debug)]
        struct MixedStructTest<'a> {
            integer: usize,
            negative_integer: i32,

            #[serde(borrow)]
            inner_struct: InnerMixedStructTest<'a>,
        }

        assert_eq!(
            MixedStructTest {
                integer: 3000,
                negative_integer: -89343451,
                inner_struct: InnerMixedStructTest { string: "asdf" }
            },
            from_str::<MixedStructTest>(
                r#"d7:integeri3000e16:negative_integeri-89343451e12:inner_structd6:string4:asdfee"#
            )
            .unwrap()
        );
    }

    #[test]
    fn struct_from_file() {
        use std::env;
        use std::fs;
        use std::path::Path;

        #[derive(Deserialize, PartialEq, Debug)]
        struct TorrentInfo<'a> {
            length: usize,

            name: &'a str,

            #[serde(rename(deserialize = "piece length"))]
            piece_length: usize,

            pieces: &'a [u8],
        }

        #[derive(Deserialize, PartialEq, Debug)]
        struct TorrentMetainfo<'a> {
            #[serde(borrow)]
            announce: &'a str,

            info: TorrentInfo<'a>,
        }

        let mut dir = env::current_dir().unwrap();
        dir.push(Path::new(
            "tests/data/ubuntu-19.10-desktop-amd64.iso.torrent",
        ));
        let f = &fs::read(dir).unwrap();

        // Expecting a valid deserialization, therefore shouldn't throw any errors.
        from_slice::<TorrentMetainfo>(f).unwrap();
    }
}
