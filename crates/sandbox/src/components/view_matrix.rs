use cgmath::{Matrix4, SquareMatrix};

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ViewMatrix(cgmath::Matrix4<f32>);

impl Default for ViewMatrix {
    fn default() -> Self {
        ViewMatrix(Matrix4::identity())
    }
}

impl std::ops::Deref for ViewMatrix {
    type Target = Matrix4<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ViewMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


legion_debugger::register_component!(ViewMatrix);
