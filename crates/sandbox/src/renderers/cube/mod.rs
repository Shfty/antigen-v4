use antigen_wgpu::{RenderPass, WgpuManager};
use bytemuck::{Pod, Zeroable};
use cgmath::Zero;
use lazy::Lazy;
use std::{borrow::Cow, sync::Arc};
use wgpu::{BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, Device, RenderPipeline, ShaderModuleDescriptor, ShaderSource, Texture, TextureFormat, VertexBufferLayout, util::DeviceExt};

use antigen_resources::Timing;

use antigen_cgmath::components::{EyePosition, FieldOfView, LookAt};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

pub struct CubeRenderer {
    bind_group: BindGroup,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Arc<Buffer>,

    texture: Arc<Texture>,

    pipelines: Lazy<(RenderPipeline, Option<RenderPipeline>), (Arc<Device>, TextureFormat)>,
}

impl CubeRenderer {
    const BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor<'static> =
        BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        };

    const SHADER_MODULE_DESCRIPTOR: ShaderModuleDescriptor<'static> = ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    };

    const VERTEX_BUFFER_LAYOUTS: [VertexBufferLayout<'static>; 1] = [VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 4 * 4,
                shader_location: 1,
            },
        ],
    }];

    pub fn new(wgpu_manager: &WgpuManager) -> Self {
        // Fetch resources
        let device = wgpu_manager.device();
        let queue = wgpu_manager.queue();

        // Create vertex and index buffers
        let (vertex_data, index_data) = Self::create_vertices();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create pipeline layout
        let bind_group_layout =
            device.create_bind_group_layout(&Self::BIND_GROUP_LAYOUT_DESCRIPTOR);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create the texture
        let size = 256u32;
        let texels = Self::create_texels(size as usize);
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };

        let texture = Arc::new(device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        }));

        /*
        queue.write_texture(
            texture.as_image_copy(),
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(std::num::NonZeroU32::new(size).unwrap()),
                rows_per_image: None,
            },
            texture_extent,
        );
        */

        // Create uniform buffer
        let mx_total = cgmath::Matrix4::zero();
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let uniform_buffer = Arc::new(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(mx_ref),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        ));

        // Create texture view
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
            label: None,
        });

        let shader_module = device.create_shader_module(&Self::SHADER_MODULE_DESCRIPTOR);

        let pipelines = Lazy::new(Box::new(
            move |(device, format): (Arc<Device>, TextureFormat)| {
                let cube_pipeline =
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader_module,
                            entry_point: "vs_main",
                            buffers: &Self::VERTEX_BUFFER_LAYOUTS,
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader_module,
                            entry_point: "fs_main",
                            targets: &[format.clone().into()],
                        }),
                        primitive: wgpu::PrimitiveState {
                            cull_mode: Some(wgpu::Face::Back),
                            ..Default::default()
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState::default(),
                    });

                let wire_pipeline = if device
                    .features()
                    .contains(wgpu::Features::POLYGON_MODE_LINE)
                {
                    Some(
                        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: None,
                            layout: Some(&pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &shader_module,
                                entry_point: "vs_main",
                                buffers: &Self::VERTEX_BUFFER_LAYOUTS,
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &shader_module,
                                entry_point: "fs_wire",
                                targets: &[wgpu::ColorTargetState {
                                    format,
                                    blend: Some(wgpu::BlendState {
                                        color: wgpu::BlendComponent {
                                            operation: wgpu::BlendOperation::Add,
                                            src_factor: wgpu::BlendFactor::SrcAlpha,
                                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                        },
                                        alpha: wgpu::BlendComponent::REPLACE,
                                    }),
                                    write_mask: wgpu::ColorWrites::ALL,
                                }],
                            }),
                            primitive: wgpu::PrimitiveState {
                                front_face: wgpu::FrontFace::Ccw,
                                cull_mode: Some(wgpu::Face::Back),
                                polygon_mode: wgpu::PolygonMode::Line,
                                ..Default::default()
                            },
                            depth_stencil: None,
                            multisample: wgpu::MultisampleState::default(),
                        }),
                    )
                } else {
                    None
                };

                (cube_pipeline, wire_pipeline)
            },
        ));

        CubeRenderer {
            bind_group,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            texture,
            pipelines,
        }
    }

    pub fn take_uniform_buffer_handle(&self) -> Arc<Buffer> {
        self.uniform_buffer.clone()
    }

    pub fn take_texture_handle(&self) -> Arc<Texture> {
        self.texture.clone()
    }

    fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
        let vertex_data = [
            // top (0, 0, 1)
            vertex([-1, -1, 1], [0, 0]),
            vertex([1, -1, 1], [1, 0]),
            vertex([1, 1, 1], [1, 1]),
            vertex([-1, 1, 1], [0, 1]),
            // bottom (0, 0, -1)
            vertex([-1, 1, -1], [1, 0]),
            vertex([1, 1, -1], [0, 0]),
            vertex([1, -1, -1], [0, 1]),
            vertex([-1, -1, -1], [1, 1]),
            // right (1, 0, 0)
            vertex([1, -1, -1], [0, 0]),
            vertex([1, 1, -1], [1, 0]),
            vertex([1, 1, 1], [1, 1]),
            vertex([1, -1, 1], [0, 1]),
            // left (-1, 0, 0)
            vertex([-1, -1, 1], [1, 0]),
            vertex([-1, 1, 1], [0, 0]),
            vertex([-1, 1, -1], [0, 1]),
            vertex([-1, -1, -1], [1, 1]),
            // front (0, 1, 0)
            vertex([1, 1, -1], [1, 0]),
            vertex([-1, 1, -1], [0, 0]),
            vertex([-1, 1, 1], [0, 1]),
            vertex([1, 1, 1], [1, 1]),
            // back (0, -1, 0)
            vertex([1, -1, 1], [0, 0]),
            vertex([-1, -1, 1], [1, 0]),
            vertex([-1, -1, -1], [1, 1]),
            vertex([1, -1, -1], [0, 1]),
        ];

        let index_data: &[u16] = &[
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        (vertex_data.to_vec(), index_data.to_vec())
    }

    fn create_texels(size: usize) -> Vec<u8> {
        (0..size * size)
            .map(|id| {
                // get high five for recognizing this ;)
                let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
                let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
                let (mut x, mut y, mut count) = (cx, cy, 0);
                while count < 0xFF && x * x + y * y < 4.0 {
                    let old_x = x;
                    x = x * x - y * y + cx;
                    y = 2.0 * old_x * y + cy;
                    count += 1;
                }
                count
            })
            .collect()
    }
}

impl RenderPass for CubeRenderer {
    fn render(
        &mut self,
        wgpu_manager: &WgpuManager,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        config: &wgpu::SurfaceConfiguration,
    ) {
        let device = wgpu_manager.device();

        // Render
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        let (cube_pipeline, wire_pipeline) = self.pipelines.get((device, config.format));

        rpass.push_debug_group("Prepare data for draw.");
        rpass.set_pipeline(cube_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.pop_debug_group();
        rpass.insert_debug_marker("Draw!");
        rpass.draw_indexed(0..36, 0, 0..1);

        if let Some(wire_pipeline) = wire_pipeline {
            rpass.set_pipeline(wire_pipeline);
            rpass.draw_indexed(0..36, 0, 0..1);
        }
    }
}

// Sandbox specific code
#[legion::system(par_for_each)]
pub fn update_look(
    EyePosition(eye_position): &mut EyePosition,
    _: &LookAt,
    #[resource] timing: &Timing,
) {
    let total = timing.total_time().as_secs_f32();
    *eye_position = cgmath::Point3::new(total.sin() * 1.5, total.cos() * -5.0, 5.0);
}

#[legion::system(par_for_each)]
pub fn update_projection(perspective_projection: &mut FieldOfView, #[resource] timing: &Timing) {
    let total = timing.total_time().as_secs_f32();
    let fov = ((total * 0.2).sin() * 90.0) + 90.0;
    perspective_projection.set_fov(cgmath::Deg(fov));
}
