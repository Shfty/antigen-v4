#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

legion_debugger::register_component!(Velocity);
