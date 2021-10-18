use std::marker::PhantomData;

use legion::Entity;
use wgpu::BufferAddress;

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct UniformWrite<T: AsRef<[u8]>> {
    from: Option<Entity>,
    to: Option<Entity>,
    offset: BufferAddress,
    _phantom: PhantomData<T>,
}

impl<T: AsRef<[u8]>> UniformWrite<T> {
    pub fn new(from: Option<Entity>, to: Option<Entity>, offset: BufferAddress) -> Self {
        UniformWrite {
            from,
            to,
            offset,
            _phantom: Default::default(),
        }
    }

    pub fn from_entity(&self) -> Option<&Entity> {
        self.from.as_ref()
    }

    pub fn to_entity(&self) -> Option<&Entity> {
        self.to.as_ref()
    }

    pub fn offset(&self) -> wgpu::BufferAddress {
        self.offset
    }
}

