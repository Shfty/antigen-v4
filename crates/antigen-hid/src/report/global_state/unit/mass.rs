/// HID Mass Unit
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitMass {
    None,
    Gram,
    Slug
}

impl From<u32> for UnitMass {
    fn from(v: u32) -> Self {
        match v {
            0x0 => UnitMass::None,
            0x1..=0x2 => UnitMass::Gram,
            0x3..=0x4 => UnitMass::Slug,
            _ => panic!("Invalid unit")
        }
    }
}
