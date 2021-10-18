use std::marker::PhantomData;

use legion::Entity;
use serde::ser::SerializeStruct;
use wgpu::{Extent3d, ImageDataLayout};

use crate::TextureData;

#[derive(Debug, Copy, Clone)]
pub struct TextureWrite<T: TextureData> {
    from: Option<Entity>,
    to: Option<Entity>,
    data_layout: ImageDataLayout,
    extent: Extent3d,
    _phantom: PhantomData<T>,
}

impl<T: TextureData> serde::Serialize for TextureWrite<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("TextureWrite", 2)?;
        s.serialize_field("from", &self.from)?;
        s.serialize_field("to", &self.to)?;
        s.end()
    }
}

impl<'de, T: TextureData> serde::Deserialize<'de> for TextureWrite<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl<T: TextureData> TextureWrite<T> {
    pub fn new(
        from: Option<Entity>,
        to: Option<Entity>,
        data_layout: ImageDataLayout,
        extent: Extent3d,
    ) -> Self {
        TextureWrite {
            from,
            to,
            data_layout,
            extent,
            _phantom: Default::default(),
        }
    }

    pub fn from_entity(&self) -> Option<&Entity> {
        self.from.as_ref()
    }

    pub fn to_entity(&self) -> Option<&Entity> {
        self.to.as_ref()
    }

    pub fn data_layout(&self) -> &ImageDataLayout {
        &self.data_layout
    }

    pub fn extent(&self) -> &Extent3d {
        &self.extent
    }
}
