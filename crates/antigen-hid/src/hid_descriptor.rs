/// The HID descriptor contains information about the device's Report Descriptor,
#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct HidDescriptor {
    /// Numeric expression that is the total size of the HID descriptor.
    pub bLength: u8,

    /// Constant name specifying type of HID descriptor.
    pub bDescriptorType_Hid: u8,

    /// Numeric expression identifying the HID Class Specification release
    pub bcdHID: u16,

    /// Numeric expression identifying country code of the localized hardware.
    pub bCountryCode: u8,

    /// Numeric expression specifying the number of class descriptors (always at least one i.e.
    /// Report descriptor)
    pub bNumDescriptors: u8,

    /// Constant name identifying type of class descriptor.
    pub bDescriptorType_Class: u8,

    /// Numeric expression that is the total size of the Report descriptor.
    pub wDescriptorLength: u16,

    /// Numeric expression that is the total size of the optional descriptor.
    pub optional: Vec<(u8, u16)>,
}

#[allow(non_snake_case)]
impl HidDescriptor {
    /// Parse a [`HidDescriptor`] from the bytes interleaved between interface and endpoint descriptors.
    pub fn parse(mut extra: impl Iterator<Item = u8>) -> Self {
        let bLength = extra.next().unwrap();
        let bDescriptorType_Hid = extra.next().unwrap();
        let bcdHID = u16::from_le_bytes([extra.next().unwrap(), extra.next().unwrap()]);
        let bCountryCode = extra.next().unwrap();
        let bNumDescriptors = extra.next().unwrap();
        let bDescriptorType_Class = extra.next().unwrap();
        let wDescriptorLength = u16::from_le_bytes([extra.next().unwrap(), extra.next().unwrap()]);

        let mut optional = vec![];
        while let (Some(bDescriptorType), Some(wDescriptorLengthA), Some(wDescriptorLengthB)) =
            (extra.next(), extra.next(), extra.next())
        {
            optional.push((
                bDescriptorType,
                u16::from_le_bytes([wDescriptorLengthA, wDescriptorLengthB]),
            ))
        }

        HidDescriptor {
            bLength,
            bDescriptorType_Hid,
            bcdHID,
            bCountryCode,
            bNumDescriptors,
            bDescriptorType_Class,
            wDescriptorLength,
            optional,
        }
    }
}

