use rusb::UsbContext;

use crate::report::{
    global_state::UsagePage,
    input_report::{InputReport, InputValue, ReportValue},
    report_descriptor::{item_data::Constness, DataType, ReportDescriptor},
};

/// Device abstration containing all information necessary to read and interpret input
#[derive(Debug, Clone)]
pub struct Device<T: UsbContext> {
    device: rusb::Device<T>,
    interrupt_input: u8,
    report_desc: ReportDescriptor,
}

impl<T: UsbContext> Device<T> {
    pub fn new(
        device: rusb::Device<T>,
        interrupt_input: u8,
        report_desc: ReportDescriptor,
    ) -> Self {
        Device {
            device,
            interrupt_input,
            report_desc,
        }
    }

    pub fn parse(&self, input: impl Iterator<Item = u8>) -> InputReport {
        parse_input_report(&self.report_desc, input)
    }

    pub fn input_buffer_len(&self) -> usize {
        let mut bits = 0;

        for item in self.report_desc.iter() {
            if item.data_type != DataType::Feature {
                let report_size = item.global_state.report_size.expect("Unknown report size");
                let report_count = item
                    .global_state
                    .report_count
                    .expect("Unknown report count");
                let size = report_size as u16 * report_count as u16;
                bits += size + (size % 8)
            }
        }

        (bits / 8) as usize
    }

    pub fn open(&self) -> Result<rusb::DeviceHandle<T>, rusb::Error> {
        self.device.open()
    }

    pub fn interrupt_input(&self) -> u8 {
        self.interrupt_input
    }
}

/// Parse the bytes of an input report into a vector of input values
pub fn parse_input_report(
    report_desc: &ReportDescriptor,
    mut input: impl Iterator<Item = u8>,
) -> InputReport {
    let mut current_byte = 0u8;
    let mut current_len = 0u8;

    let mut read = |count: usize| -> ReportValue {
        let mut bits = vec![];

        for _ in 0..count {
            if current_len == 0 {
                current_byte = input.next().unwrap();
                current_len = 8;
            }

            bits.insert(0, current_byte & 1 > 0);
            current_byte = current_byte >> 1;
            current_len -= 1;
        }

        let mut out = 0;
        for bit in &bits {
            out = (out << 1) | if *bit { 1 } else { 0 }
        }

        match bits.len() {
            1 => ReportValue::Bool(out > 0),
            2..=8 => ReportValue::U8(out as u8),
            9..=16 => ReportValue::U16(out as u16),
            17..=32 => ReportValue::U32(out as u32),
            _ => panic!("Invalid value size"),
        }
    };

    let mut out = vec![];
    let report_id = if report_desc
        .iter()
        .find(|item| item.global_state.report_id.is_some())
        .is_some()
    {
        match read(8) {
            ReportValue::U8(report_id) => Some(report_id),
            _ => unreachable!(),
        }
    } else {
        None
    };

    for item in report_desc.iter() {
        if let (Some(report_id), Some(item_report_id)) = (report_id, item.global_state.report_id) {
            if item_report_id != report_id {
                continue;
            }
        }

        if item.data_type != DataType::Feature {
            let data = read(
                item.global_state.report_size.unwrap() as usize
                    * item.global_state.report_count.unwrap() as usize,
            );

            // Skip over const padding
            if item.main_data.constness == Constness::Constant {
                continue;
            }

            if let Some(usage_page) = item.global_state.usage_page {
                match usage_page {
                    UsagePage::Button => {
                        out.push(InputValue::new(item.clone(), data));
                    }
                    UsagePage::GenericDesktop => {
                        if item.local_state.usage.is_some() {
                            out.push(InputValue::new(item.clone(), data));
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    InputReport::new(out)
}
