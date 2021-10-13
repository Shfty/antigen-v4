//! HID library using rusb
//!
//!     Note: Complete HID report descriptors can't be fetched via the windows libusb backend
//!         Can retrieve them succesfully using usbdk, but that requires installing a driver
//!
//!         For now these descriptors must be manually dumped and loaded externally for
//!         correct behavior on windows

pub mod devices;
pub mod report;

mod hid_descriptor;

pub use hid_descriptor::*;
