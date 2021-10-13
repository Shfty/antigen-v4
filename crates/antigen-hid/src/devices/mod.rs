//! Abstractions to handle communicating with USB HID devices
mod device;
mod device_id;
mod device_name;

pub use device::*;
pub use device_id::*;
pub use device_name::*;

use crate::report::report_descriptor::{ReportDescriptor, ReportDescriptorData};
use rusb::{Direction, TransferType, UsbContext};
use std::collections::BTreeMap;

/// Collection of [`Device`] structs, their names, and report descriptor fallbacks
#[derive(Debug, Default, Clone)]
pub struct Devices<T: UsbContext> {
    devices: BTreeMap<DeviceId, Device<T>>,
    device_names: BTreeMap<DeviceId, DeviceName>,
}

impl<T: UsbContext> Devices<T> {
    pub fn new() -> Self {
        Devices {
            devices: Default::default(),
            device_names: Default::default(),
        }
    }

    pub fn insert(&mut self, device_id: DeviceId, device: Device<T>) -> Option<Device<T>> {
        self.devices.insert(device_id, device)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DeviceId, &Device<T>)> {
        self.devices.iter()
    }

    pub fn take(
        self,
    ) -> (
        impl Iterator<Item = (DeviceId, Device<T>)>,
        BTreeMap<DeviceId, DeviceName>,
    ) {
        (self.devices.into_iter(), self.device_names)
    }

    pub fn enumerate(
        &mut self,
        context: T,
        report_descriptors: &ReportDescriptorData,
    ) -> Result<(), rusb::Error> {
        for device in context.devices()?.iter() {
            let device_desc = device.device_descriptor()?;

            let vid = device_desc.vendor_id();
            let pid = device_desc.product_id();

            println!("VID: 0x{:04x}, PID: 0x{:04x}", vid, pid);

            let device_handle = if let Ok(device_handle) = device.open() {
                device_handle
            } else {
                println!("\tFailed to open device handle");
                continue;
            };

            let manufacturer = device_handle.read_manufacturer_string_ascii(&device_desc)?;
            let product = device_handle.read_product_string_ascii(&device_desc)?;

            println!("\t{} {}", manufacturer, product);

            'configs: for i in 0..device_desc.num_configurations() {
                let config_descriptor = device.config_descriptor(i)?;
                for interface in config_descriptor.interfaces() {
                    for interface_desc in interface.descriptors() {
                        if interface_desc.class_code() != rusb::constants::LIBUSB_CLASS_HID {
                            println!("\tNon-HID interface");
                            continue;
                        }

                        for endpoint_desc in interface_desc.endpoint_descriptors() {
                            if endpoint_desc.transfer_type() != TransferType::Interrupt
                                || endpoint_desc.direction() != Direction::In
                            {
                                continue;
                            }

                            let interrupt_input = endpoint_desc.address();

                            let report_desc = if let Some(report) = report_descriptors.get(vid, pid)
                            {
                                ReportDescriptor::new(report.iter().copied())
                            } else {
                                println!("\tInvalid report descriptor");
                                continue;
                            };

                            let device_id = DeviceId::new(vid, pid);
                            let device = Device::new(device, interrupt_input, report_desc);
                            let device_name = DeviceName::new(manufacturer, product);

                            self.devices.insert(device_id, device);
                            self.device_names.insert(device_id, device_name);

                            break 'configs;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
