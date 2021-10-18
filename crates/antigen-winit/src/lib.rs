pub mod components;
pub mod systems;
pub mod window_manager;
mod window_state;

pub use window_state::*;

use legion::World;
use remote_channel::*;
use window_manager::WindowManager;
use winit::event_loop::EventLoopWindowTarget;

pub type WinitRequester = RemoteRequester<WindowManager, EventLoopWindowTarget<()>, World>;
pub type WinitResponder = RemoteResponder<WindowManager, EventLoopWindowTarget<()>, World>;
