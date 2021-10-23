use cgmath::{Matrix4, SquareMatrix};
use on_change::{OnChange, OnChangeTrait};

#[repr(C)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ViewProjectionMatrix(pub OnChange<Matrix4<f32>>);

impl std::ops::Deref for ViewProjectionMatrix {
    type Target = OnChange<Matrix4<f32>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ViewProjectionMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for ViewProjectionMatrix {
    fn default() -> Self {
        ViewProjectionMatrix(OnChange::new_dirty(Matrix4::identity()))
    }
}

impl OnChangeTrait<Matrix4<f32>> for ViewProjectionMatrix {
    fn take_change(&self) -> Option<&Matrix4<f32>> {
        self.0.take_change()
    }
}

legion_debugger::register_component!(ViewProjectionMatrix);
