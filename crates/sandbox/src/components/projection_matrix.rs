#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectionMatrix(cgmath::Matrix4<f32>);

legion_debugger::register_component!(ProjectionMatrix);
