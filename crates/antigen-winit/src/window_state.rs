use std::sync::Arc;

use winit::window::Window;

#[derive(Debug)]
pub enum WindowState {
    Invalid,
    Pending,
    Valid(Arc<Window>),
    Closed,
}

impl serde::Serialize for WindowState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            WindowState::Invalid => serializer.serialize_unit_variant("WindowState", 0, "Invalid"),
            WindowState::Pending => serializer.serialize_unit_variant("WindowState", 1, "Pending"),
            WindowState::Valid(_) => serializer.serialize_unit_variant("WindowState", 2, "Valid"),
            WindowState::Closed => serializer.serialize_unit_variant("WindowState", 3, "Closed"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for WindowState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState::Invalid
    }
}
