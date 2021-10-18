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

