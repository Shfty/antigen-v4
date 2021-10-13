/// HID Current Unit
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitCurrent {
    None,
    Ampere,
}

impl From<u32> for UnitCurrent {
    fn from(v: u32) -> Self {
        match v {
            0x0 => UnitCurrent::None,
            0x1..=0x4 => UnitCurrent::Ampere,
            _ => panic!("Invalid unit")
        }
    }
}

