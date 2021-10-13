use crate as legion_debugger;

/// Newtype wrapping a function that registers a component type with a [`legion::Registry`]
pub struct ComponentRegistrar(pub fn(&mut legion::serialize::Registry<String>));

// Initialize component registry
plugin_registry::init!(ComponentRegistrar);

// Register primitive types
legion_debugger::register_component!(bool);
legion_debugger::register_component!(i8);
legion_debugger::register_component!(i16);
legion_debugger::register_component!(i32);
legion_debugger::register_component!(i64);
legion_debugger::register_component!(i128);
legion_debugger::register_component!(u8);
legion_debugger::register_component!(u16);
legion_debugger::register_component!(u32);
legion_debugger::register_component!(u64);
legion_debugger::register_component!(u128);
legion_debugger::register_component!(f32);
legion_debugger::register_component!(f64);
legion_debugger::register_component!(char);
legion_debugger::register_component!(String);

/// Returns a pair of world serializers configured using registered components
pub fn world_serializers() -> (legion::serialize::Registry<String>, legion::serialize::Canon) {
    // Serialization
    let mut registry = legion::serialize::Registry::<String>::default();

    // Register custom types
    for ComponentRegistrar(registry_func) in plugin_registry::iter!(ComponentRegistrar) {
        registry_func(&mut registry);
    }

    (registry, Default::default())
}
