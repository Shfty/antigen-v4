use cgmath::{Matrix4, SquareMatrix};

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectionMatrix(Matrix4<f32>);

impl Default for ProjectionMatrix {
    fn default() -> Self {
        ProjectionMatrix(Matrix4::identity())
    }
}

impl std::ops::Deref for ProjectionMatrix {
    type Target = Matrix4<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ProjectionMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(ProjectionMatrix);
