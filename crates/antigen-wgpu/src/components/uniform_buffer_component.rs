use std::{ops::Deref, sync::Arc};
use wgpu::Buffer;
use serde::ser::SerializeStruct;

#[derive(Debug, Clone)]
pub struct UniformBufferComponent(Arc<Buffer>);

impl Deref for UniformBufferComponent {
    type Target = Arc<Buffer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl serde::Serialize for UniformBufferComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer
            .serialize_struct("UniformBufferComponent", 0)?
            .end()
    }
}

impl<'de> serde::Deserialize<'de> for UniformBufferComponent {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl From<Arc<Buffer>> for UniformBufferComponent {
    fn from(v: Arc<Buffer>) -> Self {
        UniformBufferComponent(v)
    }
}

legion_debugger::register_component!(UniformBufferComponent);
