//! HID Global State
pub mod unit;
mod usage_page;

use unit::*;
pub use usage_page::*;

/// Global parse state table
///
/// Global state applies to all following main items (Input / Output / Feature) encountered in
/// a report descriptor.
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalState {
    pub usage_page: Option<UsagePage>,
    pub logical_minimum: Option<i32>,
    pub logical_maximum: Option<i32>,
    pub physical_minimum: Option<i32>,
    pub physical_maximum: Option<i32>,
    pub unit_exponent: Option<u32>,
    pub unit: Option<Unit>,
    pub report_size: Option<u32>,
    pub report_id: Option<u8>,
    pub report_count: Option<u32>,
}

impl std::fmt::Debug for GlobalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("GlobalState");

        if let Some(usage_page) = self.usage_page {
            s.field("usage_page", &usage_page);
        }

        if let Some(logical_minimum) = self.logical_minimum {
            s.field("logical_minimum", &logical_minimum);
        }

        if let Some(logical_maximum) = self.logical_maximum {
            s.field("logical_maximum", &logical_maximum);
        }

        if let Some(physical_minimum) = self.physical_minimum {
            s.field("physical_minimum", &physical_minimum);
        }

        if let Some(physical_maximum) = self.physical_maximum {
            s.field("physical_maximum", &physical_maximum);
        }

        if let Some(unit_exponent) = self.unit_exponent {
            s.field("unit_exponent", &unit_exponent);
        }

        if let Some(unit) = self.unit {
            s.field("unit", &unit);
        }

        if let Some(report_size) = self.report_size {
            s.field("report_size", &report_size);
        }

        if let Some(report_id) = self.report_id {
            s.field("report_id", &report_id);
        }

        if let Some(report_count) = self.report_count {
            s.field("report_count", &report_count);
        }

        s.finish()
    }
}
