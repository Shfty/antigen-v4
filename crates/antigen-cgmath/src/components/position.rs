use cgmath::Vector3;
use on_change::OnChange;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Position3d(OnChange<Vector3<f32>>);

impl std::ops::Deref for Position3d {
    type Target = OnChange<Vector3<f32>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Position3d {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Position3d {
    pub fn new(position: cgmath::Vector3<f32>) -> Self {
        Position3d(OnChange::new_dirty(position))
    }
}

impl on_change::OnChangeTrait<cgmath::Vector3<f32>> for Position3d {
    fn take_change(&self) -> Option<&cgmath::Vector3<f32>> {
        self.0.take_change()
    }
}

legion_debugger::register_component!(Position3d);
