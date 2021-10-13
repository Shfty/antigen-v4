/// HID Temperature Unit
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitTemperature {
    None,
    Kelvin,
    Farenheit,
}

impl From<u32> for UnitTemperature {
    fn from(v: u32) -> Self {
        match v {
            0x0 => UnitTemperature::None,
            0x1..=0x2 => UnitTemperature::Kelvin,
            0x3..=0x4 => UnitTemperature::Farenheit,
            _ => panic!("Invalid unit")
        }
    }
}

