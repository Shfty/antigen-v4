/// Top-level category defining the usage of a report descriptor main item
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UsagePage {
    GenericDesktop,
    SimulationControls,
    VRControls,
    SportsControls,
    GameControls,
    GenericDeviceControls,
    KeyboardKeypad,
    Led,
    Button,
    Ordinal,
    TelephonyDevice,
    Consumer,
    Digitizers,
    Unicode,
    AlphanumericDisplay,
    MedicalInstrument,
    Monitor,
    Power,
    BarCodeScanner,
    Scale,
    MagneticStripeReadingDevices,
    ReservedPointOfSale,
    CameraControl,
    Arcade,
    VendorDefined,
    Reserved(u32),
}

impl From<u32> for UsagePage {
    fn from(v: u32) -> Self {
        match v {
            0x01 => UsagePage::GenericDesktop,
            0x02 => UsagePage::SimulationControls,
            0x03 => UsagePage::VRControls,
            0x04 => UsagePage::SportsControls,
            0x05 => UsagePage::GameControls,
            0x06 => UsagePage::GenericDeviceControls,
            0x07 => UsagePage::KeyboardKeypad,
            0x08 => UsagePage::Led,
            0x09 => UsagePage::Button,
            0x0A => UsagePage::Ordinal,
            0x0B => UsagePage::TelephonyDevice,
            0x0C => UsagePage::Consumer,
            0x0D => UsagePage::Digitizers,
            0x10 => UsagePage::Unicode,
            0x14 => UsagePage::AlphanumericDisplay,
            0x40 => UsagePage::MedicalInstrument,
            0x80..=0x83 => UsagePage::Monitor,
            0x84..=0x87 => UsagePage::Power,
            0x8C => UsagePage::BarCodeScanner,
            0x8D => UsagePage::Scale,
            0x8E => UsagePage::MagneticStripeReadingDevices,
            0x8F => UsagePage::ReservedPointOfSale,
            0x90 => UsagePage::CameraControl,
            0x91 => UsagePage::Arcade,
            0xFF00..=0xFFFF => UsagePage::VendorDefined,
            usage_page => UsagePage::Reserved(usage_page),
        }
    }
}

