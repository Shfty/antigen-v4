/// HID System Unit
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitSystem {
    None,
    SiLinear,
    SiRotation,
    EnglishLinear,
    EnglishRotation,
}

impl From<u32> for UnitSystem {
    fn from(v: u32) -> Self {
        match v {
            0x0 => UnitSystem::None,
            0x1 => UnitSystem::SiLinear,
            0x2 => UnitSystem::SiRotation,
            0x3 => UnitSystem::EnglishLinear,
            0x4 => UnitSystem::EnglishRotation,
            _ => panic!("Invalid unit")
        }
    }
}

