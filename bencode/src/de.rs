//! Bencode deserialization using serde library.

use num_traits::{CheckedAdd, CheckedMul};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use std::ops::Neg;

use super::error::{Error, Result};

pub struct Deserializer<'de> {
    /// This string starts with the input data and characters are
    /// truncated off the beginning, as data is being parsed.
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

    if !deserializer.input.is_empty() {
        Err(Error::TrailingCharacters)
    } else {
        Ok(t)
    }
}

impl<'de> Deserializer<'de> {
    /// Peek at the first character in the input without consuming it.
    fn peek_char(&mut self) -> Result<char> {
        self.input.chars().next().ok_or(Error::EOF)
    }

    /// Peek at the n-th character in the input without consuming it.
    fn peek_char_nth(&mut self, n: usize) -> Result<char> {
        self.input.chars().nth(n).ok_or(Error::EOF)
    }

    /// Consume the first character in the input.
    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[1..];
        Ok(ch)
    }

    /// Parse the Bencode unsigned integer value.
    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: CheckedAdd + CheckedMul + From<u8>,
    {
        let mut integer = T::from(0);
        let mut is_first_loop = true;

        loop {
            match self.next_char()? {
                // Numbers (besides '0'), get added to the final integer.
                ch @ '1'..='9' => {
                    integer = match integer.checked_mul(&T::from(10)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };

                    integer = match integer.checked_add(&T::from(ch as u8 - b'0')) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };
                }
                // Number '0' is treated differently, as it cannot occur multiple
                // times at the beginning.
                // It will yield an error, if it happens to be on the beginning,
                // while there are still some numbers left.
                '0' => {
                    let next = self.peek_char()?;
                    if next != 'e' && is_first_loop {
                        return Err(Error::ExpectedUnsignedInteger);
                    }

                    integer = match integer.checked_mul(&T::from(10)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };
                }
                // Break the loop, if it's the end of integer.
                'e' => {
                    // If an end has been occured while the integer is empty,
                    // yield an error (it's not a number).
                    if is_first_loop {
                        return Err(Error::ExpectedUnsignedInteger);
                    }
                    break;
                }
                // If an non-expecting character has been found.
                _ => {
                    return Err(Error::ExpectedUnsignedInteger);
                }
            }

            is_first_loop = false;
        }

        Ok(integer)
    }

    /// Parse the Bencode signed integer value.
    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + CheckedAdd + CheckedMul + From<i8>,
    {
        let mut integer = T::from(0);
        let mut is_first_loop = true;
        let mut is_negative = false;

        loop {
            match self.next_char()? {
                // Numbers (besides '0') get added to the final integer.
                ch @ '1'..='9' => {
                    integer = match integer.checked_mul(&T::from(10)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };

                    integer = match integer.checked_add(&T::from((ch as u8 - b'0') as i8)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };
                }
                // Number '0' is treated differently, as it cannot occur multiple
                // times at the beginning.
                // It will yield an error, if it happens to be on the beginning,
                // while there are still some numbers left.
                '0' => {
                    let next = self.peek_char()?;
                    if next != 'e' && is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }

                    integer = match integer.checked_mul(&T::from(10)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };
                }
                '-' => {
                    // Special case to check, if a negative symbol happens to be in
                    // front of characters '0' or 'e', which are not numbers.
                    // Also, if the symbol happens to appear anywhere except at the
                    // beginning, it will yield an error.
                    let next = self.peek_char()?;
                    if next == '0' || next == 'e' || !is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }

                    is_negative = true;
                }
                // Break the loop, if it's the end of integer.
                'e' => {
                    // If an end has been occured while the integer is empty,
                    // yield an error (it's not a number).
                    if is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }
                    break;
                }
                // If an non-expecting character has been found.
                _ => {
                    return Err(Error::ExpectedInteger);
                }
            }

            is_first_loop = false;
        }

        Ok(if is_negative { -integer } else { integer })
    }

    /// Parse the Bencode string value.
    fn parse_string(&mut self) -> Result<&'de str> {
        match self.input.find(':') {
            Some(idx) => {
                // Retrieve the length of the string as number from the
                // beginning of a string.
                let length = match self.input[..idx].parse::<usize>() {
                    Ok(l) => l,
                    _ => return Err(Error::ParseStringIntegerLengthError),
                };

                // If length of the string is bigger than the input itself,
                // it could result into an array out of bounds error.
                // Yields an unexpected end error.
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

macro_rules! fn_deserialize_unsigned {
    ($method:ident, $visit:ident, $type:ty) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>
        {
            match self.next_char()? {
                'i' => {
                    if self.peek_char()? == '-' {
                        return Err(Error::ExpectedUnsignedInteger);
                    }
                    visitor.$visit(self.parse_unsigned::<$type>()?)
                }
                _ => Err(Error::ExpectedUnsignedInteger),
            }
        }
    };
}

macro_rules! fn_deserialize_signed {
    ($method:ident, $visit:ident, $type:ty) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.next_char()? {
                'i' => visitor.$visit(self.parse_signed::<$type>()?),
                _ => Err(Error::ExpectedInteger),
            }
        }
    };
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    /// Look at the input data to decide, what Serde data model type to deserialize as.
    /// It will infer a Bencode type based on starting characters, useful when no
    /// type was provided to "from_*" deserialization functions.
    ///
    /// Integers will be always deserialized to unsigned or signed type, depending on
    /// a knowledge, if unparsed integer starts with a '-':
    ///     - If it doesn't, the type of integer will be `u64`.
    ///     - If it does, the type of integer will be `i64`.
    ///
    /// Not all data formats are able to support this operation & will result in an
    /// UnknownType error.
    ///
    /// Can return errors when deserializing types.
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
            _ => Err(Error::UnknownType),
        }
    }

    /// See `Deserializer::deserialize_any` method.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    /// Method definitions for various unsigned deserializations.
    ///
    /// Every definition guarantees, that it will use the appropriate type when
    /// deserializing, saving it from using bigger, unnecessary unsigned types.
    ///
    /// This can overflow, if an input has a bigger size than expected type.
    ///
    /// Can return errors when deserializing unsigned types.
    fn_deserialize_unsigned!(deserialize_u8, visit_u8, u8);
    fn_deserialize_unsigned!(deserialize_u16, visit_u16, u16);
    fn_deserialize_unsigned!(deserialize_u32, visit_u32, u32);
    fn_deserialize_unsigned!(deserialize_u64, visit_u64, u64);

    /// Method definitions for various signed deserializations.
    ///
    /// Every definition guarantees, that it will use the appropriate type when
    /// deserializing, saving it from using bigger, unnecessary signed types.
    ///
    /// This can overflow, if an input has a bigger size than expected type.
    ///
    /// Can return errors when deserializing signed types.
    fn_deserialize_signed!(deserialize_i8, visit_i8, i8);
    fn_deserialize_signed!(deserialize_i16, visit_i16, i16);
    fn_deserialize_signed!(deserialize_i32, visit_i32, i32);
    fn_deserialize_signed!(deserialize_i64, visit_i64, i64);

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_char()? {
            '0'..='9' => visitor.visit_borrowed_str(self.parse_string()?),
            _ => Err(Error::ExpectedStringIntegerLength),
        }
    }

    /// See `Deserializer::deserialize_str` method.
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    /// See `Deserializer::deserialize_str` method.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.next_char()? {
            'l' => {
                let value = visitor.visit_seq(ListDeserializer::new(&mut self))?;
                if self.next_char()? != 'e' {
                    return Err(Error::ExpectedListEnd);
                }

                Ok(value)
            }
            _ => Err(Error::ExpectedList),
        }
    }

    /// See `Deserializer::deserialize_seq` method.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    /// See `Deserializer::deserialize_seq` method.
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
        match self.next_char()? {
            'd' => {
                let value = visitor.visit_map(DictionaryDeserializer::new(&mut self))?;
                if self.next_char()? != 'e' {
                    return Err(Error::ExpectedDictionaryEnd);
                }

                Ok(value)
            }
            _ => Err(Error::ExpectedDictionary),
        }
    }

    /// See `Deserializer::deserialize_map` method.
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

struct DictionaryDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> DictionaryDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        DictionaryDeserializer { de }
    }
}

impl<'de, 'a> MapAccess<'de> for DictionaryDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.de.peek_char()? {
            'e' => return Ok(None),
            '0'..='9' => {}
            _ => return Err(Error::ExpectedDictionaryKeyString),
        };

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
