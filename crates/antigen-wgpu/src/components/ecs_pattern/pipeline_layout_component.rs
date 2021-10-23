use serde::ser::SerializeStruct;
use wgpu::{PipelineLayout, PipelineLayoutDescriptor};

pub struct PipelineLayoutComponent {
    pub pending_desc: Option<PipelineLayoutDescriptor<'static>>,
    pub pipeline_layout: Option<PipelineLayout>,
}

impl serde::Serialize for PipelineLayoutComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer
            .serialize_struct("PipelineLayoutComponent", 0)?
            .end()
    }
}

impl<'de> serde::Deserialize<'de> for PipelineLayoutComponent {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

legion_debugger::register_component!(PipelineLayoutComponent);

#[legion::system(par_for_each)]
pub fn create_pipeline_layouts(
    pipeline_layout: &mut PipelineLayoutComponent,
    #[resource] device: &wgpu::Device,
) {
    if let Some(desc) = pipeline_layout.pending_desc.take() {
        pipeline_layout.pipeline_layout = Some(device.create_pipeline_layout(&desc));
    }
}
