use cgmath::Point3;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LookAt(pub Point3<f32>);

impl Default for LookAt {
    fn default() -> Self {
        LookAt(Point3::new(0.0, 0.0, 0.0))
    }
}

legion_debugger::register_component!(LookAt);
