use cgmath::{Matrix4, SquareMatrix};
use on_change::{OnChange,OnChangeTrait};

#[repr(C)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ViewProjectionMatrix(pub OnChange<cgmath::Matrix4<f32>>);

impl Default for ViewProjectionMatrix {
    fn default() -> Self {
        ViewProjectionMatrix(OnChange::new_dirty(Matrix4::identity()))
    }
}

impl OnChangeTrait<cgmath::Matrix4<f32>> for ViewProjectionMatrix {
    fn take_change(&self) -> Option<&cgmath::Matrix4<f32>> {
        self.0.take_change()
    }
}

legion_debugger::register_component!(ViewProjectionMatrix);
