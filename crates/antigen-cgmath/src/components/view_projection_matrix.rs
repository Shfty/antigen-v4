use cgmath::{Matrix4, SquareMatrix};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ViewProjectionMatrix(cgmath::Matrix4<f32>);

impl AsRef<[u8]> for ViewProjectionMatrix {
    fn as_ref(&self) -> &[u8] {
        let mx: &[f32; 16] = self.0.as_ref();
        bytemuck::cast_slice(mx)
    }
}

impl Default for ViewProjectionMatrix {
    fn default() -> Self {
        ViewProjectionMatrix(Matrix4::identity())
    }
}

impl std::ops::Deref for ViewProjectionMatrix {
    type Target = Matrix4<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ViewProjectionMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(ViewProjectionMatrix);
