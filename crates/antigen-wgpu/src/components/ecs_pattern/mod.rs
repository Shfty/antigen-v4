//! Work-in-progress components for building renderers directly in ECS,
//! rather than needing a black-box wrapper as per the current pattern

mod shader_component;
mod pipeline_layout_component;

pub use shader_component::*;
pub use pipeline_layout_component::*;
