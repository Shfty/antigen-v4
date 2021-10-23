#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LinearVelocity3d(pub cgmath::Vector3<f32>);

impl std::ops::Deref for LinearVelocity3d {
    type Target = cgmath::Vector3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for LinearVelocity3d {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl LinearVelocity3d {
    pub fn new(position: cgmath::Vector3<f32>) -> Self {
        LinearVelocity3d(position)
    }

    pub fn position(&self) -> &cgmath::Vector3<f32> {
        &self.0
    }
}

legion_debugger::register_component!(LinearVelocity3d);
