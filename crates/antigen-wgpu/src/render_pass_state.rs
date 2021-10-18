#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum RenderPassState {
    Invalid,
    Pending,
    Registered,
    Unregistered,
}

impl Default for RenderPassState {
    fn default() -> Self {
        RenderPassState::Invalid
    }
}

