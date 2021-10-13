//! HID Report Descriptor
pub mod item_data;
pub mod tags;

mod collection_type;
mod item;
mod report_descriptor_data;

pub use collection_type::*;
pub use item::*;
pub use report_descriptor_data::*;

use item_data::*;
use tags::*;

use rusb::{DeviceHandle, Direction, Recipient, RequestType};

use crate::HidDescriptor;

use super::{
    global_state::{GlobalState, UsagePage},
    local_state::{GenericDesktopUsage, LocalState, Usage},
};

use std::time::Duration;

/// The Report Descriptor describes the format of a device's output data,
/// and is used to parse it from raw bytes into structured HID data.
///
/// This information is retrieved from the device via control request,
/// but is unavailable on the default Windows USB backend due to missing functionality.
///
/// Using an alternate libusb backend - such as libusbk or usbdk - allows this
/// information to be read from the device under Windows, but requires the
/// installation of custom drivers.
///
/// Thus, report descriptors must be dumped from a device and provided to
/// the program externally in order to parse a device's output under Windows.
#[derive(Debug, Clone)]
pub struct ReportDescriptor(Vec<ReportDescriptorItem>);

impl ReportDescriptor {
    pub fn new(report_desc_bytes: impl Iterator<Item = u8>) -> Self {
        let ItemTags(items) = ItemTags::parse(report_desc_bytes);
        let mut collection_stack: Vec<CollectionType> = Default::default();

        // Global state
        let mut global_state_stack = vec![GlobalState::default()];
        let mut local_state = LocalState::default();
        let mut usage_stack: Vec<Usage> = vec![];

        let mut data: Vec<ReportDescriptorItem> = vec![];

        for item in items {
            match item {
                ItemTag::Main(main) => {
                    match main {
                        MainTag::Input(main_data)
                        | MainTag::Output(main_data)
                        | MainTag::Feature(main_data) => {
                            let gs = *global_state_stack.last().unwrap();
                            let report_count = gs.report_count.unwrap();

                            for _ in 0..report_count {
                                let usage = match usage_stack.len() {
                                    0 => None,
                                    1 => Some(usage_stack[0]),
                                    _ => Some(usage_stack.remove(0)),
                                };

                                let mut gs = gs;
                                if let Some(report_count) = &mut gs.report_count {
                                    *report_count = 1;
                                }

                                let mut ls = local_state;
                                ls.usage = usage;

                                data.push(ReportDescriptorItem {
                                    data_type: match main {
                                        MainTag::Input(_) => DataType::Input,
                                        MainTag::Output(_) => DataType::Output,
                                        MainTag::Feature(_) => DataType::Feature,
                                        _ => unreachable!(),
                                    },
                                    global_state: gs,
                                    local_state: ls,
                                    main_data,
                                })
                            }
                        }
                        MainTag::Collection(collection) => collection_stack.push(collection),
                        MainTag::EndCollection => {
                            collection_stack.pop();
                        }
                        MainTag::Reserved => (),
                    }

                    local_state = Default::default();
                    usage_stack.clear();
                }
                ItemTag::Global(global) => match global {
                    GlobalTag::UsagePage(v) => {
                        global_state_stack.last_mut().unwrap().usage_page = Some(v)
                    }
                    GlobalTag::LogicalMinimum(v) => {
                        global_state_stack.last_mut().unwrap().logical_minimum = Some(v)
                    }
                    GlobalTag::LogicalMaximum(v) => {
                        global_state_stack.last_mut().unwrap().logical_maximum = Some(v)
                    }
                    GlobalTag::PhysicalMinimum(v) => {
                        global_state_stack.last_mut().unwrap().physical_minimum = Some(v)
                    }
                    GlobalTag::PhysicalMaximum(v) => {
                        global_state_stack.last_mut().unwrap().physical_maximum = Some(v)
                    }
                    GlobalTag::UnitExponent(v) => {
                        global_state_stack.last_mut().unwrap().unit_exponent = Some(v)
                    }
                    GlobalTag::Unit(v) => {
                        global_state_stack.last_mut().unwrap().unit = Some(v.into())
                    }
                    GlobalTag::ReportSize(v) => {
                        global_state_stack.last_mut().unwrap().report_size = Some(v)
                    }
                    GlobalTag::ReportId(v) => {
                        global_state_stack.last_mut().unwrap().report_id = Some(v as u8)
                    }
                    GlobalTag::ReportCount(v) => {
                        global_state_stack.last_mut().unwrap().report_count = Some(v)
                    }
                    GlobalTag::Push => {
                        global_state_stack.push(*global_state_stack.last().unwrap());
                    }
                    GlobalTag::Pop => {
                        global_state_stack.pop();
                    }
                    GlobalTag::Reserved => (),
                },
                ItemTag::Local(local) => match local {
                    LocalTag::Usage(usage) => {
                        if let Some(usage_page) = global_state_stack.last().unwrap().usage_page {
                            let usage = match usage_page {
                                UsagePage::GenericDesktop => {
                                    Usage::GenericDesktop(GenericDesktopUsage::from(usage))
                                }
                                _ => Usage::Other(usage),
                            };
                            usage_stack.push(usage);
                        }
                    }
                    LocalTag::UsageMinimum(usage_min) => {
                        local_state.usage_minimum = Some(usage_min)
                    }
                    LocalTag::UsageMaximum(usage_max) => {
                        local_state.usage_maximum = Some(usage_max)
                    }
                    LocalTag::DesignatorIndex(designator_index) => {
                        local_state.designator_index = Some(designator_index)
                    }
                    LocalTag::DesignatorMinimum(designator_min) => {
                        local_state.designator_minimum = Some(designator_min)
                    }
                    LocalTag::DesignatorMaximum(designator_max) => {
                        local_state.designator_maximum = Some(designator_max)
                    }
                    LocalTag::StringIndex(string_index) => {
                        local_state.string_index = Some(string_index)
                    }
                    LocalTag::StringMinimum(string_min) => {
                        local_state.string_minimum = Some(string_min)
                    }
                    LocalTag::StringMaximum(string_max) => {
                        local_state.string_maximum = Some(string_max)
                    }
                    LocalTag::Delimiter(delimiter) => local_state.delimiter = Some(delimiter),
                    LocalTag::Reserved(_) => (),
                },
                ItemTag::Reserved => (),
            }
        }

        ReportDescriptor(data)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ReportDescriptorItem> {
        self.0.iter()
    }

    /// Read a [`ReportDescriptor`] from a [`DeviceHandle`]
    pub fn read<T: rusb::UsbContext>(
        &mut self,
        device_handle: &DeviceHandle<T>,
        hid_desc: &HidDescriptor,
    ) -> Result<ReportDescriptor, ReadError> {
        let mut buf = vec![0; hid_desc.wDescriptorLength as usize];

        let len = device_handle.read_control(
            rusb::request_type(Direction::In, RequestType::Standard, Recipient::Interface),
            rusb::constants::LIBUSB_REQUEST_GET_DESCRIPTOR,
            (hid_desc.bDescriptorType_Class as u16) << 8,
            0,
            buf.as_mut_slice(),
            Duration::from_secs(1000),
        )?;

        if len as u16 == hid_desc.wDescriptorLength {
            Ok(ReportDescriptor::new(buf.iter().copied()))
        } else {
            Err(ReadError::InvalidReportDescriptor)
        }
    }
}

/// An error reading a [`ReportDescriptor`] from a [`DeviceHandle`]
#[derive(Debug, Copy, Clone)]
pub enum ReadError {
    InvalidReportDescriptor,
    Rusb(rusb::Error),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::InvalidReportDescriptor => f.write_str("Invalid report descriptor"),
            ReadError::Rusb(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ReadError::InvalidReportDescriptor => None,
            ReadError::Rusb(e) => Some(e),
        }
    }
}

impl From<rusb::Error> for ReadError {
    fn from(e: rusb::Error) -> Self {
        ReadError::Rusb(e)
    }
}
