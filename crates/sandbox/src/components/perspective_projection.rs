#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PerspectiveProjection;

legion_debugger::register_component!(PerspectiveProjection);
