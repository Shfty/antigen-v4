use std::rc::Rc;

use antigen_wgpu::{RenderPass, WgpuManager};
use lazy::Lazy;
use wgpu::RenderPipeline;

type PipelineCtx = (Rc<wgpu::Device>, wgpu::TextureFormat);
pub struct TriangleRenderer {
    pipeline: Lazy<RenderPipeline, PipelineCtx>,
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

        let pipeline = Lazy::new(Box::new(
            move |(device, format): (Rc<wgpu::Device>, wgpu::TextureFormat)| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "fs_main",
                        targets: &[format.into()],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                })
            },
        ));

        TriangleRenderer { pipeline }
    }
}

impl RenderPass for TriangleRenderer {
    fn render(
        &mut self,
        wgpu_manager: &WgpuManager,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        config: &wgpu::SurfaceConfiguration,
    ) {
        let pipeline = self.pipeline.get((wgpu_manager.device(), config.format));

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
