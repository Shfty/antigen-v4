use cgmath::{One, Quaternion};
use on_change::{OnChange, OnChangeTrait};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Orientation(OnChange<Quaternion<f32>>);

impl Default for Orientation {
    fn default() -> Self {
        Orientation(OnChange::new_dirty(Quaternion::one()))
    }
}

impl Orientation {
    pub fn new(quaternion: Quaternion<f32>) -> Self {
        Orientation(OnChange::new_dirty(quaternion))
    }
}

impl std::ops::Deref for Orientation {
    type Target = OnChange<Quaternion<f32>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Orientation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl OnChangeTrait<Quaternion<f32>> for Orientation {
    fn take_change(&self) -> Option<&Quaternion<f32>> {
        self.0.take_change()
    }
}

legion_debugger::register_component!(Orientation);
