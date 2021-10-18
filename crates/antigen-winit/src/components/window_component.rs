use crate::window_state::WindowState;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct WindowComponent(pub WindowState);

legion_debugger::register_component!(WindowComponent);
