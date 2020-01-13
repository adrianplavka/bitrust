#[cfg(test)]
mod de_tests {
    extern crate bitrust_bencode;
    use bitrust_bencode::{from_slice, from_str, Error};
    use serde::Deserialize;

    #[test]
    fn de_integers() {
        // Happy paths.
        assert_eq!(0usize, from_str(r#"i0e"#).unwrap());
        assert_eq!(0isize, from_str(r#"i0e"#).unwrap());
        assert_eq!(1usize, from_str(r#"i1e"#).unwrap());
        assert_eq!(1isize, from_str(r#"i1e"#).unwrap());
        assert_eq!(123usize, from_str(r#"i123e"#).unwrap());
        assert_eq!(123isize, from_str(r#"i123e"#).unwrap());
        assert_eq!(-0, from_str(r#"i0e"#).unwrap());
        assert_eq!(-1, from_str(r#"i-1e"#).unwrap());
        assert_eq!(-123, from_str(r#"i-123e"#).unwrap());

        // Unhappy paths.
        assert_eq!(
            Err(Error::ExpectedUnsignedInteger),
            from_str::<usize>(r#"ie"#)
        );
        assert_eq!(Err(Error::ExpectedInteger), from_str::<isize>(r#"i-0e"#));
        assert_eq!(
            Err(Error::ExpectedUnsignedInteger),
            from_str::<usize>(r#"i1-23e"#)
        );
        assert_eq!(
            Err(Error::ExpectedUnsignedInteger),
            from_str::<usize>(r#"iasdfe"#)
        );
        assert_eq!(
            Err(Error::ExpectedUnsignedInteger),
            from_str::<usize>(r#"i e"#)
        );
        assert_eq!(
            Err(Error::ExpectedUnsignedInteger),
            from_str::<u8>(r#"i-100e"#)
        );
        assert_eq!(Err(Error::EOF), from_str::<usize>(r#"i123"#));
        assert_eq!(
            Err(Error::ExpectedUnsignedInteger),
            from_str::<usize>(r#"i123.456e"#)
        );
        assert_eq!(
            Err(Error::ExpectedUnsignedInteger),
            from_str::<usize>(r#"i007e"#)
        );
        assert_eq!(Err(Error::ExpectedInteger), from_str::<isize>(r#"i007e"#));
        assert_eq!(
            Err(Error::ExpectedInteger),
            from_str::<isize>(r#"i-1.034e"#)
        );
        assert_eq!(
            Err(Error::ExpectedUnsignedInteger),
            from_str::<usize>(r#"4:asdf"#)
        );
        assert_eq!(
            Err(Error::ExpectedUnsignedInteger),
            from_str::<usize>(r#"li123ee"#)
        );
        assert_eq!(
            Err(Error::ExpectedUnsignedInteger),
            from_str::<usize>(r#"d1:ai323ee"#)
        );
        assert_eq!(
            Err(Error::TrailingCharacters),
            from_str::<usize>(r#"i123etrailing"#)
        );
    }

    #[test]
    fn de_integers_bounds() {
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
        assert_eq!(
            Err(Error::IntegerOverflow),
            from_str::<u8>(format!("i{}0e", std::u8::MAX).as_str())
        );
        assert_eq!(
            Err(Error::IntegerOverflow),
            from_str::<u16>(format!("i{}0e", std::u16::MAX).as_str())
        );
        assert_eq!(
            Err(Error::IntegerOverflow),
            from_str::<u32>(format!("i{}0e", std::u32::MAX).as_str())
        );
        assert_eq!(
            Err(Error::IntegerOverflow),
            from_str::<u64>(format!("i{}0e", std::u64::MAX).as_str())
        );
        assert_eq!(
            Err(Error::IntegerOverflow),
            from_str::<i8>(format!("i{}0e", std::i8::MAX).as_str())
        );
        assert_eq!(
            Err(Error::IntegerOverflow),
            from_str::<i16>(format!("i{}0e", std::i16::MAX).as_str())
        );
        assert_eq!(
            Err(Error::IntegerOverflow),
            from_str::<i32>(format!("i{}0e", std::i32::MAX).as_str())
        );
        assert_eq!(
            Err(Error::IntegerOverflow),
            from_str::<i64>(format!("i{}0e", std::i64::MAX).as_str())
        );
    }

    #[test]
    fn de_strings() {
        // Happy paths.
        assert_eq!("key", from_str::<&str>(r#"3:key"#).unwrap());
        assert_eq!("asdfg", from_str::<&str>(r#"5:asdfg"#).unwrap());
        assert_eq!("0087", from_str::<&str>(r#"4:0087"#).unwrap());
        assert_eq!("", from_str::<&str>(r#"0:"#).unwrap());
        assert_eq!("  ", from_str::<&str>(r#"2:  "#).unwrap());
        assert_eq!("❤️", from_str::<&str>(r#"6:❤️"#).unwrap());
        assert_eq!(
            "!@#$%^&*()_+{}|:<>?\"/",
            from_str::<&str>(r#"21:!@#$%^&*()_+{}|:<>?"/"#).unwrap()
        );
        assert_eq!(
            r#"KR�/[W+x/^nAkW��;T0"#,
            from_str::<&str>(r#"28:KR�/[W+x/^nAkW��;T0"#).unwrap()
        );

        // Unhappy paths.
        assert_eq!(Err(Error::EOF), from_str::<&str>(r#"4:EOF"#));
        assert_eq!(
            Err(Error::ExpectedStringIntegerLength),
            from_str::<&str>(r#"string"#)
        );
        assert_eq!(
            Err(Error::ExpectedStringIntegerLength),
            from_str::<&str>(r#"nointeger:value"#)
        );
        assert_eq!(
            Err(Error::ExpectedStringIntegerLength),
            from_str::<&str>(r#"i123e"#)
        );
        assert_eq!(
            Err(Error::ExpectedStringIntegerLength),
            from_str::<&str>(r#"l2:abe"#)
        );
        assert_eq!(
            Err(Error::ExpectedStringIntegerLength),
            from_str::<&str>(r#"d1:ae"#)
        );
        assert_eq!(
            Err(Error::TrailingCharacters),
            from_str::<&str>(r#"3:keytrailing"#)
        );
    }

    #[test]
    fn de_floats() {
        // Happy paths.
        assert_eq!(4.32, from_str::<f32>(r#"4:4.32"#).unwrap());
        assert_eq!(134.64, from_str::<f64>(r#"6:134.64"#).unwrap());
        assert_eq!(-134.64, from_str::<f64>(r#"7:-134.64"#).unwrap());
        assert_eq!(-0.0, from_str::<f64>(r#"4:-0.0"#).unwrap());
        assert_eq!(-5032.0, from_str::<f64>(r#"5:-5032"#).unwrap());
        assert_eq!(0.0, from_str::<f64>(r#"0:"#).unwrap());

        // Unhappy paths.
        assert_eq!(Err(Error::ExpectedFloat), from_str::<f64>(r#"7:invalid"#));
        assert_eq!(Err(Error::ExpectedFloat), from_str::<f64>(r#"3:-0a"#));
    }

    #[test]
    fn de_bytes() {
        // Happy paths.

        // Check for a valid conversion from byte slice.
        // This sequence would translate to: `6:He?llo`.
        //
        // Since this conversion is raw & doesn't translate to UTF-8, it should
        // unwrap without an error (even though there is an invalid code point).
        assert_eq!(
            &[0x48, 0x65, 0xf0, 0x6c, 0x6c, 0x6f],
            from_slice::<&[u8]>(&[0x36, 0x3a, 0x48, 0x65, 0xf0, 0x6c, 0x6c, 0x6f]).unwrap()
        );

        // Unhappy paths.

        // Check for an invalid conversion from byte slice to an UTF-8 `&str`.
        // This sequence would translate to: `6:He?llo`.
        //
        // This sequence has an invalid code point 0xf0, therefore it should fail.
        assert_eq!(
            Err(Error::InvalidUnicodeCodePoint),
            from_slice::<&str>(&[0x36, 0x3a, 0x48, 0x65, 0xf0, 0x6c, 0x6c, 0x6f])
        );
    }

    #[test]
    fn de_structs() {
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
    fn de_structs_file() {
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
