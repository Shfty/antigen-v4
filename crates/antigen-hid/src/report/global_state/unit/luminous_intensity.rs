/// HID Luminous Intensity Unit
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitLuminousIntensity {
    None,
    Candela,
}

impl From<u32> for UnitLuminousIntensity {
    fn from(v: u32) -> Self {
        match v {
            0x0 => UnitLuminousIntensity::None,
            0x1..=0x4 => UnitLuminousIntensity::Candela,
            _ => panic!("Invalid unit")
        }
    }
}

