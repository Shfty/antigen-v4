use crate::{RenderPassState, RenderPassId};
use serde::ser::SerializeStruct;

#[derive(Debug, Clone, Default)]
pub struct RenderPassComponent {
    passes: Vec<(RenderPassState, RenderPassId)>,
}

impl serde::Serialize for RenderPassComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("RenderPassComponent", 2)?;
        s.serialize_field("passes", &self.passes)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for RenderPassComponent {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl RenderPassComponent {
    pub fn add_render_pass(&mut self, render_pass: RenderPassId) {
        self.passes.push((Default::default(), render_pass))
    }

    pub fn remove_render_pass(&mut self, render_pass: RenderPassId) {
        self.passes
            .iter_mut()
            .find(|(_, pass)| *pass == render_pass)
            .unwrap()
            .0 = RenderPassState::Unregistered;
    }

    pub fn passes(&self) -> &Vec<(RenderPassState, RenderPassId)> {
        &self.passes
    }

    pub fn passes_mut(&mut self) -> &mut Vec<(RenderPassState, RenderPassId)> {
        &mut self.passes
    }
}

legion_debugger::register_component!(RenderPassComponent);
