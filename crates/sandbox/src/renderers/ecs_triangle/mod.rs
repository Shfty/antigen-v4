use std::collections::BTreeMap;

use antigen_wgpu::components::{PipelineLayoutComponent, ShaderComponent};
use legion::{world::SubWorld, Entity, IntoQuery, World};
use parking_lot::Mutex;
use wgpu::{Adapter, CommandBuffer, Device, RenderPipeline, Surface, TextureView};
use winit::window::WindowId;

pub fn assemble(world: &mut World, window_id: WindowId) {
    world.push((
        window_id,
        ShaderComponent {
            pending_desc: Some(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            }),
            shader: None,
        },
        PipelineLayoutComponent {
            pending_desc: Some(wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            }),
            pipeline_layout: None,
        },
        ECSTriangleRenderPipeline {
            shader_entity: None,
            pipeline: None,
        },
    ));
}

struct ECSTriangleRenderPipeline {
    pub shader_entity: Option<Entity>,
    pub pipeline: Option<RenderPipeline>,
}

#[legion::system(par_for_each)]
#[read_component(ShaderComponent)]
fn ecs_triangle_prepare(
    window_id: &WindowId,
    world: &SubWorld,
    entity: &Entity,
    pipeline_layout: &PipelineLayoutComponent,
    render_pipeline: &mut ECSTriangleRenderPipeline,
    #[resource] adapter: &Adapter,
    #[resource] device: &Device,
    #[resource] surface: &Surface,
    #[resource] target_window_id: &WindowId,
) {
    if *window_id != *target_window_id {
        return;
    }

    if render_pipeline.pipeline.is_some() {
        return;
    }

    let pipeline_layout = if let Some(pipeline_layout) = &pipeline_layout.pipeline_layout {
        pipeline_layout
    } else {
        return;
    };

    let shader = if let Ok(shader) =
        <&ShaderComponent>::query().get(world, render_pipeline.shader_entity.unwrap_or(*entity))
    {
        if let Some(shader) = &shader.shader {
            shader
        } else {
            return;
        }
    } else {
        return;
    };

    render_pipeline.pipeline = Some(
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[surface
                    .get_preferred_format(adapter)
                    .expect("Surface incompatible with adapter")
                    .into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        }),
    );
}

struct CommandBuffers(pub Mutex<BTreeMap<usize, CommandBuffer>>);

impl std::ops::Deref for CommandBuffers {
    type Target = Mutex<BTreeMap<usize, CommandBuffer>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CommandBuffers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[legion::system(par_for_each)]
fn ecs_triangle_render(
    window_id: &WindowId,
    render_pipeline: &ECSTriangleRenderPipeline,
    #[resource] target_window_id: &WindowId,
    #[resource] device: &Device,
    #[resource] view: &TextureView,
    #[resource] command_buffers: &CommandBuffers,
) {
    if *window_id != *target_window_id {
        return
    }

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let pipeline = if let Some(pipeline) = &render_pipeline.pipeline {
        pipeline
    } else {
        return;
    };

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

    rpass.set_pipeline(pipeline);
    rpass.draw(0..3, 0..1);
    drop(rpass);

    command_buffers.lock().insert(0, encoder.finish());
}
