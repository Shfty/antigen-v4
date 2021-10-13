//! Reflection framework for introspecting Rust types at runtime using an in-memory serde format

pub mod data;
pub mod serializer;
pub mod path;
pub mod index;

use serde::Serialize;

/// Convenience function for serializing a value into reflection data
pub fn to_data<T>(value: T, human_readable: bool) -> Result<data::Data, serializer::Error>
where
    T: Serialize,
{
    value.serialize(&mut serializer::Serializer::new(human_readable))
}

