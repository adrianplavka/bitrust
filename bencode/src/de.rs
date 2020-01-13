//! Bencode deserialization using Serde library.

use std::ops::Neg;
use std::str::{self, FromStr};

use num_traits::{CheckedAdd, CheckedMul, Float};
use serde::de::{self, DeserializeSeed, Visitor};
use serde::Deserialize;

use crate::error::{Error, Result};
use crate::read::{self};
pub use crate::read::{Read, SliceRead, StrRead};

/// A structure that deserializes Bencode into Rust values.
pub struct Deserializer<R> {
    read: R,
}

impl<'de, R> Deserializer<R>
where
    R: read::Read<'de>,
{
    /// Create a Bencode deserializer from one of the possible bitrust_bencode
    /// input sources.
    ///
    /// Typically it is more convenient to use one of these methods instead:
    ///     - Deserializer::from_str
    ///     - Deserializer::from_slice
    ///
    /// Or using exported functions:
    ///     - bitrust_bencode::from_str
    ///     - bitrust_benocde::from_slice
    pub fn new(read: R) -> Self {
        Deserializer { read }
    }
}

impl<'de, 'a> Deserializer<read::SliceRead<'a>> {
    /// Creates a Bencode deserializer from a `&[u8]`.
    pub fn from_slice(bytes: &'a [u8]) -> Self {
        Deserializer::new(read::SliceRead::new(bytes))
    }
}

impl<'de, 'a> Deserializer<read::StrRead<'a>> {
    /// Creates a Bencode deserializer from a `&str`.
    pub fn from_str(s: &'a str) -> Self {
        Deserializer::new(read::StrRead::new(s))
    }
}

fn from_trait<'de, R, T>(read: R) -> Result<T>
where
    R: Read<'de>,
    T: de::Deserialize<'de>,
{
    let mut de = Deserializer::new(read);
    let value = de::Deserialize::deserialize(&mut de)?;

    if de.read.end() {
        Ok(value)
    } else {
        Err(Error::TrailingCharacters)
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    from_trait(read::StrRead::new(s))
}

pub fn from_slice<'a, T>(v: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    from_trait(read::SliceRead::new(v))
}

//////////////////////////////////////////////////////////////////////////////

impl<'de, R> Deserializer<R>
where
    R: read::Read<'de>,
{
    /// Parse a Bencode's signed integer value.
    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + CheckedAdd + CheckedMul + From<i8>,
    {
        let mut integer = T::from(0);
        let mut is_first_loop = true;
        let mut is_negative = false;

        loop {
            match self.read.next_byte()? {
                // Numbers (besides '0') get added to the final integer.
                ch @ b'1'..=b'9' => {
                    integer = match integer.checked_mul(&T::from(10)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };

                    integer = match integer.checked_add(&T::from((ch - b'0') as i8)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };
                }
                // Number '0' is treated differently, as it cannot occur multiple
                // times at the beginning.
                // It will yield an error, if it happens to be on the beginning,
                // while there are still some numbers left.
                b'0' => {
                    let next = self.read.peek_byte()?;
                    if next != b'e' && is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }

                    integer = match integer.checked_mul(&T::from(10)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };
                }
                b'-' => {
                    // Special case to check, if a negative symbol happens to be in
                    // front of characters '0' or 'e', which are not valid negative
                    // numbers.
                    // Also, if the symbol happens to appear anywhere except at the
                    // beginning, it will yield an error.
                    let next = self.read.peek_byte()?;
                    if next == b'0' || next == b'e' || !is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }

                    is_negative = true;
                }
                // Break the loop, if it's the end of integer.
                b'e' => {
                    // If an end has been occured, while at the beginning of an
                    // integer, yield an error (it's not a number).
                    if is_first_loop {
                        return Err(Error::ExpectedInteger);
                    }
                    break;
                }
                // If an unexpecting character has been found.
                _ => {
                    return Err(Error::ExpectedInteger);
                }
            }

            is_first_loop = false;
        }

        Ok(if is_negative { -integer } else { integer })
    }

    /// Parse a Bencode's unsigned integer value.
    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: CheckedAdd + CheckedMul + From<u8>,
    {
        let mut integer = T::from(0);
        let mut is_first_loop = true;

        loop {
            match self.read.next_byte()? {
                // Numbers (besides '0'), get added to the final integer.
                ch @ b'1'..=b'9' => {
                    integer = match integer.checked_mul(&T::from(10)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };

                    integer = match integer.checked_add(&T::from(ch - b'0')) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };
                }
                // Number '0' is treated differently, as it cannot occur multiple
                // times at the beginning.
                // It will yield an error, if it happens to be on the beginning,
                // while there are still some numbers left.
                b'0' => {
                    let next = self.read.peek_byte()?;
                    if next != b'e' && is_first_loop {
                        return Err(Error::ExpectedUnsignedInteger);
                    }

                    integer = match integer.checked_mul(&T::from(10)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };
                }
                // Break the loop, if it's the end of integer.
                b'e' => {
                    // If an end has been occured, while at the beginning of an
                    // integer, yield an error (it's not a number).
                    if is_first_loop {
                        return Err(Error::ExpectedUnsignedInteger);
                    }
                    break;
                }
                // If an unexpecting character has been found.
                _ => {
                    return Err(Error::ExpectedUnsignedInteger);
                }
            }

            is_first_loop = false;
        }

        Ok(integer)
    }

    /// Parse a Bencode's byte string length.
    ///
    /// The parsing is essentially the same as parsing an unsigned value
    /// (since the byte string's length cannot be negative),
    /// but with a different end delimiter.
    fn parse_string_length<T>(&mut self) -> Result<T>
    where
        T: CheckedAdd + CheckedMul + From<u8>,
    {
        let mut integer = T::from(0);
        let mut is_first_loop = true;

        loop {
            match self.read.next_byte()? {
                // Numbers (besides '0'), get added to the final integer.
                ch @ b'1'..=b'9' => {
                    integer = match integer.checked_mul(&T::from(10)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };

                    integer = match integer.checked_add(&T::from(ch - b'0')) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };
                }
                // Number '0' is treated differently, as it cannot occur multiple
                // times at the beginning.
                // It will yield an error, if it happens to be on the beginning,
                // while there are still some numbers left.
                b'0' => {
                    let next = self.read.peek_byte()?;
                    if next != b':' && is_first_loop {
                        return Err(Error::ExpectedStringIntegerLength);
                    }

                    integer = match integer.checked_mul(&T::from(10)) {
                        Some(i) => i,
                        _ => return Err(Error::IntegerOverflow),
                    };
                }
                // Break the loop, if it's the end of string length.
                b':' => {
                    // If an end has been occured, while at the beginning of an
                    // integer, yield an error (it's not a number).
                    if is_first_loop {
                        return Err(Error::ExpectedStringIntegerLength);
                    }
                    break;
                }
                // If an unexpecting byte has been found.
                _ => {
                    return Err(Error::ExpectedStringIntegerLength);
                }
            }

            is_first_loop = false;
        }

        Ok(integer)
    }

    /// Parse a Bencode's byte string value as UTF-8 encoded string.
    fn parse_string(&mut self) -> Result<&'de str> {
        let length = self.parse_string_length::<usize>()?;

        // This will always be a positive length, but check, if the length is greater
        // than zero.
        if length > 0 {
            // Assumption, that the deserialized value is a valid UTF-8 string.
            let string = match str::from_utf8(self.read.next_bytes(length - 1)?) {
                Ok(s) => s,
                _ => return Err(Error::InvalidUnicodeCodePoint),
            };
            Ok(string)
        } else {
            Ok("")
        }
    }

    /// Parse a Bencode's byte string value as bytes.
    fn parse_bytes(&mut self) -> Result<&'de [u8]> {
        let length = self.parse_string_length::<usize>()?;

        // This will always be a positive length, but check, if the length is greater
        // than zero.
        if length > 0 {
            let bytes = self.read.next_bytes(length - 1)?;
            Ok(bytes)
        } else {
            Ok(&[])
        }
    }

    /// Parse a Bencode's byte string value as a float.
    fn parse_float<T>(&mut self) -> Result<T>
    where
        T: Float + FromStr,
    {
        let length = self.parse_string_length::<usize>()?;

        // This will always be a positive length, but check, if the length is greater
        // than zero.
        if length != 0 {
            // Assumption, that the deserialized value is a valid UTF-8 string.
            let string_float = match str::from_utf8(self.read.next_bytes(length - 1)?) {
                Ok(s) => s,
                _ => return Err(Error::InvalidUnicodeCodePoint),
            };

            let float = match string_float.parse::<T>() {
                Ok(f) => f,
                _ => return Err(Error::ExpectedFloat),
            };

            Ok(float)
        } else {
            Ok(T::from(0.0).unwrap())
        }
    }
}

macro_rules! fn_deserialize_unsigned {
    ($method:ident, $visit:ident, $type:ty) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>
        {
            match self.read.next_byte()? {
                b'i' => {
                    if self.read.peek_byte()? == b'-' {
                        return Err(Error::ExpectedUnsignedInteger);
                    }
                    visitor.$visit(self.parse_unsigned::<$type>()?)
                }
                _ => {
                    Err(Error::ExpectedUnsignedInteger)
                },
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
            match self.read.next_byte()? {
                b'i' => visitor.$visit(self.parse_signed::<$type>()?),
                _ => Err(Error::ExpectedInteger),
            }
        }
    };
}

impl<'de, 'a, R: Read<'de>> de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    /// Look at the input data to decide, what Serde data model type to deserialize as.
    /// It will infer a Bencode type based on starting characters, useful when no
    /// type was provided to `from_*` deserialization functions.
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
        match self.read.peek_byte()? {
            b'0'..=b'9' => self.deserialize_str(visitor),
            b'i' => {
                if self.read.peek_byte_nth(1)? != b'-' {
                    self.deserialize_u64(visitor)
                } else {
                    self.deserialize_i64(visitor)
                }
            }
            b'l' => self.deserialize_seq(visitor),
            b'd' => self.deserialize_map(visitor),
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
    serde::serde_if_integer128! {
        fn_deserialize_unsigned!(deserialize_u128, visit_u128, u128);
    }

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
    serde::serde_if_integer128! {
        fn_deserialize_signed!(deserialize_i128, visit_i128, i128);
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.read.peek_byte()? {
            b'0'..=b'9' => visitor.visit_borrowed_str(self.parse_string()?),
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

    /// Deserializes string as `f32`, since Bencode's integer doesn't allow floats.
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.read.peek_byte()? {
            b'0'..=b'9' => visitor.visit_f32(self.parse_float::<f32>()?),
            _ => Err(Error::ExpectedStringIntegerLength),
        }
    }

    /// Deserializes string as `f64`, since Bencode's integer doesn't allow floats.
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.read.peek_byte()? {
            b'0'..=b'9' => visitor.visit_f64(self.parse_float::<f64>()?),
            _ => Err(Error::ExpectedStringIntegerLength),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.read.peek_byte()? {
            b'0'..=b'9' => visitor.visit_borrowed_bytes(self.parse_bytes()?),
            _ => Err(Error::ExpectedStringIntegerLength),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.read.peek_byte()? {
            b'0'..=b'9' => visitor.visit_borrowed_bytes(self.parse_bytes()?),
            _ => Err(Error::ExpectedStringIntegerLength),
        }
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.read.next_byte()? {
            b'l' => {
                let value = visitor.visit_seq(ListDeserializer::new(&mut self))?;
                if self.read.next_byte()? != b'e' {
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
        match self.read.next_byte()? {
            b'd' => {
                let value = visitor.visit_map(DictionaryDeserializer::new(&mut self))?;
                if self.read.next_byte()? != b'e' {
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
        bool char
        unit unit_struct option
        enum newtype_struct
    }
}

//////////////////////////////////////////////////////////////////////////////

struct ListDeserializer<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: 'a> ListDeserializer<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        ListDeserializer { de }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::SeqAccess<'de> for ListDeserializer<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.read.peek_byte()? == b'e' {
            return Ok(None);
        }

        // Deserialize an array element.
        seed.deserialize(&mut *self.de).map(Some)
    }
}

//////////////////////////////////////////////////////////////////////////////

struct DictionaryDeserializer<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: 'a> DictionaryDeserializer<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        DictionaryDeserializer { de }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::MapAccess<'de> for DictionaryDeserializer<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.de.read.peek_byte()? {
            b'e' => return Ok(None),
            b'0'..=b'9' => {}
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
