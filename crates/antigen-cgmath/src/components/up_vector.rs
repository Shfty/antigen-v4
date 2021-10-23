use cgmath::Vector3;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpVector(pub Vector3<f32>);

impl Default for UpVector {
    fn default() -> Self {
        UpVector(Vector3::unit_y())
    }
}

legion_debugger::register_component!(UpVector);
