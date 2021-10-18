use std::{ops::Deref, sync::Arc};
use wgpu::Texture;
use serde::ser::SerializeStruct;

#[derive(Debug, Clone)]
pub struct TextureComponent(Arc<Texture>);

impl Deref for TextureComponent {
    type Target = Arc<Texture>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl serde::Serialize for TextureComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer
            .serialize_struct("TextureComponent", 0)?
            .end()
    }
}

impl<'de> serde::Deserialize<'de> for TextureComponent {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl From<Arc<Texture>> for TextureComponent {
    fn from(v: Arc<Texture>) -> Self {
        TextureComponent(v)
    }
}

legion_debugger::register_component!(TextureComponent);
