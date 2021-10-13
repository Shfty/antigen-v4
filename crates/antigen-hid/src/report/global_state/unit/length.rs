/// HID Length Unit
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitLength {
    None,
    Centimeter,
    Radians,
    Inch,
    Degrees,
}

impl From<u32> for UnitLength {
    fn from(v: u32) -> Self {
        match v {
            0x0 => UnitLength::None,
            0x1 => UnitLength::Centimeter,
            0x2 => UnitLength::Radians,
            0x3 => UnitLength::Inch,
            0x4 => UnitLength::Degrees,
            _ => panic!("Invalid unit")
        }
    }
}

