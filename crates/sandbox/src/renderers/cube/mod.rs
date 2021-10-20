use antigen_wgpu::{CastSlice, RenderPass, WgpuManager};
use bytemuck::{Pod, Zeroable};
use lazy::Lazy;
use on_change::{OnChange, OnChangeTrait};
use std::{borrow::Cow, sync::Arc};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, BufferAddress,
    BufferDescriptor, Device, RenderPipeline, ShaderModuleDescriptor, ShaderSource, Texture,
    TextureFormat, VertexBufferLayout,
};

use antigen_resources::Timing;

use antigen_cgmath::components::{EyePosition, FieldOfView, LookAt};
use antigen_wgpu::DrawIndexedIndirect;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

fn vertex(pos: [f32; 3], tc: [f32; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0], pos[1], pos[2], 1.0],
        _tex_coord: [tc[0], tc[1]],
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Vertices(pub OnChange<Vec<Vertex>>);

impl Vertices {
    pub fn new(vertices: Vec<Vertex>) -> Self {
        Vertices(OnChange::new_dirty(vertices))
    }
}

impl OnChangeTrait<Vec<Vertex>> for Vertices {
    fn take_change(&self) -> Option<&Vec<Vertex>> {
        self.0.take_change()
    }
}

legion_debugger::register_component!(Vertices);

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct Instance {
    _trx: [f32; 16],
}

impl CastSlice<u8> for Instance {
    fn cast_slice(&self) -> &[u8] {
        bytemuck::cast_slice(&self._trx)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InstanceComponent(pub OnChange<Instance>);

impl InstanceComponent {
    pub fn new(mx: cgmath::Matrix4<f32>) -> Self {
        let mx: [f32; 16] = *mx.as_ref();
        InstanceComponent(OnChange::new_dirty(Instance { _trx: mx }))
    }
}

impl OnChangeTrait<Instance> for InstanceComponent {
    fn take_change(&self) -> Option<&Instance> {
        self.0.take_change()
    }
}

legion_debugger::register_component!(InstanceComponent);

pub type Index = u16;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Indices(pub OnChange<Vec<Index>>);

impl Indices {
    pub fn new(indices: Vec<Index>) -> Self {
        Indices(OnChange::new_dirty(indices))
    }
}

impl OnChangeTrait<Vec<Index>> for Indices {
    fn take_change(&self) -> Option<&Vec<Index>> {
        self.0.take_change()
    }
}

legion_debugger::register_component!(Indices);

#[derive(serde::Serialize, serde::Deserialize)]
pub struct IndexedIndirectComponent(pub OnChange<DrawIndexedIndirect>);

impl IndexedIndirectComponent {
    pub fn new(indirect: DrawIndexedIndirect) -> Self {
        IndexedIndirectComponent(OnChange::new_dirty(indirect))
    }
}

impl OnChangeTrait<DrawIndexedIndirect> for IndexedIndirectComponent {
    fn take_change(&self) -> Option<&DrawIndexedIndirect> {
        self.0.take_change()
    }
}

legion_debugger::register_component!(IndexedIndirectComponent);

pub struct CubeRenderer {
    bind_group: BindGroup,

    vertex_buffer: Arc<Buffer>,
    index_buffer: Arc<Buffer>,

    uniform_buffer: Arc<Buffer>,
    indirect_buffer: Arc<Buffer>,

    instance_buffer: Arc<Buffer>,

    texture: Arc<Texture>,

    pipelines: Lazy<(RenderPipeline, Option<RenderPipeline>), (Arc<Device>, TextureFormat)>,

    indirect_count: BufferAddress,
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

    const VERTEX_BUFFER_LAYOUTS: [VertexBufferLayout<'static>; 2] = [
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
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
        },
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 4,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 4 * 2,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 4 * 3,
                    shader_location: 5,
                },
            ],
        },
    ];

    pub fn new(
        wgpu_manager: &WgpuManager,
        vertex_count: BufferAddress,
        index_count: BufferAddress,
        indirect_count: BufferAddress,
        instance_count: BufferAddress,
    ) -> Self {
        // Fetch resources
        let device = wgpu_manager.device();

        // Create vertex, index, indirect and instance buffers
        let vertex_buffer = Arc::new(device.create_buffer(&BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: std::mem::size_of::<Vertex>() as BufferAddress * vertex_count,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let index_buffer = Arc::new(device.create_buffer(&BufferDescriptor {
            label: Some("Index Buffer"),
            size: std::mem::size_of::<Index>() as BufferAddress * index_count,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let indirect_buffer = Arc::new(device.create_buffer(&BufferDescriptor {
            label: Some("Indirect Buffer"),
            size: std::mem::size_of::<DrawIndexedIndirect>() as BufferAddress
                * indirect_count as BufferAddress,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let instance_buffer = Arc::new(device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: std::mem::size_of::<Instance>() as BufferAddress
                * instance_count as BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

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

        // Create uniform buffer
        let uniform_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<cgmath::Matrix4<f32>>() as BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

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
            instance_buffer,
            indirect_buffer,
            texture,
            pipelines,
            indirect_count,
        }
    }

    pub fn take_vertex_buffer_handle(&self) -> Arc<Buffer> {
        self.vertex_buffer.clone()
    }

    pub fn take_index_buffer_handle(&self) -> Arc<Buffer> {
        self.index_buffer.clone()
    }

    pub fn take_uniform_buffer_handle(&self) -> Arc<Buffer> {
        self.uniform_buffer.clone()
    }

    pub fn take_instance_buffer_handle(&self) -> Arc<Buffer> {
        self.instance_buffer.clone()
    }

    pub fn take_indirect_buffer_handle(&self) -> Arc<Buffer> {
        self.indirect_buffer.clone()
    }

    pub fn take_texture_handle(&self) -> Arc<Texture> {
        self.texture.clone()
    }

    pub fn cube_vertices() -> (Vec<Vertex>, Vec<Index>) {
        #[rustfmt::skip]
        let vertex_data = [
            // top (0, 0, 1)
            vertex([-0.5, -0.5, 0.5], [0.0, 0.0]),
            vertex([0.5, -0.5, 0.5], [1.0, 0.0]),
            vertex([0.5, 0.5, 0.5], [1.0, 1.0]),
            vertex([-0.5, 0.5, 0.5], [0.0, 1.0]),
            // bottom (0, 0, -0.5)
            vertex([-0.5, 0.5, -0.5], [1.0, 0.0]),
            vertex([0.5, 0.5, -0.5], [0.0, 0.0]),
            vertex([0.5, -0.5, -0.5], [0.0, 1.0]),
            vertex([-0.5, -0.5, -0.5], [1.0, 1.0]),
            // right (0.5, 0, 0)
            vertex([0.5, -0.5, -0.5], [0.0, 0.0]),
            vertex([0.5, 0.5, -0.5], [1.0, 0.0]),
            vertex([0.5, 0.5, 0.5], [1.0, 1.0]),
            vertex([0.5, -0.5, 0.5], [0.0, 1.0]),
            // left (-0.5, 0, 0)
            vertex([-0.5, -0.5, 0.5], [1.0, 0.0]),
            vertex([-0.5, 0.5, 0.5], [0.0, 0.0]),
            vertex([-0.5, 0.5, -0.5], [0.0, 1.0]),
            vertex([-0.5, -0.5, -0.5], [1.0, 1.0]),
            // front (0, 0.5, 0)
            vertex([0.5, 0.5, -0.5], [1.0, 0.0]),
            vertex([-0.5, 0.5, -0.5], [0.0, 0.0]),
            vertex([-0.5, 0.5, 0.5], [0.0, 1.0]),
            vertex([0.5, 0.5, 0.5], [1.0, 1.0]),
            // back (0, -0.5, 0)
            vertex([0.5, -0.5, 0.5], [0.0, 0.0]),
            vertex([-0.5, -0.5, 0.5], [1.0, 0.0]),
            vertex([-0.5, -0.5, -0.5], [1.0, 1.0]),
            vertex([0.5, -0.5, -0.5], [0.0, 1.0]),
        ];

        #[rustfmt::skip]
        let index_data = [
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        (vertex_data.to_vec(), index_data.to_vec())
    }

    pub fn tetrahedron_vertices() -> (Vec<Vertex>, Vec<Index>) {
        let a = 1.0f32 / 3.0;
        let b = (8.0f32 / 9.0).sqrt();
        let c = (2.0f32 / 9.0).sqrt();
        let d = (2.0f32 / 3.0).sqrt();

        #[rustfmt::skip]
        let vertex_data = [
            vertex([0.0, 0.0, 1.0], [0.0, 0.0]),
            vertex([-c, d, -a], [1.0, 0.0]),
            vertex([-c, -d, -a], [1.0, 1.0]),
            vertex([b, 0.0, a], [0.0, 1.0]),
        ];

        #[rustfmt::skip]
        let index_data = [
            0, 1, 2,
            0, 2, 3,
            0, 3, 1,
            3, 2, 1,
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

    fn create_instances() -> Vec<Instance> {
        let mut instances = vec![];

        let count = 1;
        for x in -count..=count {
            for y in -count..=count {
                for z in -count..=count {
                    let mx = cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3::new(
                        x as f32, y as f32, z as f32,
                    ));
                    let mx: [f32; 16] = *mx.as_ref();
                    instances.push(Instance { _trx: mx })
                }
            }
        }

        instances
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
        rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rpass.pop_debug_group();

        rpass.insert_debug_marker("Draw!");
        rpass.draw_indexed_indirect(&self.indirect_buffer, 0);
        for i in 0..self.indirect_count {
            rpass.draw_indexed_indirect(
                &self.indirect_buffer,
                std::mem::size_of::<DrawIndexedIndirect>() as BufferAddress * i,
            );
        }

        if let Some(wire_pipeline) = wire_pipeline {
            rpass.set_pipeline(wire_pipeline);
            for i in 0..self.indirect_count {
                rpass.draw_indexed_indirect(
                    &self.indirect_buffer,
                    std::mem::size_of::<DrawIndexedIndirect>() as BufferAddress * i,
                );
            }
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
