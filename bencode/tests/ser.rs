#[cfg(test)]
mod tests {
    use nom::AsBytes;
    use quickcheck_macros::quickcheck;
    use serde_derive::Serialize;

    use bitrust_bencode::{to_string, to_vec};

    macro_rules! integer_test {
        ($method: ident, $type:ty) => {
            #[quickcheck]
            fn $method(value: $type) {
                assert_eq!(format!("i{}e", value), to_string(&value).unwrap())
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
    fn integers_near_bounds() {
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
    fn bools() {
        assert_eq!("4:true", to_string(&true).unwrap());
        assert_eq!("5:false", to_string(&false).unwrap());
    }

    #[quickcheck]
    fn strings(value: String) {
        assert_eq!(
            format!("{}:{}", value.len(), value),
            to_string(&value).unwrap()
        );
    }

    macro_rules! float_test {
        ($method: ident, $type:ty) => {
            #[quickcheck]
            fn $method(value: $type) {
                assert_eq!(
                    format!("{}:{}", value.to_string().len(), value),
                    to_string(&value).unwrap()
                )
            }
        };
    }

    float_test!(f32_float, f32);
    float_test!(f64_float, f64);

    #[quickcheck]
    fn bytes(value: String) {
        assert_eq!(
            format!("{}:{}", value.len(), value).as_bytes(),
            to_vec(&serde_bytes::Bytes::new(&value.as_bytes()))
                .unwrap()
                .as_bytes()
        )
    }

    #[test]
    fn structs() {
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
