use std::collections::BTreeMap;

/// Enum representation of the [`tracing`] field data model
#[derive(Debug)]
pub enum TraceField {
    Debug(String),
    I64(i64),
    U64(u64),
    Bool(bool),
    Str(String),
    Error(String),
}

/// Collection of named [`TraceField`]s
pub type TraceFields = BTreeMap<&'static str, TraceField>;

