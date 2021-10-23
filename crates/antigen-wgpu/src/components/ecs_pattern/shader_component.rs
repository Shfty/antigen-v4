use serde::ser::SerializeStruct;
use wgpu::{ShaderModule, ShaderModuleDescriptor};

pub struct ShaderComponent {
    pub pending_desc: Option<ShaderModuleDescriptor<'static>>,
    pub shader: Option<ShaderModule>,
}

impl serde::Serialize for ShaderComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_struct("ShaderComponent", 0)?.end()
    }
}

impl<'de> serde::Deserialize<'de> for ShaderComponent {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

legion_debugger::register_component!(ShaderComponent);

#[legion::system(par_for_each)]
pub fn create_shaders(shader: &mut ShaderComponent, #[resource] device: &wgpu::Device) {
    if let Some(desc) = shader.pending_desc.take() {
        shader.shader = Some(device.create_shader_module(&desc));
    }
}
