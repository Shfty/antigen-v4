//! Units of measurement

mod system;
mod length;
mod mass;
mod time;
mod temperature;
mod current;
mod luminous_intensity;

pub use system::*;
pub use length::*;
pub use mass::*;
pub use time::*;
pub use temperature::*;
pub use current::*;
pub use luminous_intensity::*;

/// HID Unit
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unit {
    System(UnitSystem),
    Length(UnitLength),
    Mass(UnitMass),
    Time(UnitTime),
    Temperature(UnitTemperature),
    Current(UnitCurrent),
    LuminousIntensity(UnitLuminousIntensity),
    Reserved(u32),
}

impl From<u32> for Unit {
    fn from(v: u32) -> Self {
        match v & 0xF0 {
            0x00 => Unit::System(UnitSystem::from(v & 0x0F)),
            0x10 => Unit::Length(UnitLength::from(v & 0x0F)),
            0x20 => Unit::Mass(UnitMass::from(v & 0x0F)),
            0x30 => Unit::Time(UnitTime::from(v & 0x0F)),
            0x40 => Unit::Temperature(UnitTemperature::from(v & 0x0F)),
            0x50 => Unit::Current(UnitCurrent::from(v & 0x0F)),
            0x60 => Unit::LuminousIntensity(UnitLuminousIntensity::from(v & 0x0F)),
            0x70 => Unit::Reserved(v & 0x0F),
            _ => panic!("Invalid unit")
        }
    }
}

