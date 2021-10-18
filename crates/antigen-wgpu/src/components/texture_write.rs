use std::marker::PhantomData;

use legion::Entity;
use on_change::OnChangeTrait;
use serde::ser::SerializeStruct;
use wgpu::{Extent3d, ImageDataLayout};

use crate::CastSlice;

#[derive(Debug, Copy, Clone)]
pub struct TextureWrite<T: OnChangeTrait<D>, D: CastSlice<u8>> {
    from: Option<Entity>,
    to: Option<Entity>,
    data_layout: ImageDataLayout,
    extent: Extent3d,
    _phantom: PhantomData<(T, D)>,
}

impl<T: OnChangeTrait<D>, D: CastSlice<u8>> serde::Serialize for TextureWrite<T, D> {
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

impl<'de, T: OnChangeTrait<D>, D: CastSlice<u8>> serde::Deserialize<'de> for TextureWrite<T, D> {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl<T: OnChangeTrait<D>, D: CastSlice<u8>> TextureWrite<T, D> {
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
