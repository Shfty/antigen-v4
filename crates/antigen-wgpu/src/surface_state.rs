use parking_lot::RwLock;
use std::sync::Arc;
use wgpu::SurfaceConfiguration;

#[derive(Debug)]
pub enum SurfaceState {
    Invalid,
    Pending,
    Valid(Arc<RwLock<SurfaceConfiguration>>),
    Destroyed,
}

impl Default for SurfaceState {
    fn default() -> Self {
        SurfaceState::Invalid
    }
}

impl serde::Serialize for SurfaceState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SurfaceState::Invalid => serializer.serialize_str("Invalid"),
            SurfaceState::Pending => serializer.serialize_str("Pending"),
            SurfaceState::Valid(_) => serializer.serialize_str("Valid"),
            SurfaceState::Destroyed => serializer.serialize_str("Destroyed"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SurfaceState {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}
