//! Bencode deserialization.

use std::str::{self, FromStr};

use crate::{
    error::{Error, Result},
    token,
};

use lexical::FromLexical;
use nom::bytes::complete::{is_a, tag, take};
use num_traits::{Float, Signed, Unsigned};
use serde::de;

/// A structure that deserializes Bencode into Rust values.
pub struct Deserializer<'a> {
    data: &'a [u8],
}

impl<'a> Deserializer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

/// Deserializes a byte slice containing Bencode format.
///
/// The type of the data to be deserialized into is specified using
/// a generic type `T`.
///
/// This function will also check, if any trailing characters are
/// present at the end of the deserialization, triggering an error.
pub fn from_slice<'a, T>(data: &'a [u8]) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    let mut de = Deserializer::new(data);
    let value = de::Deserialize::deserialize(&mut de)?;

    if de.data.len() == 0 {
        Ok(value)
    } else {
        Err(Error::TrailingCharacters)
    }
}

/// Deserializes a string slice containing Bencode format.
///
/// The type of the data to be deserialized into is specified using
/// a generic type `T`.
///
/// This function will also check, if any trailing characters are
/// present at the end of the deserialization, triggering an error.
pub fn from_str<'a, T>(data: &'a str) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    let mut de = Deserializer::new(data.as_bytes());
    let value = de::Deserialize::deserialize(&mut de)?;

    if de.data.len() == 0 {
        Ok(value)
    } else {
        Err(Error::TrailingCharacters)
    }
}

//////////////////////////////////////////////////////

#[inline]
fn consume_integer_start(x: &[u8]) -> Result<&[u8]> {
    tag::<&[u8], &[u8], ()>(&[token::INTEGER_START])(x)
        .map(|(rest, _)| rest)
        .map_err(|_| Error::ExpectedInteger)
}

#[inline]
fn consume_signed_number<T>(x: &[u8]) -> Result<(&[u8], T)>
where
    T: Signed + FromLexical,
{
    let (rest, value) = is_a::<&[u8], &[u8], ()>(token::SIGNED_NUMBER_CHARSET)(x)
        .map_err(|_| Error::ExpectedSignedNumber)?;

    let integer = lexical::parse::<T, _>(value).map_err(|e| {
        if e.is_overflow() {
            Error::IntegerOverflow
        } else {
            Error::ExpectedSignedNumber
        }
    })?;

    Ok((rest, integer))
}

#[inline]
fn consume_unsigned_number<T>(x: &[u8]) -> Result<(&[u8], T)>
where
    T: Unsigned + FromLexical,
{
    let (rest, value) = is_a::<&[u8], &[u8], ()>(token::UNSIGNED_NUMBER_CHARSET)(x)
        .map_err(|_| Error::ExpectedUnsignedNumber)?;

    let integer = lexical::parse::<T, _>(value).map_err(|e| {
        if e.is_overflow() {
            Error::IntegerOverflow
        } else {
            Error::ExpectedUnsignedNumber
        }
    })?;

    Ok((rest, integer))
}

#[inline]
fn consume_bytes_delimiter(x: &[u8]) -> Result<&[u8]> {
    tag::<&[u8], &[u8], ()>(&[token::BYTES_DELIMITER])(x)
        .map(|(rest, _)| rest)
        .map_err(|_| Error::ExpectedStringIntegerLength)
}

#[inline]
fn consume_bytes(x: &[u8], count: usize) -> Result<(&[u8], &[u8])> {
    take::<usize, &[u8], ()>(count)(x).map_err(|_| Error::EOF)
}

#[inline]
fn consume_end(x: &[u8], e: Error) -> Result<&[u8]> {
    tag::<&[u8], &[u8], ()>(&[token::END])(x)
        .map(|(rest, _)| rest)
        .map_err(|_| e)
}

//////////////////////////////////////////////////////

impl<'a> Deserializer<'a> {
    fn peek_byte(&mut self, index: usize) -> Result<u8> {
        self.data.get(index).ok_or(Error::EOF).map(|v| v.to_owned())
    }

    fn next_byte(&mut self) -> Result<u8> {
        let byte = self.data.get(0).ok_or(Error::EOF).map(|b| b.to_owned())?;
        self.data = &self.data[1..];

        Ok(byte)
    }

    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Signed + FromLexical,
    {
        let data = consume_integer_start(self.data)?;
        let (data, number) = consume_signed_number::<T>(data)?;
        self.data = consume_end(data, Error::ExpectedIntegerEnd)?;

        Ok(number)
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: Unsigned + FromLexical,
    {
        let data = consume_integer_start(self.data)?;
        let (data, number) = consume_unsigned_number::<T>(data)?;
        self.data = consume_end(data, Error::ExpectedIntegerEnd)?;

        Ok(number)
    }

    fn parse_bytes(&mut self) -> Result<&'a [u8]> {
        let (data, count) = consume_unsigned_number::<usize>(self.data)?;
        let data = consume_bytes_delimiter(data)?;
        let (data, bytes) = consume_bytes(data, count)?;
        self.data = data;

        Ok(bytes)
    }

    fn parse_string(&mut self) -> Result<&'a str> {
        let bytes = self.parse_bytes()?;
        let string = str::from_utf8(&bytes).map_err(|_| Error::InvalidUTF8)?;

        Ok(string)
    }

    fn parse_float<T>(&mut self) -> Result<T>
    where
        T: Float + FromStr,
    {
        let string = self.parse_string()?;
        let float = string.parse::<T>().map_err(|_| Error::ExpectedFloat)?;

        Ok(float)
    }
}

//////////////////////////////////////////////////////

macro_rules! fn_deserialize_unsigned {
    ($method:ident, $visit:ident, $type:ty) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            visitor.$visit(self.parse_unsigned::<$type>()?)
        }
    };
}

macro_rules! fn_deserialize_signed {
    ($method:ident, $visit:ident, $type:ty) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            visitor.$visit(self.parse_signed::<$type>()?)
        }
    };
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.peek_byte(0)? {
            b'0'..=b'9' => self.deserialize_str(visitor),
            token::INTEGER_START => {
                if let b'-' = self.peek_byte(1)? {
                    self.deserialize_i64(visitor)
                } else {
                    self.deserialize_u64(visitor)
                }
            }
            token::LIST_START => self.deserialize_seq(visitor),
            token::MAP_START => self.deserialize_map(visitor),
            _ => Err(Error::UnknownType),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn_deserialize_unsigned!(deserialize_u8, visit_u8, u8);
    fn_deserialize_unsigned!(deserialize_u16, visit_u16, u16);
    fn_deserialize_unsigned!(deserialize_u32, visit_u32, u32);
    fn_deserialize_unsigned!(deserialize_u64, visit_u64, u64);
    serde::serde_if_integer128! {
        fn_deserialize_unsigned!(deserialize_u128, visit_u128, u128);
    }

    fn_deserialize_signed!(deserialize_i8, visit_i8, i8);
    fn_deserialize_signed!(deserialize_i16, visit_i16, i16);
    fn_deserialize_signed!(deserialize_i32, visit_i32, i32);
    fn_deserialize_signed!(deserialize_i64, visit_i64, i64);
    serde::serde_if_integer128! {
        fn_deserialize_signed!(deserialize_i128, visit_i128, i128);
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float::<f32>()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float::<f64>()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.parse_bytes()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let token::LIST_START = self.next_byte()? {
            let value = visitor.visit_seq(ListDeserializer::new(&mut self))?;

            if let token::END = self.next_byte()? {
                Ok(value)
            } else {
                Err(Error::ExpectedListEnd)
            }
        } else {
            Err(Error::ExpectedList)
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
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
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let token::MAP_START = self.next_byte()? {
            let value = visitor.visit_map(MapDeserializer::new(&mut self))?;

            if let token::END = self.next_byte()? {
                Ok(value)
            } else {
                Err(Error::ExpectedDictionaryEnd)
            }
        } else {
            Err(Error::ExpectedDictionary)
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool char
        unit unit_struct option
        enum newtype_struct
    }
}

//////////////////////////////////////////////////////

struct ListDeserializer<'de, 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> ListDeserializer<'de, 'a> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        ListDeserializer { de }
    }
}

impl<'de, 'a> de::SeqAccess<'de> for ListDeserializer<'de, 'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let token::END = self.de.peek_byte(0)? {
            Ok(None)
        } else {
            seed.deserialize(&mut *self.de).map(Some)
        }
    }
}

//////////////////////////////////////////////////////

struct MapDeserializer<'de, 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> MapDeserializer<'de, 'a> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        MapDeserializer { de }
    }
}

impl<'de, 'a> de::MapAccess<'de> for MapDeserializer<'de, 'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.de.peek_byte(0)? {
            token::END => Ok(None),
            b'0'..=b'9' => seed.deserialize(&mut *self.de).map(Some),
            _ => Err(Error::ExpectedDictionaryKeyString),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}
