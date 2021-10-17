#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FarPlane(pub f32);

impl Default for FarPlane {
    fn default() -> Self {
        FarPlane(10.0)
    }
}

impl std::ops::Deref for FarPlane {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for FarPlane {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(FarPlane);
