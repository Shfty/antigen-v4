use serde::ser::SerializeStruct;
use std::{ops::Deref, sync::Arc};
use wgpu::Buffer;

#[derive(Debug, Clone)]
pub struct BufferComponent<T> {
    pub buffer: Arc<Buffer>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Deref for BufferComponent<T> {
    type Target = Arc<Buffer>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<T> serde::Serialize for BufferComponent<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("BufferComponent", 1)?;
        s.serialize_field("T", std::any::type_name::<T>())?;
        s.end()
    }
}

impl<'de, T> serde::Deserialize<'de> for BufferComponent<T> {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl<T> From<Arc<Buffer>> for BufferComponent<T> {
    fn from(buffer: Arc<Buffer>) -> Self {
        BufferComponent {
            buffer,
            _phantom: Default::default(),
        }
    }
}
