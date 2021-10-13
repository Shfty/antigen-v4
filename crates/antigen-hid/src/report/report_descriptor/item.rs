use crate::report::{
    global_state::GlobalState, local_state::LocalState, report_descriptor::ItemData,
};

/// Report item wrapping global state, local state, and item data
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReportDescriptorItem {
    pub data_type: DataType,
    pub global_state: GlobalState,
    pub local_state: LocalState,
    pub main_data: ItemData,
}

/// Data item variant
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataType {
    Input,
    Output,
    Feature,
}
