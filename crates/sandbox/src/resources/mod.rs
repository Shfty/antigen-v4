mod crossterm;

pub use self::crossterm::*;

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum MainLoopState {
    Running,
    Break,
}

impl Default for MainLoopState {
    fn default() -> Self {
        MainLoopState::Running
    }
}

