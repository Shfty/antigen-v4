use std::{borrow::Cow, fmt::Debug};

pub type Error = String;

/// Enum representation of the serde data model
#[derive(Clone, PartialEq, PartialOrd)]
pub enum Data {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Char(char),
    String(String),
    ByteArray(Vec<u8>),
    Option(Option<Box<Data>>),
    Unit,
    UnitStruct {
        name: &'static str,
    },
    UnitVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    },
    NewtypeStruct {
        name: &'static str,
        data: Box<Data>,
    },
    NewtypeVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        data: Box<Data>,
    },
    Seq(Vec<Data>),
    Tuple(Vec<Data>),
    TupleStruct {
        name: &'static str,
        data: Vec<Data>,
    },
    TupleVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        data: Vec<Data>,
    },
    Map(Vec<(Data, Data)>),
    Struct {
        name: &'static str,
        fields: Vec<(&'static str, Data)>,
    },
    StructVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        fields: Vec<(&'static str, Data)>,
    },
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Bool(v) => Debug::fmt(v, f),
            Data::I8(v) => Debug::fmt(v, f),
            Data::I16(v) => Debug::fmt(v, f),
            Data::I32(v) => Debug::fmt(v, f),
            Data::I64(v) => Debug::fmt(v, f),
            Data::I128(v) => Debug::fmt(v, f),
            Data::U8(v) => Debug::fmt(v, f),
            Data::U16(v) => Debug::fmt(v, f),
            Data::U32(v) => Debug::fmt(v, f),
            Data::U64(v) => Debug::fmt(v, f),
            Data::U128(v) => Debug::fmt(v, f),
            Data::F32(v) => Debug::fmt(v, f),
            Data::F64(v) => Debug::fmt(v, f),
            Data::Char(v) => Debug::fmt(v, f),
            Data::String(v) => Debug::fmt(v, f),
            Data::ByteArray(v) => Debug::fmt(v, f),
            Data::Option(v) => Debug::fmt(v, f),
            Data::Unit => f.write_str("()"),
            Data::UnitStruct { name } => f.write_str(name),
            Data::UnitVariant { name, variant, .. } => {
                f.write_fmt(format_args!("{}::{}", name, variant))
            }
            Data::NewtypeStruct { name, data } => f.write_fmt(format_args!("{}({:?})", name, data)),
            Data::NewtypeVariant {
                name,
                variant,
                data,
                ..
            } => f.write_fmt(format_args!("{}::{}({:?})", name, variant, data)),
            Data::Seq(seq) => f.debug_list().entries(seq).finish(),
            Data::Tuple(tuple) => {
                let mut debug = f.debug_tuple("");
                for field in tuple {
                    debug.field(field);
                }
                debug.finish()
            }
            Data::TupleStruct { name, data } => {
                let mut debug = f.debug_tuple(name);
                for field in data {
                    debug.field(field);
                }
                debug.finish()
            }
            Data::TupleVariant {
                name,
                variant,
                data,
                ..
            } => {
                let mut debug = f.debug_tuple(&format!("{}::{}", name, variant));
                for field in data {
                    debug.field(field);
                }
                debug.finish()
            }
            Data::Map(map) => {
                let mut debug = f.debug_map();
                for (key, value) in map {
                    debug.entry(key, value);
                }
                debug.finish()
            }
            Data::Struct { name, fields } => {
                let mut debug = f.debug_struct(name);
                for (key, value) in fields {
                    debug.field(key, value);
                }
                debug.finish()
            }
            Data::StructVariant {
                name,
                variant,
                fields,
                ..
            } => {
                let mut debug = f.debug_struct(&format!("{}::{}", name, variant));
                for (key, value) in fields {
                    debug.field(key, value);
                }
                debug.finish()
            }
        }
    }
}

macro_rules! impl_downcast {
    ($ref_fn:ident, $mut_fn:ident, $ty:ty, $variant:ident) => {
        pub fn $ref_fn(&self) -> Result<&$ty, Error> {
            match self {
                Data::$variant(v) => Ok(v),
                _ => Err(format!("value is not a {}", stringify!($ty))),
            }
        }

        pub fn $mut_fn(&mut self) -> Result<&mut $ty, Error> {
            match self {
                Data::$variant(v) => Ok(v),
                _ => Err(format!("value is not a {}", stringify!($ty))),
            }
        }
    };
}

impl Data {
    pub fn name(&self) -> Cow<'static, str> {
        match self {
            Data::Bool(_) => "bool".into(),
            Data::I8(_) => "i8".into(),
            Data::I16(_) => "i16".into(),
            Data::I32(_) => "i32".into(),
            Data::I64(_) => "i64".into(),
            Data::I128(_) => "i128".into(),
            Data::U8(_) => "u8".into(),
            Data::U16(_) => "u16".into(),
            Data::U32(_) => "u32".into(),
            Data::U64(_) => "u64".into(),
            Data::U128(_) => "u128".into(),
            Data::F32(_) => "f32".into(),
            Data::F64(_) => "f64".into(),
            Data::Char(_) => "char".into(),
            Data::String(_) => "String".into(),
            Data::ByteArray(_) => "ByteArray".into(),
            Data::Option(_) => "Option".into(),
            Data::Unit => "Unit".into(),
            Data::UnitStruct { name } => (*name).into(),
            Data::UnitVariant { name, .. } => (*name).into(),
            Data::NewtypeStruct { name, .. } => (*name).into(),
            Data::NewtypeVariant { name, .. } => (*name).into(),
            Data::Seq(_) => "Seq".into(),
            Data::Tuple(_) => "Tuple".into(),
            Data::TupleStruct { name, .. } => (*name).into(),
            Data::TupleVariant { name, .. } => (*name).into(),
            Data::Map(_) => "Map".into(),
            Data::Struct { name, .. } => (*name).into(),
            Data::StructVariant { name, .. } => (*name).into(),
        }
    }

    impl_downcast!(downcast_bool, downcast_bool_mut, bool, Bool);
    impl_downcast!(downcast_i8, downcast_i8_mut, i8, I8);
    impl_downcast!(downcast_i16, downcast_i16_mut, i16, I16);
    impl_downcast!(downcast_i32, downcast_i32_mut, i32, I32);
    impl_downcast!(downcast_i64, downcast_i64_mut, i64, I64);
    impl_downcast!(downcast_i128, downcast_i128_mut, i128, I128);
    impl_downcast!(downcast_u8, downcast_u8_mut, u8, U8);
    impl_downcast!(downcast_u16, downcast_u16_mut, u16, U16);
    impl_downcast!(downcast_u32, downcast_u32_mut, u32, U32);
    impl_downcast!(downcast_u64, downcast_u64_mut, u64, U64);
    impl_downcast!(downcast_u128, downcast_u128_mut, u128, U128);
    impl_downcast!(downcast_f32, downcast_f32_mut, f32, F32);
    impl_downcast!(downcast_f64, downcast_f64_mut, f64, F64);
    impl_downcast!(downcast_char, downcast_char_mut, char, Char);
    impl_downcast!(downcast_string, downcast_string_mut, String, String);

    impl_downcast!(
        downcast_byte_array,
        downcast_byte_array_mut,
        Vec<u8>,
        ByteArray
    );

    impl_downcast!(
        downcast_option,
        downcast_option_mut,
        Option<Box<Data>>,
        Option
    );

    // Unit
    pub fn downcast_unit(&self) -> Result<(), Error> {
        match self {
            Data::Unit => Ok(()),
            _ => Err("value is not a unit".into()),
        }
    }

    // Unit struct
    pub fn downcast_unit_struct(&self, type_name: &str) -> Result<(), Error> {
        match self {
            Data::UnitStruct { name } => {
                if *name == type_name {
                    Ok(())
                } else {
                    Err(format!("Data {} is not of type {}", name, type_name))
                }
            }
            _ => Err("value is not a unit struct".into()),
        }
    }

    // Unit Variant
    pub fn downcast_unit_variant(&self, type_name: &str, variant_name: &str) -> Result<(), Error> {
        match self {
            Data::UnitVariant { name, variant, .. } => {
                if *name == type_name && *variant == variant_name {
                    Ok(())
                } else {
                    Err(format!(
                        "Data {}::{} is not of type {}::{}",
                        name, variant, type_name, variant_name
                    ))
                }
            }
            _ => Err("value is not a unit variant".into()),
        }
    }

    // NewtypeStruct
    pub fn downcast_newtype_struct(&self, type_name: &str) -> Result<&Box<Data>, Error> {
        match self {
            Data::NewtypeStruct { name, data } => {
                if *name == type_name {
                    Ok(data)
                } else {
                    Err(format!("Data {} is not of type {}", name, type_name))
                }
            }
            _ => Err("value is not a newtype struct".into()),
        }
    }

    pub fn downcast_newtype_struct_mut(
        &mut self,
        type_name: &str,
    ) -> Result<&mut Box<Data>, Error> {
        match self {
            Data::NewtypeStruct { name, data } => {
                if *name == type_name {
                    Ok(data)
                } else {
                    Err(format!("Data {} is not of type {}", name, type_name))
                }
            }
            _ => Err("value is not a newtype struct".into()),
        }
    }

    // NewtypeVariant
    pub fn downcast_newtype_variant(
        &self,
        type_name: &str,
        variant_name: &str,
    ) -> Result<&Box<Data>, Error> {
        match self {
            Data::NewtypeVariant {
                name,
                variant,
                data,
                ..
            } => {
                if *name == type_name && *variant == variant_name {
                    Ok(data)
                } else {
                    Err(format!(
                        "Data {}::{} is not of type {}::{}",
                        name, variant, type_name, variant_name
                    ))
                }
            }
            _ => Err("value is not a newtype variant".into()),
        }
    }

    pub fn downcast_newtype_variant_mut(
        &mut self,
        type_name: &str,
        variant_name: &str,
    ) -> Result<&mut Box<Data>, Error> {
        match self {
            Data::NewtypeVariant {
                name,
                variant,
                data,
                ..
            } => {
                if *name == type_name && *variant == variant_name {
                    Ok(data)
                } else {
                    Err(format!(
                        "Data {}::{} is not of type {}::{}",
                        name, variant, type_name, variant_name
                    ))
                }
            }
            _ => Err("value is not a newtype variant".into()),
        }
    }

    // Sequence
    pub fn downcast_seq(&self) -> Result<&Vec<Data>, Error> {
        match self {
            Data::Seq(v) => Ok(v),
            _ => Err("value is not a sequence".into()),
        }
    }

    pub fn downcast_seq_mut(&mut self) -> Result<&mut Vec<Data>, Error> {
        match self {
            Data::Seq(v) => Ok(v),
            _ => Err("value is not a sequence".into()),
        }
    }

    // Tuple
    pub fn downcast_tuple(&self) -> Result<&Vec<Data>, Error> {
        match self {
            Data::Tuple(v) => Ok(v),
            _ => Err("value is not a tuple".into()),
        }
    }

    pub fn downcast_tuple_mut(&mut self) -> Result<&mut Vec<Data>, Error> {
        match self {
            Data::Tuple(v) => Ok(v),
            _ => Err("value is not a tuple".into()),
        }
    }

    // TupleStruct
    pub fn downcast_tuple_struct(&self, type_name: &str) -> Result<&Vec<Data>, Error> {
        match self {
            Data::TupleStruct { name, data } => {
                if *name == type_name {
                    Ok(data)
                } else {
                    Err(format!("Data {} is not of type {}", name, type_name))
                }
            }
            _ => Err("value is not a tuple struct".into()),
        }
    }

    pub fn downcast_tuple_struct_mut(&mut self, type_name: &str) -> Result<&mut Vec<Data>, Error> {
        match self {
            Data::TupleStruct { name, data } => {
                if *name == type_name {
                    Ok(data)
                } else {
                    Err(format!("Data {} is not of type {}", name, type_name))
                }
            }
            _ => Err("value is not a tuple struct".into()),
        }
    }

    // TupleVariant
    pub fn downcast_tuple_variant(
        &self,
        type_name: &str,
        variant_name: &str,
    ) -> Result<&Vec<Data>, Error> {
        match self {
            Data::TupleVariant {
                name,
                variant,
                data,
                ..
            } => {
                if *name == type_name && *variant == variant_name {
                    Ok(data)
                } else {
                    Err(format!(
                        "Data {}::{} is not of type {}::{}",
                        name, variant, type_name, variant_name
                    ))
                }
            }
            _ => Err("value is not a tuple variant".into()),
        }
    }

    pub fn downcast_tuple_variant_mut(
        &mut self,
        type_name: &str,
        variant_name: &str,
    ) -> Result<&mut Vec<Data>, Error> {
        match self {
            Data::TupleVariant {
                name,
                variant,
                data,
                ..
            } => {
                if *name == type_name && *variant == variant_name {
                    Ok(data)
                } else {
                    Err(format!(
                        "Data {}::{} is not of type {}::{}",
                        name, variant, type_name, variant_name
                    ))
                }
            }
            _ => Err("value is not a tuple variant".into()),
        }
    }

    // Map
    pub fn downcast_map(&self) -> Result<&Vec<(Data, Data)>, Error> {
        match self {
            Data::Map(v) => Ok(v),
            _ => Err("value is not a map".into()),
        }
    }

    pub fn downcast_map_mut(&mut self) -> Result<&mut Vec<(Data, Data)>, Error> {
        match self {
            Data::Map(v) => Ok(v),
            _ => Err("value is not a map".into()),
        }
    }

    // Struct
    pub fn downcast_struct(
        &self,
        type_name: &'static str,
    ) -> Result<&Vec<(&'static str, Data)>, Error> {
        match self {
            Data::Struct { name, fields } => {
                if *name == type_name {
                    Ok(fields)
                } else {
                    Err(format!("Data {} is not of type {}", name, type_name))
                }
            }
            _ => Err("value is not a struct".into()),
        }
    }

    pub fn downcast_struct_mut(
        &mut self,
        type_name: &'static str,
    ) -> Result<&mut Vec<(&'static str, Data)>, Error> {
        match self {
            Data::Struct { name, fields } => {
                if *name == type_name {
                    Ok(fields)
                } else {
                    Err(format!("Data {} is not of type {}", name, type_name))
                }
            }
            _ => Err("value is not a struct".into()),
        }
    }

    // StructVariant
    pub fn downcast_struct_variant(
        &self,
        type_name: &str,
        variant_name: &str,
    ) -> Result<&Vec<(&'static str, Data)>, Error> {
        match self {
            Data::StructVariant {
                name,
                variant,
                fields,
                ..
            } => {
                if *name == type_name && *variant == variant_name {
                    Ok(fields)
                } else {
                    Err(format!(
                        "Data {}::{} is not of type {}::{}",
                        name, variant, type_name, variant_name,
                    ))
                }
            }
            _ => Err("value is not a struct variant".into()),
        }
    }

    pub fn downcast_struct_variant_mut(
        &mut self,
        type_name: &str,
        variant_name: &str,
    ) -> Result<&mut Vec<(&'static str, Data)>, Error> {
        match self {
            Data::StructVariant {
                name,
                variant,
                fields,
                ..
            } => {
                if *name == type_name && *variant == variant_name {
                    Ok(fields)
                } else {
                    Err(format!(
                        "Data {}::{} is not of type {}::{}",
                        name, variant, type_name, variant_name
                    ))
                }
            }
            _ => Err("value is not a struct variant".into()),
        }
    }
}

pub trait DataFields {
    type Key;
    type Value;

    fn get(&self, key: Self::Key) -> Option<&Self::Value>;
    fn get_mut(&mut self, key: Self::Key) -> Option<&mut Self::Value>;
}

impl<K: PartialEq, V> DataFields for Vec<(K, V)> {
    type Key = K;
    type Value = V;

    fn get(&self, key: Self::Key) -> Option<&Self::Value> {
        self.iter()
            .find_map(|(k, v)| if *k == key { Some(v) } else { None })
    }

    fn get_mut(&mut self, key: Self::Key) -> Option<&mut Self::Value> {
        self.iter_mut()
            .find_map(|(k, v)| if *k == key { Some(v) } else { None })
    }
}
