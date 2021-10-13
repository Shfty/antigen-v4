//! HID Local State
mod usage;

pub use usage::*;

/// Local parse state table
///
/// Local state applies to the next main item (Input / Output / Feature) encountered in
/// a report descriptor.
///
/// Usages are special-case: If multiple usages are present,
/// they will apply sequentially to the next N main items, repeating the last usage
/// if more main items are encountered.
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalState {
    pub usage: Option<Usage>,
    pub usage_minimum: Option<u32>,
    pub usage_maximum: Option<u32>,
    pub designator_index: Option<u32>,
    pub designator_minimum: Option<u32>,
    pub designator_maximum: Option<u32>,
    pub string_index: Option<u32>,
    pub string_minimum: Option<u32>,
    pub string_maximum: Option<u32>,
    pub delimiter: Option<u32>,
}

impl std::fmt::Debug for LocalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("LocalState");

        if let Some(usage) = self.usage {
            s.field("usage", &usage);
        }

        if let Some(usage_minimum) = self.usage_minimum {
            s.field("usage_minimum", &usage_minimum);
        }

        if let Some(usage_maximum) = self.usage_maximum {
            s.field("usage_maximum", &usage_maximum);
        }

        if let Some(designator_index) = self.designator_index {
            s.field("designator_index", &designator_index);
        }

        if let Some(designator_minimum) = self.designator_minimum {
            s.field("designator_minimum", &designator_minimum);
        }

        if let Some(designator_maximum) = self.designator_maximum {
            s.field("designator_maximum", &designator_maximum);
        }

        if let Some(string_index) = self.string_index {
            s.field("string_index", &string_index);
        }

        if let Some(string_minimum) = self.string_minimum {
            s.field("string_minimum", &string_minimum);
        }

        if let Some(string_maximum) = self.string_maximum {
            s.field("string_maximum", &string_maximum);
        }

        if let Some(delimiter) = self.delimiter {
            s.field("delimiter", &delimiter);
        }

        s.finish()
    }
}

