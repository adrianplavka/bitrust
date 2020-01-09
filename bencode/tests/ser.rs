#[cfg(test)]
mod ser_tests {
    extern crate bitrust_bencode;
    use bitrust_bencode::to_string;
    use serde::Serialize;

    #[test]
    fn ser_integers() {
        assert_eq!(r#"i0e"#, to_string(&0usize).unwrap());
        assert_eq!(r#"i0e"#, to_string(&0isize).unwrap());
        assert_eq!(r#"i1e"#, to_string(&1usize).unwrap());
        assert_eq!(r#"i1e"#, to_string(&1isize).unwrap());
        assert_eq!(r#"i123e"#, to_string(&123usize).unwrap());
        assert_eq!(r#"i123e"#, to_string(&123isize).unwrap());
        assert_eq!(r#"i0e"#, to_string(&-0).unwrap());
        assert_eq!(r#"i-1e"#, to_string(&-1).unwrap());
        assert_eq!(r#"i-123e"#, to_string(&-123).unwrap());
    }

    #[test]
    fn ser_integers_bounds() {
        assert_eq!(
            format!("i{}e", std::u8::MAX),
            to_string(&std::u8::MAX).unwrap()
        );
        assert_eq!(
            format!("i{}e", std::u16::MAX),
            to_string(&std::u16::MAX).unwrap()
        );
        assert_eq!(
            format!("i{}e", std::u32::MAX),
            to_string(&std::u32::MAX).unwrap()
        );
        assert_eq!(
            format!("i{}e", std::u64::MAX),
            to_string(&std::u64::MAX).unwrap()
        );
        assert_eq!(
            format!("i{}e", std::i8::MAX),
            to_string(&std::i8::MAX).unwrap()
        );
        assert_eq!(
            format!("i{}e", std::i16::MAX),
            to_string(&std::i16::MAX).unwrap()
        );
        assert_eq!(
            format!("i{}e", std::i32::MAX),
            to_string(&std::i32::MAX).unwrap()
        );
        assert_eq!(
            format!("i{}e", std::i64::MAX),
            to_string(&std::i64::MAX).unwrap()
        );
    }

    #[test]
    fn ser_strings() {
        assert_eq!(r#"3:key"#, to_string(&"key").unwrap());
        assert_eq!(r#"5:asdfg"#, to_string(&"asdfg").unwrap());
        assert_eq!(r#"4:0087"#, to_string(&"0087").unwrap());
        assert_eq!(r#"0:"#, to_string(&"").unwrap());
        assert_eq!(r#"2:  "#, to_string(&"  ").unwrap());
        assert_eq!(r#"6:❤️"#, to_string(&"❤️").unwrap());
        assert_eq!(
            r#"21:!@#$%^&*()_+{}|:<>?"/"#,
            to_string(&"!@#$%^&*()_+{}|:<>?\"/").unwrap()
        );
        assert_eq!(
            r#"28:KR�/[W+x/^nAkW��;T0"#,
            to_string(&r#"KR�/[W+x/^nAkW��;T0"#).unwrap()
        );
    }

    #[test]
    fn ser_structs() {
        #[derive(Serialize, PartialEq, Debug)]
        struct IntegerTest {
            integer: i32,
            integers: Vec<i32>,
        }

        assert_eq!(
            r#"d7:integeri1995e8:integersli1ei2ei3eee"#,
            to_string(&IntegerTest {
                integer: 1995,
                integers: vec!(1, 2, 3)
            })
            .unwrap()
        );

        #[derive(Serialize, PartialEq, Debug)]
        struct StringTest<'a> {
            string: String,
            strings: Vec<String>,
            string_slice: &'a str,
            string_slices: Vec<&'a str>,
        }

        assert_eq!(
            r#"d6:string10:somestring7:stringsl1:a1:b1:ce12:string_slice100:longstringlongstringlongstringlongstringlongstringlongstringlongstringlongstringlongstringlongstring13:string_slicesl1:d1:e1:f1:gee"#,
            to_string(&StringTest {
                string: String::from("somestring"),
                strings: vec!(String::from("a"), String::from("b"), String::from("c")),
                string_slice: "longstring".repeat(10).as_str(),
                string_slices: vec!("d", "e", "f", "g")
            })
            .unwrap()
        );

        #[derive(Serialize, PartialEq, Debug)]
        struct InnerMixedStructTest<'a> {
            string: &'a str,
        }

        #[derive(Serialize, PartialEq, Debug)]
        struct MixedStructTest<'a> {
            integer: usize,
            negative_integer: i32,

            #[serde(borrow)]
            inner_struct: InnerMixedStructTest<'a>,
        }

        assert_eq!(
            r#"d7:integeri3000e16:negative_integeri-89343451e12:inner_structd6:string4:asdfee"#,
            to_string(&MixedStructTest {
                integer: 3000,
                negative_integer: -89343451,
                inner_struct: InnerMixedStructTest { string: "asdf" }
            })
            .unwrap()
        );
    }
}
