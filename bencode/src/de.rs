//! Bencode deserializer using serde library.

use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;

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

    // Consume the first character in the input.
    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    // Parse the Bencode integer value.
    fn parse_integer(&mut self) -> Result<i64> {
        let mut integer = 0i64;
        let mut is_first_loop = true;

        loop {
            match self.input.chars().next() {
                // Numbers, besides '0', get added to the final integer.
                Some(ch @ '1'..='9') => {
                    integer *= 10;
                    integer += (ch as u8 - b'0') as i64;
                    self.input = &self.input[1..];
                }
                // Character '0' will yield an error, if it happens to be on the beginning,
                // while there are still some numbers left.
                Some(ch @ '0') => {
                    let next = self.peek_char()?;
                    if next != 'e' && is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }

                    integer *= 10;
                    self.input = &self.input[1..];
                }
                // Break the loop, if it's the end of integer.
                Some('e') => {
                    // If an end has been occured while the integer is empty, yield an error (it's not a number).
                    if is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }
                    self.input = &self.input[1..];
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

                let string = &self.input[idx + 1..=length + 1];
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
            'i' => self.deserialize_i64(visitor),
            'l' => self.deserialize_seq(visitor),
            'd' => self.deserialize_map(visitor),
            _ => Err(Error::NonExistingType),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_char()?;
        visitor.visit_i64(self.parse_integer()?)
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
        bool u8 u16 u32 u64 i8 i16 i32 f32 f64 char
        bytes byte_buf unit unit_struct option enum newtype_struct
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
    fn test_integer() {
        assert_eq!(1, from_str(r#"i1e"#).unwrap());

        assert_eq!(
            std::i64::MAX,
            from_str(format!("i{}e", std::i64::MAX).as_str()).unwrap()
        );
        // TODO: This panics (poor implementation of integer parsing)
        /*
        assert_eq!(
            std::u64::MAX,
            from_str(format!("i{}e", std::u64::MAX).as_str()).unwrap()
        );
        */
    }

    #[test]
    fn test_string() {
        // Happy paths.
        assert_eq!("key", from_str::<&str>(r#"3:key"#).unwrap());
        assert_eq!("asdfg", from_str::<&str>(r#"5:asdfg"#).unwrap());
        assert_eq!("0087", from_str::<&str>(r#"4:0087"#).unwrap());
        assert_eq!(
            "!@#$%^&*()_+{}|:<>?\"/",
            from_str::<&str>(r#"22:!@#$%^&*()_+{}|:<>?"//"#).unwrap()
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
            from_str::<&str>(r#"3:key4:asdf"#)
        );
    }
}
