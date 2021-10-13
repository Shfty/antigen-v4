/// Input report value variants
#[derive(Debug, Copy, Clone)]
pub enum ReportValue {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
}

impl std::fmt::Display for ReportValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportValue::Bool(v) => v.fmt(f),
            ReportValue::U8(v) => v.fmt(f),
            ReportValue::U16(v) => v.fmt(f),
            ReportValue::U32(v) => v.fmt(f),
        }
    }
}

