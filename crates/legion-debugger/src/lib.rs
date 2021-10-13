/// Legion Debugger
///
/// Provides reflection data parsing for legion's serialization formats, and methods for displaying
/// such data at runtime
mod parser;
mod registry;

pub use parser::*;
pub use registry::*;

pub use legion_debugger_macros::register_component;
pub use legion_debugger_macros::register_resource;

