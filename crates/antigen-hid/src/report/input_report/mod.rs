//! HID Input Report
mod report_value;

pub use report_value::*;

use crate::report::report_descriptor::ReportDescriptorItem;

/// An input report parsed from raw HID input
#[derive(Debug, Default, Clone)]
pub struct InputReport(Vec<InputValue>);

impl InputReport {
    pub fn new(values: Vec<InputValue>) -> Self {
        InputReport(values)
    }

    pub fn iter(&self) -> impl Iterator<Item = &InputValue> {
        self.0.iter()
    }
}

/// Data item information and current value
#[derive(Debug, Copy, Clone)]
pub struct InputValue {
    data_item: ReportDescriptorItem,
    report_value: ReportValue,
}

impl InputValue {
    pub fn new(data_item: ReportDescriptorItem, report_value: ReportValue) -> Self {
        InputValue {
            data_item,
            report_value,
        }
    }

    pub fn data_item(&self) -> &ReportDescriptorItem {
        &self.data_item
    }

    pub fn report_value(&self) -> &ReportValue {
        &self.report_value
    }

    pub fn logical(&self) -> &ReportValue {
        &self.report_value
    }

    pub fn physical(&self) -> Option<ReportValue> {
        let logical_min = if let Some(logical_min) = self.data_item.global_state.logical_minimum {
            logical_min
        } else {
            return None;
        };

        let logical_max = if let Some(logical_max) = self.data_item.global_state.logical_maximum {
            logical_max
        } else {
            return None;
        };

        let logical_range = logical_max - logical_min;

        let physical_min = if let Some(physical_min) = self.data_item.global_state.physical_minimum
        {
            physical_min
        } else {
            return None;
        };

        let physical_max = if let Some(physical_max) = self.data_item.global_state.physical_maximum
        {
            physical_max
        } else {
            return None;
        };

        let physical_range = physical_max - physical_min;

        None
    }

    pub fn normalized(&self) -> Option<ReportValue> {
        None
    }
}
