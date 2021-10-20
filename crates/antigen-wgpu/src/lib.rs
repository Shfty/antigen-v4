pub mod components;
pub mod systems;

mod surface_state;
mod render_pass_state;
mod render_pass;
mod wgpu_manager;
mod cast_slice;

pub use surface_state::*;
pub use render_pass::*;
pub use wgpu_manager::*;
pub use render_pass_state::*;
pub use cast_slice::*;

use atomic_id::atomic_id;
use legion::World;
use remote_channel::{RemoteRequester, RemoteResponder};

pub type WgpuRequester = RemoteRequester<WgpuManager, (), World>;
pub type WgpuResponder = RemoteResponder<WgpuManager, (), World>;

atomic_id!(NEXT_RENDER_PASS_ID, RenderPassId);
atomic_id!(NEXT_BUFFER_ID, BufferId);

/// Indirect draw data for use with an indirect buffer
#[repr(C)]
#[derive(Copy, Clone, serde::Serialize, serde::Deserialize, bytemuck::Zeroable, bytemuck::Pod)]
pub struct DrawIndirect {
    pub vertex_count: u32, // The number of vertices to draw.
    pub instance_count: u32, // The number of instances to draw.
    pub base_vertex: u32, // The Index of the first vertex to draw.
    pub base_instance: u32, // The instance ID of the first instance to draw.
}

/// Indirect multi draw count
#[repr(C)]
#[derive(Copy, Clone, serde::Serialize, serde::Deserialize, bytemuck::Zeroable, bytemuck::Pod)]
pub struct DrawIndirectCount {
    pub count: u32, // Number of draw calls to issue.
}

/// Indexed indirect draw data for use with an indirect buffer
#[repr(C)]
#[derive(Copy, Clone, serde::Serialize, serde::Deserialize, bytemuck::Zeroable, bytemuck::Pod)]
pub struct DrawIndexedIndirect {
    pub vertex_count: u32,   // The number of vertices to draw.
    pub instance_count: u32, // The number of instances to draw.
    pub base_index: u32,     // The base index within the index buffer.
    pub vertex_offset: i32, // The value added to the vertex index before indexing into the vertex buffer.
    pub base_instance: u32, // The instance ID of the first instance to draw.
}

impl CastSlice<u8> for DrawIndexedIndirect {
    fn cast_slice(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

/// Indexed indirect multi draw count
#[repr(C)]
#[derive(Copy, Clone, serde::Serialize, serde::Deserialize, bytemuck::Zeroable, bytemuck::Pod)]
pub struct DrawIndexedIndirectCount {
    pub count: u32, // Number of draw calls to issue.
}

