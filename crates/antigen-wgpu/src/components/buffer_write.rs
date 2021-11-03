use std::marker::PhantomData;

use crate::CastSlice;
use legion::Entity;
use on_change::OnChangeTrait;
use wgpu::BufferAddress;

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct BufferWrite<T: OnChangeTrait<D>, D: CastSlice<u8>> {
    from: Option<Entity>,
    to: Option<Entity>,
    offset: BufferAddress,
    _phantom: PhantomData<(T, D)>,
}

impl<T: OnChangeTrait<D>, D: CastSlice<u8>> BufferWrite<T, D> {
    pub fn new(from: Option<Entity>, to: Option<Entity>, offset: BufferAddress) -> Self {
        BufferWrite {
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

    pub fn set_offset(&mut self, offset: wgpu::BufferAddress) {
        self.offset = offset
    }
}
