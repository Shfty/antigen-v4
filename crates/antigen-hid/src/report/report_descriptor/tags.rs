//! Intermediate representation for parsed HID reports
//! Used to parse byte arrays into rust types, which are then parsed into a [`ReportDescriptor`]

use crate::report::{
    global_state::UsagePage,
    report_descriptor::{CollectionType, ItemData},
};

#[derive(Debug, Copy, Clone)]
pub enum MainTag {
    Input(ItemData),
    Output(ItemData),
    Feature(ItemData),
    Collection(CollectionType),
    EndCollection,
    Reserved,
}

impl MainTag {
    pub fn new(v: u8, data: &[u8]) -> Self {
        match v {
            8 => MainTag::Input(ItemData::new(data)),
            9 => MainTag::Output(ItemData::new(data)),
            10 => MainTag::Collection(CollectionType::from(data[0])),
            11 => MainTag::Feature(ItemData::new(data)),
            12 => MainTag::EndCollection,
            13..=16 => MainTag::Reserved,
            _ => panic!("Can't create MainTag from a >4bit &value"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GlobalTag {
    UsagePage(UsagePage),
    LogicalMinimum(i32),
    LogicalMaximum(i32),
    PhysicalMinimum(i32),
    PhysicalMaximum(i32),
    UnitExponent(u32),
    Unit(u32),
    ReportSize(u32),
    ReportId(u32),
    ReportCount(u32),
    Push,
    Pop,
    Reserved,
}

impl GlobalTag {
    pub fn new(v: u8, data: &[u8]) -> Self {
        let data_as_u32 = || match data.len() {
            1 => u8::from_le_bytes([data[0]]) as u32,
            2 => u16::from_le_bytes([data[0], data[1]]) as u32,
            4 => u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            _ => panic!("Invalid usage page"),
        };

        let data_as_i32 = || match data.len() {
            1 => i8::from_le_bytes([data[0]]) as i32,
            2 => i16::from_le_bytes([data[0], data[1]]) as i32,
            4 => i32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            _ => panic!("Invalid usage page"),
        };

        match v {
            0 => GlobalTag::UsagePage(UsagePage::from(data_as_u32())),
            1 => GlobalTag::LogicalMinimum(data_as_i32()),
            2 => GlobalTag::LogicalMaximum(data_as_i32()),
            3 => GlobalTag::PhysicalMinimum(data_as_i32()),
            4 => GlobalTag::PhysicalMaximum(data_as_i32()),
            5 => GlobalTag::UnitExponent(data_as_u32()),
            6 => GlobalTag::Unit(data_as_u32()),
            7 => GlobalTag::ReportSize(data_as_u32()),
            8 => GlobalTag::ReportId(data_as_u32()),
            9 => GlobalTag::ReportCount(data_as_u32()),
            10 => GlobalTag::Push,
            11 => GlobalTag::Pop,
            12..=16 => GlobalTag::Reserved,
            _ => panic!("Can't create GlobalTag from a >4bit value"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LocalTag {
    Usage(u32),
    UsageMinimum(u32),
    UsageMaximum(u32),
    DesignatorIndex(u32),
    DesignatorMinimum(u32),
    DesignatorMaximum(u32),
    StringIndex(u32),
    StringMinimum(u32),
    StringMaximum(u32),
    Delimiter(u32),
    Reserved(u32),
}

impl LocalTag {
    pub fn new(v: u8, data: &[u8]) -> Self {
        match v {
            0 => LocalTag::Usage(data[0] as u32),
            1 => LocalTag::UsageMinimum(data[0] as u32),
            2 => LocalTag::UsageMaximum(data[0] as u32),
            3 => LocalTag::DesignatorIndex(data[0] as u32),
            4 => LocalTag::DesignatorMinimum(data[0] as u32),
            5 => LocalTag::DesignatorMaximum(data[0] as u32),
            6 => LocalTag::StringIndex(data[0] as u32),
            7 => LocalTag::StringMinimum(data[0] as u32),
            8 => LocalTag::StringMaximum(data[0] as u32),
            9 => LocalTag::Delimiter(data[0] as u32),
            10..=16 => LocalTag::Reserved(v as u32),
            _ => panic!("Can't create LocalTag from a >4bit value"),
        }
    }
}

/// An item parsed from a report descriptor
#[derive(Debug, Copy, Clone)]
pub enum ItemTag {
    Main(MainTag),
    Global(GlobalTag),
    Local(LocalTag),
    Reserved,
}

impl ItemTag {
    pub fn parse(mut report: impl Iterator<Item = u8>) -> Option<ItemTag> {
        let byte = report.next()?;

        let size = byte & 0b00000011;
        let item_type = ItemType::from((byte >> 2) & 0b00000011);
        let tag = (byte >> 4) & 0b00001111;

        let bytes = match size {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 4,
            _ => panic!("Size value exceeds 2bits"),
        };

        let mut data = vec![];
        for _ in 0..bytes {
            data.push(report.next().expect("Missing data bytes"));
        }

        let item_tag = match item_type {
            ItemType::Main => ItemTag::Main(MainTag::new(tag, &data)),
            ItemType::Global => ItemTag::Global(GlobalTag::new(tag, &data)),
            ItemType::Local => ItemTag::Local(LocalTag::new(tag, &data)),
            ItemType::Reserved => ItemTag::Reserved,
        };

        Some(item_tag)
    }
}

/// A set of item tags parsed from a byte array representing a report descriptor
#[derive(Debug, Default, Clone)]
pub struct ItemTags(pub Vec<ItemTag>);

impl ItemTags {
    pub fn parse(mut report: impl Iterator<Item = u8>) -> ItemTags {
        let mut items = vec![];
        while let Some(item) = ItemTag::parse(&mut report) {
            items.push(item);
        }
        ItemTags(items)
    }
}

/// ItemTag variants
#[derive(Debug, Copy, Clone)]
pub enum ItemType {
    Main,
    Global,
    Local,
    Reserved,
}

impl From<u8> for ItemType {
    fn from(v: u8) -> Self {
        match v {
            0 => ItemType::Main,
            1 => ItemType::Global,
            2 => ItemType::Local,
            3 => ItemType::Reserved,
            _ => panic!("Can't create ItemType from a >2bit value"),
        }
    }
}
