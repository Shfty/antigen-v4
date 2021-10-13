//! Metadata describing Input / Output / Feature report data

/// Input / Output / Feature item data
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemData {
    pub constness: Constness,
    pub dimensionality: Dimensionality,
    pub absolute_relative: AbsoluteRelative,
    pub wrapping: Wrapping,
    pub linearity: Linearity,
    pub state_preference: StatePreference,
    pub nullability: Nullability,
    pub volatility: Volatility,
    pub byte_stream: ByteStream,
}

impl ItemData {
    pub fn new(data: &[u8]) -> Self {
        let v = if data.len() == 1 {
            data[0] as u16
        } else {
            u16::from_le_bytes([data[0], data[1]])
        };

        let constness = if v & 1 == 0 {
            Constness::Data
        } else {
            Constness::Constant
        };

        let dimensionality = if (v >> 1) & 1 == 0 {
            Dimensionality::Array
        } else {
            Dimensionality::Variable
        };

        let absolute_relative = if (v >> 2) & 1 == 0 {
            AbsoluteRelative::Absolute
        } else {
            AbsoluteRelative::Relative
        };

        let wrapping = if (v >> 3) & 1 == 0 {
            Wrapping::NoWrap
        } else {
            Wrapping::Wrap
        };

        let linearity = if (v >> 4) & 1 == 0 {
            Linearity::Linear
        } else {
            Linearity::NonLinear
        };

        let state_preference = if (v >> 5) & 1 == 0 {
            StatePreference::PreferredState
        } else {
            StatePreference::NoPreferred
        };

        let nullability = if (v >> 6) & 1 == 0 {
            Nullability::NoNullPosition
        } else {
            Nullability::NullState
        };

        let volatility = if (v >> 7) & 1 == 0 {
            Volatility::NonVolatile
        } else {
            Volatility::Volatile
        };

        let byte_stream = if (v >> 8) & 1 == 0 {
            ByteStream::BitField
        } else {
            ByteStream::BufferedBytes
        };

        ItemData {
            constness,
            dimensionality,
            absolute_relative,
            wrapping,
            linearity,
            state_preference,
            nullability,
            volatility,
            byte_stream,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Constness {
    Data,
    Constant,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dimensionality {
    Array,
    Variable,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbsoluteRelative {
    Absolute,
    Relative,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Wrapping {
    NoWrap,
    Wrap,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Linearity {
    Linear,
    NonLinear,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatePreference {
    PreferredState,
    NoPreferred,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nullability {
    NoNullPosition,
    NullState,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Volatility {
    NonVolatile,
    Volatile,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ByteStream {
    BitField,
    BufferedBytes,
}
