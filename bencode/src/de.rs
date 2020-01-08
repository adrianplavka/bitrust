//! Bencode deserializer using serde library.

use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;
use std::ops::{AddAssign, MulAssign, Neg};

use super::error::{Error, Result};

pub struct Deserializer<'de> {
    // This string starts with the input data
    // and characters are truncated off the beginning, as data is parsed.
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;

    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    // Look at the first character in the input without consuming it.
    fn peek_char(&mut self) -> Result<char> {
        self.input.chars().next().ok_or(Error::EOF)
    }

    // Look at the n-th character in the input without consuming it.
    fn peek_char_nth(&mut self, n: usize) -> Result<char> {
        self.input.chars().nth(n).ok_or(Error::EOF)
    }

    // Consume the first character in the input.
    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    // Parse the Bencode integer value.
    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        let mut integer = T::from(0);
        let mut is_first_loop = true;

        loop {
            match self.input.chars().next() {
                // Numbers, besides '0', get added to the final integer.
                Some(ch @ '1'..='9') => {
                    self.input = &self.input[1..];

                    integer *= T::from(10);
                    integer += T::from(ch as u8 - b'0');
                }
                // Character '0' will yield an error, if it happens to be on the beginning,
                // while there are still some numbers left.
                Some(ch @ '0') => {
                    self.input = &self.input[1..];

                    let next = self.peek_char()?;
                    if next != 'e' && is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }

                    integer *= T::from(10);
                }
                // Break the loop, if it's the end of integer.
                Some('e') => {
                    self.input = &self.input[1..];

                    // If an end has been occured while the integer is empty, yield an error (it's not a number).
                    if is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }
                    break;
                }
                // Default case, when something hadn't been covered.
                _ => {
                    return Err(Error::ExpectedInteger);
                }
            }

            is_first_loop = false;
        }

        Ok(integer)
    }

    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i8>,
    {
        let mut integer = T::from(0);
        let mut is_first_loop = true;
        let mut is_negative = false;

        loop {
            match self.input.chars().next() {
                // Numbers, besides '0', get added to the final integer.
                Some(ch @ '1'..='9') => {
                    self.input = &self.input[1..];

                    integer *= T::from(10);
                    integer += T::from((ch as u8 - b'0') as i8);
                }
                // Character '0' will yield an error, if it happens to be on the beginning,
                // while there are still some numbers left.
                Some(ch @ '0') => {
                    self.input = &self.input[1..];

                    let next = self.peek_char()?;
                    if next != 'e' && is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }

                    integer *= T::from(10);
                }
                Some('-') => {
                    self.input = &self.input[1..];

                    let next = self.peek_char()?;
                    if next == '0' || next == 'e' || !is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }

                    is_negative = true;
                }
                // Break the loop, if it's the end of integer.
                Some('e') => {
                    self.input = &self.input[1..];

                    // If an end has been occured while the integer is empty, yield an error (it's not a number).
                    if is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }
                    break;
                }
                // Default case, when something hadn't been covered.
                _ => {
                    return Err(Error::ExpectedInteger);
                }
            }

            is_first_loop = false;
        }

        Ok(if is_negative { -integer } else { integer })
    }

    // Parse the Bencode string value.
    fn parse_string(&mut self) -> Result<&'de str> {
        match self.input.find(':') {
            Some(idx) => {
                let length = match self.input[..idx].parse::<usize>() {
                    Ok(l) => l,
                    _ => return Err(Error::ParseStringIntegerLengthError),
                };

                if length > self.input[idx + 1..].len() {
                    return Err(Error::EOF);
                }

                let string = &self.input[idx + 1..=length + 1 + (self.input[..idx].len() - 1)];
                self.input = &self.input[idx + 1 + length..];
                Ok(string)
            }
            None => Err(Error::ExpectedStringIntegerLength),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_char()? {
            '0'..='9' => self.deserialize_str(visitor),
            'i' => {
                if self.peek_char_nth(1)? != '-' {
                    self.deserialize_u64(visitor)
                } else {
                    self.deserialize_i64(visitor)
                }
            }
            'l' => self.deserialize_seq(visitor),
            'd' => self.deserialize_map(visitor),
            _ => Err(Error::NonExistingType),
        }
    }
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_char()?;
        visitor.visit_u8(self.parse_unsigned::<u8>()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_char()?;
        visitor.visit_u16(self.parse_unsigned::<u16>()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_char()?;
        visitor.visit_u32(self.parse_unsigned::<u32>()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_char()?;
        visitor.visit_u64(self.parse_unsigned::<u64>()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_char()?;
        visitor.visit_i8(self.parse_signed::<i8>()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_char()?;
        visitor.visit_i16(self.parse_signed::<i16>()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_char()?;
        visitor.visit_i32(self.parse_signed::<i32>()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_char()?;
        visitor.visit_i64(self.parse_signed::<i64>()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_char()?;
        let value = visitor.visit_seq(ListDeserializer::new(&mut self))?;

        if self.next_char()? != 'e' {
            return Err(Error::ExpectedListEnd);
        }

        Ok(value)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_char()?;
        let value = visitor.visit_map(MapDeserializer::new(&mut self))?;

        if self.next_char()? != 'e' {
            return Err(Error::ExpectedMapEnd);
        }

        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool f32 f64 char
        bytes byte_buf unit unit_struct option
        enum newtype_struct
    }
}

struct ListDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> ListDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        ListDeserializer { de }
    }
}

impl<'de, 'a> SeqAccess<'de> for ListDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.peek_char()? == 'e' {
            return Ok(None);
        }

        // Deserialize an array element.
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct MapDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> MapDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        MapDeserializer { de }
    }
}

impl<'de, 'a> MapAccess<'de> for MapDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.de.peek_char()? {
            'e' => return Ok(None),
            '0'..='9' => {}
            _ => return Err(Error::ExpectedMapKeyString),
        }

        // Deserialize a map key.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }
}

#[cfg(test)]
mod test {
    use crate::de::from_str;
    use crate::error::Error;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_basic_de_integers() {
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

        // Unhappy paths.
        assert_eq!(Err(Error::ExpectedInteger), from_str::<usize>(r#"ie"#));
        assert_eq!(Err(Error::ExpectedInteger), from_str::<usize>(r#"i-0e"#));
        assert_eq!(Err(Error::ExpectedInteger), from_str::<usize>(r#"i1-23e"#));
        assert_eq!(Err(Error::ExpectedInteger), from_str::<usize>(r#"iasdfe"#));
        assert_eq!(Err(Error::ExpectedInteger), from_str::<usize>(r#"i e"#));
        assert_eq!(
            Err(Error::ExpectedInteger),
            from_str::<usize>(r#"i123.456e"#)
        );
        assert_eq!(
            Err(Error::ExpectedInteger),
            from_str::<usize>(r#"i-1.034e"#)
        );
        assert_eq!(
            Err(Error::TrailingCharacters),
            from_str::<usize>(r#"i123etrailing"#)
        );
    }

    #[test]
    fn test_basic_de_strings() {
        // Happy paths.
        assert_eq!("key", from_str::<&str>(r#"3:key"#).unwrap());
        assert_eq!("asdfg", from_str::<&str>(r#"5:asdfg"#).unwrap());
        assert_eq!("0087", from_str::<&str>(r#"4:0087"#).unwrap());
        assert_eq!(
            "!@#$%^&*()_+{}|:<>?\"/",
            from_str::<&str>(r#"21:!@#$%^&*()_+{}|:<>?"/"#).unwrap()
        );
        assert_eq!("", from_str::<&str>(r#"0:"#).unwrap());
        assert_eq!("  ", from_str::<&str>(r#"2:  "#).unwrap());

        // Unhappy paths.
        assert_eq!(Err(Error::EOF), from_str::<&str>(r#"4:EOF"#));
        assert_eq!(
            Err(Error::ExpectedStringIntegerLength),
            from_str::<&str>(r#"string"#)
        );
        assert_eq!(
            Err(Error::ParseStringIntegerLengthError),
            from_str::<&str>(r#"nointeger:value"#)
        );
        assert_eq!(
            Err(Error::TrailingCharacters),
            from_str::<&str>(r#"3:keytrailing"#)
        );
    }

    #[test]
    fn test_basic_de_structs() {
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
}
