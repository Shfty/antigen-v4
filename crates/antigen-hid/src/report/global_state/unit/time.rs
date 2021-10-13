/// HID Time Unit
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitTime {
    None,
    Seconds,
}

impl From<u32> for UnitTime {
    fn from(v: u32) -> Self {
        match v {
            0x0 => UnitTime::None,
            0x1..=0x4 => UnitTime::Seconds,
            _ => panic!("Invalid unit")
        }
    }
}

