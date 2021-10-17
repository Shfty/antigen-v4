#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NearPlane(pub f32);

impl Default for NearPlane {
    fn default() -> Self {
        NearPlane(1.0)
    }
}

impl std::ops::Deref for NearPlane {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for NearPlane {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(NearPlane);
