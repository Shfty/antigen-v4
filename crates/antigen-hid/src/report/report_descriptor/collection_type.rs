/// HID Collection Type
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CollectionType {
    Physical,
    Application,
    Logical,
    Report,
    NamedArray,
    UsageSwitch,
    UsageModifier,
    Reserved,
    VendorDefined,
}

impl From<u8> for CollectionType {
    fn from(v: u8) -> Self {
        match v {
            0x00 => CollectionType::Physical,
            0x01 => CollectionType::Application,
            0x02 => CollectionType::Logical,
            0x03 => CollectionType::Report,
            0x04 => CollectionType::NamedArray,
            0x05 => CollectionType::UsageSwitch,
            0x06 => CollectionType::UsageModifier,
            0x07..=0x7F => CollectionType::Reserved,
            0x80..=0xFF => CollectionType::VendorDefined,
        }
    }
}

