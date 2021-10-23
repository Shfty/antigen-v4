use cgmath::{One, Quaternion};

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Orientation(Quaternion<f32>);

impl Default for Orientation {
    fn default() -> Self {
        Orientation(Quaternion::one())
    }
}

impl std::ops::Deref for Orientation {
    type Target = Quaternion<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Orientation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(Orientation);
