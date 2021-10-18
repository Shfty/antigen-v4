use cgmath::Vector3;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LookTo(pub Vector3<f32>);

impl Default for LookTo {
    fn default() -> Self {
        LookTo(Vector3::new(0.0, 0.0, 0.0))
    }
}

legion_debugger::register_component!(LookTo);
