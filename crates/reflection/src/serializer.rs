use std::fmt::{Debug, Display};

use serde::Serialize;

use crate::data::Data;

/// Error type for reflection data serialization
#[derive(Debug, Clone)]
pub enum Error {
    InvalidPendingData,
    MissingPendingData,
    MissingPendingKey,
    Custom(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidPendingData => f.write_str("Invalid pending data"),
            Error::MissingPendingData => f.write_str("Missing pending data"),
            Error::MissingPendingKey => f.write_str("Missing pending key"),
            Error::Custom(msg) => f.write_str(&msg),
        }
    }
}

impl std::error::Error for Error {}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}

/// Reflection data serializer
#[derive(Debug, Default, Clone)]
pub struct Serializer {
    human_readable: bool,
    pending_data: Vec<Data>,
    pending_keys: Vec<Data>,
}

impl Serializer {
    pub fn new(human_readable: bool) -> Self {
        Serializer {
            human_readable,
            pending_data: Default::default(),
            pending_keys: Default::default(),
        }
    }
}

impl<'a> serde::Serializer for &'a mut Serializer {
    type Ok = Data;

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Data::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Data::I8(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Data::I16(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Data::I32(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Data::I64(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(Data::I128(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Data::U8(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Data::U16(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Data::U32(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Data::U64(v))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(Data::U128(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Data::F32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Data::F64(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Data::Char(v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Data::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Data::ByteArray(v.iter().copied().collect()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Data::Option(None))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(&mut *self)?;
        Ok(Data::Option(Some(Box::new(value))))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Data::Unit)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Data::UnitStruct { name })
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Data::UnitVariant {
            name,
            variant_index,
            variant,
        })
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(&mut *self)?;
        Ok(Data::NewtypeStruct {
            name,
            data: Box::new(value),
        })
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(&mut *self)?;
        Ok(Data::NewtypeVariant {
            name,
            variant_index,
            variant,
            data: Box::new(value),
        })
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.pending_data.push(Data::Seq(if let Some(len) = len {
            Vec::with_capacity(len)
        } else {
            Vec::new()
        }));
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.pending_data.push(Data::Tuple(Vec::with_capacity(len)));
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.pending_data.push(Data::TupleStruct {
            name,
            data: Vec::with_capacity(len),
        });
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.pending_data.push(Data::TupleVariant {
            name,
            variant_index,
            variant,
            data: Vec::with_capacity(len),
        });
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.pending_data.push(Data::Map(Default::default()));
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.pending_data.push(Data::Struct {
            name,
            fields: Default::default(),
        });
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.pending_data.push(Data::StructVariant {
            name,
            variant_index,
            variant,
            fields: Default::default(),
        });
        Ok(self)
    }

    fn is_human_readable(&self) -> bool {
        self.human_readable
    }
}

impl<'a> serde::ser::SerializeSeq for &'a mut Serializer {
    type Ok = Data;

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(&mut **self)?;
        if let Some(Data::Seq(seq)) = self.pending_data.last_mut() {
            seq.push(value);
            Ok(())
        } else {
            Err(Error::InvalidPendingData)
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.pending_data
            .pop()
            .ok_or(Error::MissingPendingData)
    }
}

impl<'a> serde::ser::SerializeTuple for &'a mut Serializer {
    type Ok = Data;

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(&mut **self)?;
        if let Some(Data::Tuple(tuple)) = self.pending_data.last_mut() {
            tuple.push(value);
            Ok(())
        } else {
            Err(Error::InvalidPendingData)
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.pending_data
            .pop()
            .ok_or(Error::MissingPendingData)
    }
}

impl<'a> serde::ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = Data;

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(&mut **self)?;
        if let Some(Data::TupleStruct { data, .. }) = self.pending_data.last_mut() {
            data.push(value);
            Ok(())
        } else {
            Err(Error::InvalidPendingData)
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.pending_data
            .pop()
            .ok_or(Error::MissingPendingData)
    }
}

impl<'a> serde::ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = Data;

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(&mut **self)?;
        if let Some(Data::TupleVariant { data, .. }) = self.pending_data.last_mut() {
            data.push(value);
            Ok(())
        } else {
            Err(Error::InvalidPendingData)
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.pending_data
            .pop()
            .ok_or(Error::MissingPendingData)
    }
}

impl<'a> serde::ser::SerializeMap for &'a mut Serializer {
    type Ok = Data;

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let key = key.serialize(&mut **self)?;
        self.pending_keys.push(key);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let key = self
            .pending_keys
            .pop()
            .ok_or(Error::MissingPendingKey)?;

        let value = value.serialize(&mut **self)?;

        if let Some(Data::Map(map)) = self.pending_data.last_mut() {
            map.push((key, value));
            Ok(())
        } else {
            Err(Error::InvalidPendingData)
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.pending_data
            .pop()
            .ok_or(Error::MissingPendingData)
    }
}

impl<'a> serde::ser::SerializeStruct for &'a mut Serializer {
    type Ok = Data;

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(&mut **self)?;
        if let Some(Data::Struct { fields, .. }) = self.pending_data.last_mut() {
            fields.push((key, value));
            Ok(())
        } else {
            Err(Error::InvalidPendingData)
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.pending_data
            .pop()
            .ok_or(Error::MissingPendingData)
    }
}

impl<'a> serde::ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = Data;

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(&mut **self)?;
        if let Some(Data::StructVariant { fields, .. }) = self.pending_data.last_mut() {
            fields.push((key, value));
            Ok(())
        } else {
            Err(Error::InvalidPendingData)
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.pending_data
            .pop()
            .ok_or(Error::MissingPendingData)
    }
}
