#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AspectRatio(pub f32);

impl Default for AspectRatio {
    fn default() -> Self {
        AspectRatio(1.0)
    }
}

impl std::ops::Deref for AspectRatio {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AspectRatio {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(AspectRatio);
