use std::borrow::Cow;

use antigen_wgpu::{RenderPass, WgpuManager};
use legion::Entity;
use wgpu::{Buffer, ColorTargetState, PipelineLayout, RenderBundle, RenderPipeline, ShaderModule, ShaderModuleDescriptor, ShaderSource, SurfaceConfiguration, TextureView};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 2],
    _color: [f32; 4],
}

#[derive(Debug)]
pub struct MsaaLinesRenderer {
    surface_entity: Entity,
    bundle: Option<RenderBundle>,
    shader: ShaderModule,
    pipeline_layout: PipelineLayout,
    multisampled_framebuffer: Option<TextureView>,
    vertex_buffer: Buffer,
    vertex_count: u32,
    sample_count: u32,

    prev_width: u32,
    prev_height: u32,
}

impl MsaaLinesRenderer {
    const SHADER_MODULE_DESCRIPTOR: ShaderModuleDescriptor<'static> = ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    };

    pub fn new(wgpu_manager: &WgpuManager, surface_entity: Entity) -> Self {
        let device = wgpu_manager.device();

        let sample_count = 4;

        let shader = device.create_shader_module(&Self::SHADER_MODULE_DESCRIPTOR);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let mut vertex_data = vec![];

        let max = 50;
        for i in 0..max {
            let percent = i as f32 / max as f32;
            let (sin, cos) = (percent * 2.0 * std::f32::consts::PI).sin_cos();
            vertex_data.push(Vertex {
                _pos: [0.0, 0.0],
                _color: [1.0, -sin, cos, 1.0],
            });
            vertex_data.push(Vertex {
                _pos: [1.0 * cos, 1.0 * sin],
                _color: [sin, -cos, 1.0, 1.0],
            });
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let vertex_count = vertex_data.len() as u32;

        MsaaLinesRenderer {
            surface_entity,
            bundle: None,
            shader,
            pipeline_layout,
            multisampled_framebuffer: None,
            vertex_buffer,
            vertex_count,
            sample_count,
            prev_width: Default::default(),
            prev_height: Default::default(),
        }
    }

    fn create_bundle(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shader: &wgpu::ShaderModule,
        pipeline_layout: &wgpu::PipelineLayout,
        sample_count: u32,
        vertex_buffer: &wgpu::Buffer,
        vertex_count: u32,
        format: ColorTargetState,
    ) -> wgpu::RenderBundle {
        log::info!("sample_count: {}", sample_count);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[format.clone().into()],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                ..Default::default()
            },
        });
        let mut encoder =
            device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                label: None,
                color_formats: &[format.format],
                depth_stencil: None,
                sample_count,
            });
        encoder.set_pipeline(&pipeline);
        encoder.set_vertex_buffer(0, vertex_buffer.slice(..));
        encoder.draw(0..vertex_count, 0..1);
        encoder.finish(&wgpu::RenderBundleDescriptor {
            label: Some("main"),
        })
    }

    fn create_multisampled_framebuffer(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        format: ColorTargetState,
        sample_count: u32,
    ) -> wgpu::TextureView {
        let multisampled_texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            size: multisampled_texture_extent,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: format.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        };

        device
            .create_texture(multisampled_frame_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}

impl RenderPass for MsaaLinesRenderer {
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        wgpu_manager: &WgpuManager,
        view: &TextureView,
        format: ColorTargetState,
    ) {
        let device = wgpu_manager.device();

        let config = wgpu_manager
            .surface_configuration(&self.surface_entity)
            .unwrap();

        if self.prev_width != config.width || self.prev_height != config.height {
            self.bundle = Some(Self::create_bundle(
                device,
                config,
                &self.shader,
                &self.pipeline_layout,
                self.sample_count,
                &self.vertex_buffer,
                self.vertex_count,
                format.clone(),
            ));

            self.multisampled_framebuffer = Some(Self::create_multisampled_framebuffer(
                device,
                config,
                format,
                self.sample_count,
            ));

            self.prev_width = config.width;
            self.prev_height = config.height;
        }

        let ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: true,
        };

        let multisampled_framebuffer = self.multisampled_framebuffer.as_ref().unwrap();
        let bundle = self.bundle.as_ref().unwrap();

        let rpass_color_attachment = if self.sample_count == 1 {
            wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops,
            }
        } else {
            wgpu::RenderPassColorAttachment {
                view: &multisampled_framebuffer,
                resolve_target: Some(view),
                ops,
            }
        };

        encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[rpass_color_attachment],
                depth_stencil_attachment: None,
            })
            .execute_bundles(std::iter::once(bundle));
    }
}
