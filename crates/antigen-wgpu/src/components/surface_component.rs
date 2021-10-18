use crate::SurfaceState;
use serde::ser::SerializeStruct;
use wgpu::{PresentMode, SurfaceConfiguration};
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug)]
pub struct SurfaceComponent {
    state: SurfaceState,
    present_mode: PresentMode,
}

impl Default for SurfaceComponent {
    fn default() -> Self {
        SurfaceComponent {
            state: Default::default(),
            present_mode: PresentMode::Mailbox,
        }
    }
}

impl SurfaceComponent {
    pub fn new(present_mode: PresentMode) -> Self {
        SurfaceComponent {
            present_mode,
            ..Default::default()
        }
    }

    pub fn state(&self) -> &SurfaceState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut SurfaceState {
        &mut self.state
    }

    pub fn set_invalid(&mut self) {
        self.state = SurfaceState::Invalid;
    }

    pub fn set_pending(&mut self) {
        self.state = SurfaceState::Pending;
    }

    pub fn set_valid(&mut self, config: Arc<RwLock<SurfaceConfiguration>>) {
        self.state = SurfaceState::Valid(config);
    }

    pub fn set_destroyed(&mut self) {
        self.state = SurfaceState::Destroyed;
    }
}

impl serde::Serialize for SurfaceComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("SurfaceComponent", 2)?;
        s.serialize_field("state", &self.state)?;
        s.serialize_field(
            "present_mode",
            match self.present_mode {
                PresentMode::Immediate => "Immediate",
                PresentMode::Mailbox => "Mailbox",
                PresentMode::Fifo => "Fifo",
            },
        )?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for SurfaceComponent {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

legion_debugger::register_component!(SurfaceComponent);
