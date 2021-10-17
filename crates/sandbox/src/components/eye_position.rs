use cgmath::Point3;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EyePosition(pub Point3<f32>);

impl Default for EyePosition {
    fn default() -> Self {
        EyePosition(Point3::new(0.0, 0.0, 0.0))
    }
}

legion_debugger::register_component!(EyePosition);
