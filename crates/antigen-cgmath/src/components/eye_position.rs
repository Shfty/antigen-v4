use std::ops::{Deref, DerefMut};

use cgmath::Point3;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EyePosition(pub Point3<f32>);

impl Default for EyePosition {
    fn default() -> Self {
        EyePosition(Point3::new(0.0, 0.0, 0.0))
    }
}

impl Deref for EyePosition {
    type Target = Point3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EyePosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(EyePosition);
