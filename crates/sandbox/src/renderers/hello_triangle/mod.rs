use antigen_wgpu::{RenderPass, WgpuManager};
use wgpu::{PipelineLayout, RenderPipeline, ShaderModule};

#[derive(Debug)]
pub struct TriangleRenderer {
    shader_module: ShaderModule,
    pipeline_layout: PipelineLayout,

    pipeline: Option<RenderPipeline>,
}

impl TriangleRenderer {
    pub fn new(wgpu_manager: &WgpuManager) -> Self {
        let device = wgpu_manager.device();

        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        TriangleRenderer {
            shader_module,
            pipeline_layout,
            pipeline: None,
        }
    }
}

impl RenderPass for TriangleRenderer {
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        wgpu_manager: &WgpuManager,
        view: &wgpu::TextureView,
        format: wgpu::ColorTargetState,
    ) {
        if self.pipeline.is_none() {
            self.pipeline = Some(wgpu_manager.device().create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&self.pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &self.shader_module,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &self.shader_module,
                        entry_point: "fs_main",
                        targets: &[format],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                },
            ));
        }

        let pipeline = self.pipeline.as_ref().unwrap();

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        rpass.set_pipeline(&pipeline);
        rpass.draw(0..3, 0..1);
    }
}

