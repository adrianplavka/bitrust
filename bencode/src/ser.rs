//! Bencode serialization.

use std::{io::Write, str, string::ToString};

use crate::{
    error::{Error, Result},
    token,
};

use serde::{ser, Serialize};

/// A structure that serializes Rust values into Bencode.
pub struct Serializer {
    data: Vec<u8>,
}

impl Serializer {
    pub fn new() -> Self {
        Serializer { data: Vec::new() }
    }
}

/// Serializes a value into a `Vec` of bytes containing Bencode value.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut ser = Serializer::new();

    value.serialize(&mut ser)?;

    Ok(ser.data)
}

/// Serializes a value into a `String` containing Bencode value.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ser::Serialize,
{
    let mut ser = Serializer::new();

    value.serialize(&mut ser)?;

    let string = String::from_utf8(ser.data).map_err(|_| Error::InvalidUTF8)?;
    Ok(string)
}

impl Serializer {
    fn serialize_integer<T>(&mut self, value: T) -> Result<()>
    where
        T: ToString,
    {
        self.data.write(&[token::INTEGER_START])?;
        self.data.write(value.to_string().as_bytes())?;
        self.data.write(&[token::END])?;

        Ok(())
    }
}

macro_rules! fn_serialize_integer {
    ($method:ident, $type:ty) => {
        fn $method(self, value: $type) -> Result<()> {
            self.serialize_integer(value)
        }
    };
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn_serialize_integer!(serialize_u8, u8);
    fn_serialize_integer!(serialize_u16, u16);
    fn_serialize_integer!(serialize_u32, u32);
    fn_serialize_integer!(serialize_u64, u64);
    serde::serde_if_integer128! {
        fn_serialize_integer!(serialize_u128, u128);
    }

    fn_serialize_integer!(serialize_i8, i8);
    fn_serialize_integer!(serialize_i16, i16);
    fn_serialize_integer!(serialize_i32, i32);
    fn_serialize_integer!(serialize_i64, i64);
    serde::serde_if_integer128! {
        fn_serialize_integer!(serialize_i128, i128);
    }

    fn serialize_str(self, value: &str) -> Result<()> {
        self.data.write(value.len().to_string().as_bytes())?;
        self.data.write(&[token::BYTES_DELIMITER])?;
        self.data.write(value.as_bytes())?;

        Ok(())
    }

    fn serialize_bool(self, value: bool) -> Result<()> {
        self.serialize_str(if value { "true" } else { "false" })
    }

    fn serialize_char(self, value: char) -> Result<()> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_f32(self, value: f32) -> Result<()> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_f64(self, value: f64) -> Result<()> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        self.data.write(value)?;

        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        Ok(())
    }

    /// A present optional is represented as just the contained value. Note that
    /// this is a lossy representation. For example the values `Some(())` and
    /// `None` both serialize as just `null`. Unfortunately this is typically
    /// what people expect when working with JSON. Other formats are encouraged
    /// to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    /// Unit struct means a named value containing no data. Again, since there is
    /// no data. There is no need to serialize the name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    /// When serializing a unit variant (or any other kind of variant), formats
    /// can choose whether to keep track of it by index or by name. Binary
    /// formats typically use the index of the variant and human-readable formats
    /// typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    /// As is done here, serializers are encouraged to treat newtype structs as
    /// insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    /// Note that newtype variant (and all of the other variant serialization
    /// methods) refer exclusively to the "externally tagged" enum
    /// representation.
    ///
    /// Serialize this to Bencode in externally tagged form as
    /// `d<length>:<key><length>:<value>e`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.data.write(&[token::MAP_START])?;

        variant.serialize(&mut *self)?;
        value.serialize(&mut *self)?;

        self.data.write(&[token::END])?;

        Ok(())
    }

    /// The start of the sequence, each value, and the end are three separate
    /// method calls. This one is responsible only for serializing the start,
    /// which in Bencode is 'l'.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.data.write(&[token::LIST_START])?;

        Ok(self)
    }

    /// Tuples look just like sequences in Bencode. Some formats may be able to
    /// represent tuples more efficiently by omitting the length, since tuple
    /// means that the corresponding `Deserialize` implementation will know the
    /// length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    /// Tuple variants are represented in Bencode as `d<length>:<key>l<data>ee`.
    /// This method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.data.write(&[token::MAP_START])?;

        variant.serialize(&mut *self)?;

        self.data.write(&[token::LIST_START])?;

        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.data.write(&[token::MAP_START])?;

        Ok(self)
    }

    /// Structs look just like maps in Bencode. In particular, Bencode requires that we
    /// serialize the field names of the struct. Other formats may be able to
    /// omit the field names when serializing structs because the corresponding
    /// Deserialize implementation is required to know what the keys are without
    /// looking at the serialized data.
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    /// Struct variants are represented in Bencode as `d<length>:<key>d<key>:<value>...ee`.
    /// This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.data.write(&[token::MAP_START])?;

        variant.serialize(&mut *self)?;

        self.data.write(&[token::MAP_START])?;

        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.data.write(&[token::END])?;

        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.data.write(&[token::END])?;

        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.data.write(&[token::END])?;

        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        // Responsible for closing both the dictionary & list.
        self.data.write(&[token::END])?;
        self.data.write(&[token::END])?;

        Ok(())
    }
}

/// Some `Serialize` types are not able to hold a key and value in memory at the
/// same time so `SerializeMap` implementations are required to support
/// `serialize_key` and `serialize_value` individually.
impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    /// The Serde data model allows map keys to be any serializable type. Bencode
    /// only allows string keys so the implementation below will produce invalid
    /// Bencode if the key serializes as something other than a string.
    ///
    /// A real Bencode serializer would need to validate that map keys are strings.
    /// This can be done by using a different Serializer to serialize the key
    /// (instead of `&mut **self`) and having that other serializer only
    /// implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        // TODO: Make sure that keys are strings.
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.data.write(&[token::END])?;

        Ok(())
    }
}

/// Structs are like maps in which the keys are constrained to be compile-time
/// constant strings.
impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.data.write(&[token::END])?;

        Ok(())
    }
}

/// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
/// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.data.write(&[token::END])?;
        self.data.write(&[token::END])?;

        Ok(())
    }
}
