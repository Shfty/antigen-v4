use std::{borrow::Cow, rc::Rc};

use antigen_wgpu::{RenderPass, WgpuManager};
use lazy::Lazy;
use rand::{prelude::Distribution, SeedableRng};
use wgpu::{
    util::DeviceExt, BindGroup, Buffer, ComputePipeline, Device, RenderPipeline,
    ShaderModuleDescriptor, ShaderSource, TextureFormat,
};

// number of boid particles to simulate
const NUM_PARTICLES: u32 = 1500;

// number of single-particle calculations (invocations) in each gpu work group
const PARTICLES_PER_GROUP: u32 = 64;

pub struct BoidsRenderer {
    particle_bind_groups: Vec<BindGroup>,
    particle_buffers: Vec<Buffer>,
    vertices_buffer: Buffer,
    compute_pipeline: ComputePipeline,
    render_pipeline: Lazy<RenderPipeline, (Rc<Device>, TextureFormat)>,
    work_group_count: u32,
    frame_num: usize,
}

impl BoidsRenderer {
    pub fn new(wgpu_manager: &WgpuManager) -> Self {
        let device = wgpu_manager.device();

        let compute_shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("compute.wgsl"))),
        });
        let draw_shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("draw.wgsl"))),
        });

        // buffer for simulation parameters uniform
        let sim_param_data = [
            0.04f32, // deltaT
            0.1,     // rule1Distance
            0.025,   // rule2Distance
            0.025,   // rule3Distance
            0.02,    // rule1Scale
            0.05,    // rule2Scale
            0.005,   // rule3Scale
        ]
        .to_vec();

        let sim_param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Parameter Buffer"),
            contents: bytemuck::cast_slice(&sim_param_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // create compute bind layout group and compute pipeline layout
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (sim_param_data.len() * std::mem::size_of::<f32>()) as _,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new((NUM_PARTICLES * 16) as _),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new((NUM_PARTICLES * 16) as _),
                        },
                        count: None,
                    },
                ],
                label: None,
            });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        // create render pipeline with empty bind group layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = Lazy::new(Box::new(
            move |(device, format): (Rc<Device>, TextureFormat)| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &draw_shader,
                entry_point: "main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 4 * 4,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &draw_shader,
                entry_point: "main",
                targets: &[format.into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        })
            },
        ));

        // create compute pipeline
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        // buffer for the three 2d triangle vertices of each instance
        let vertex_buffer_data = [-0.01f32, -0.02, 0.01, -0.02, 0.00, 0.02];
        let vertices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::bytes_of(&vertex_buffer_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // buffer for all particles data of type [(posx,posy,velx,vely),...]
        let mut initial_particle_data = vec![0.0f32; (4 * NUM_PARTICLES) as usize];
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let unif = rand::distributions::Uniform::new_inclusive(-1.0, 1.0);
        for particle_instance_chunk in initial_particle_data.chunks_mut(4) {
            particle_instance_chunk[0] = unif.sample(&mut rng); // posx
            particle_instance_chunk[1] = unif.sample(&mut rng); // posy
            particle_instance_chunk[2] = unif.sample(&mut rng) * 0.1; // velx
            particle_instance_chunk[3] = unif.sample(&mut rng) * 0.1; // vely
        }

        // creates two buffers of particle data each of size NUM_PARTICLES
        // the two buffers alternate as dst and src for each frame
        let mut particle_buffers = Vec::<Buffer>::new();
        let mut particle_bind_groups = Vec::<BindGroup>::new();
        for i in 0..2 {
            particle_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Particle Buffer {}", i)),
                    contents: bytemuck::cast_slice(&initial_particle_data),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                }),
            );
        }

        // create two bind groups, one for each buffer as the src
        // where the alternate buffer is used as the dst
        for i in 0..2 {
            particle_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: sim_param_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: particle_buffers[(i + 1) % 2].as_entire_binding(), // bind to opposite buffer
                    },
                ],
                label: None,
            }));
        }

        // calculates number of work groups from PARTICLES_PER_GROUP constant
        let work_group_count =
            ((NUM_PARTICLES as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        // returns Example struct and No encoder commands
        BoidsRenderer {
            particle_bind_groups,
            particle_buffers,
            vertices_buffer,
            compute_pipeline,
            render_pipeline,
            work_group_count,
            frame_num: 0,
        }
    }
}

impl RenderPass for BoidsRenderer {
    fn render(
        &mut self,
        wgpu_manager: &WgpuManager,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        config: &wgpu::SurfaceConfiguration,
    ) {
        // create render pass descriptor and its color attachments
        let color_attachments = [wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: true,
            },
        }];
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
        };

        let render_pipeline = self
            .render_pipeline
            .get((wgpu_manager.device(), config.format));

        encoder.push_debug_group("compute boid movement");
        {
            // compute pass
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.particle_bind_groups[self.frame_num % 2], &[]);
            cpass.dispatch(self.work_group_count, 1, 1);
        }
        encoder.pop_debug_group();

        encoder.push_debug_group("render boids");
        {
            // render pass
            let mut rpass = encoder.begin_render_pass(&render_pass_descriptor);
            rpass.set_pipeline(render_pipeline);
            // render dst particles
            rpass.set_vertex_buffer(0, self.particle_buffers[(self.frame_num + 1) % 2].slice(..));
            // the three instance-local vertices
            rpass.set_vertex_buffer(1, self.vertices_buffer.slice(..));
            rpass.draw(0..3, 0..NUM_PARTICLES);
        }
        encoder.pop_debug_group();

        // update frame count
        self.frame_num += 1;
    }
}
