use antigen_wgpu::{CastSlice, RenderPass, WgpuManager};
use bytemuck::{Pod, Zeroable};
use cgmath::{One, Zero};
use lazy::Lazy;
use on_change::{OnChange, OnChangeTrait};
use std::{borrow::Cow, sync::Arc};
use wgpu::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, BufferAddress,
    BufferDescriptor, ComputePipeline, Device, RenderPipeline, ShaderModuleDescriptor,
    ShaderSource, Texture, TextureFormat, VertexBufferLayout,
};

use antigen_resources::Timing;

use antigen_cgmath::components::{EyePosition, FieldOfView, LookAt, ProjectionMatrix, Position3d};
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
pub struct Uniforms {
    _position: [f32; 4],
    _orientation: [f32; 4],
    _projection: [f32; 16],
}

impl CastSlice<u8> for Uniforms {
    fn cast_slice(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UniformsComponent(pub OnChange<Uniforms>);

impl UniformsComponent {
    pub fn new(proj_mx: cgmath::Matrix4<f32>) -> Self {
        let pos = cgmath::Vector4::<f32>::zero();
        let pos: [f32; 4] = *pos.as_ref();

        let quat = cgmath::Quaternion::one();
        let quat: [f32; 4] = *quat.as_ref();

        let mx: [f32; 16] = *proj_mx.as_ref();

        UniformsComponent(OnChange::new_dirty(Uniforms {
            _position: pos,
            _orientation: quat,
            _projection: mx,
        }))
    }
}

impl std::ops::Deref for UniformsComponent {
    type Target = OnChange<Uniforms>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for UniformsComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl OnChangeTrait<Uniforms> for UniformsComponent {
    fn take_change(&self) -> Option<&Uniforms> {
        self.0.take_change()
    }
}

legion_debugger::register_component!(UniformsComponent);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct Instance {
    _position: [f32; 4],
    _orientation: [f32; 4],
    _visible: u32,
    _radius: f32,
    _pad: [u32; 2],
}

impl Default for Instance {
    fn default() -> Self {
        Instance {
            _position: Default::default(),
            _orientation: [0.0, 0.0, 0.0, 1.0],
            _visible: 1,
            _radius: 0.0,
            _pad: Default::default(),
        }
    }
}

impl CastSlice<u8> for Instance {
    fn cast_slice(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InstanceComponent(pub OnChange<Instance>);

impl Default for InstanceComponent {
    fn default() -> Self {
        InstanceComponent(OnChange::new_dirty(Default::default()))
    }
}

impl InstanceComponent {
    pub fn new(
        position: cgmath::Vector3<f32>,
        orientation: cgmath::Quaternion<f32>,
        radius: f32,
        visible: bool,
    ) -> Self {
        let pos: [f32; 3] = *position.as_ref();
        let pos = [pos[0], pos[1], pos[2], 0.0];

        let quat: [f32; 4] = *orientation.as_ref();

        let visible: u32 = if visible { 1 } else { 0 };

        InstanceComponent(OnChange::new_dirty(Instance {
            _position: pos,
            _orientation: quat,
            _visible: visible,
            _radius: radius,
            _pad: Default::default(),
        }))
    }
}

impl std::ops::Deref for InstanceComponent {
    type Target = OnChange<Instance>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
    compute_bind_group: BindGroup,
    render_bind_group: BindGroup,

    vertex_buffer: Arc<Buffer>,
    index_buffer: Arc<Buffer>,

    uniform_buffer: Arc<Buffer>,
    indirect_buffer: Arc<Buffer>,

    instance_buffer: Arc<Buffer>,

    texture: Arc<Texture>,

    pipelines: Lazy<
        (ComputePipeline, RenderPipeline, Option<RenderPipeline>),
        (Arc<Device>, TextureFormat),
    >,

    indirect_count: BufferAddress,
}

impl CubeRenderer {
    const COMPUTE_BIND_GROUP_LAYOUT_DESC: BindGroupLayoutDescriptor<'static> =
        BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Uniforms>() as u64
                        ),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Instance>() as u64
                        ),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                            DrawIndexedIndirect,
                        >() as u64),
                    },
                    count: None,
                },
            ],
        };

    const RENDER_BIND_GROUP_LAYOUT_DESC: BindGroupLayoutDescriptor<'static> =
        BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Uniforms>() as u64
                        ),
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
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let instance_buffer = Arc::new(device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: std::mem::size_of::<Instance>() as BufferAddress
                * instance_count as BufferAddress,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

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
            size: std::mem::size_of::<Uniforms>() as BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // Create texture view
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create pipeline layout
        let compute_bind_group_layout =
            device.create_bind_group_layout(&Self::COMPUTE_BIND_GROUP_LAYOUT_DESC);

        let render_bind_group_layout =
            device.create_bind_group_layout(&Self::RENDER_BIND_GROUP_LAYOUT_DESC);

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Create bind group
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: indirect_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_bind_group_layout,
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

        let compute_shader_desc = ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Owned(format!(
                "{}\n{}\n{}\n{}",
                include_str!("quaternion.wgsl"),
                include_str!("plane.wgsl"),
                include_str!("frustum.wgsl"),
                include_str!("compute.wgsl")
            ))),
        };

        let render_shader_desc = ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Owned(format!(
                "{}\n{}",
                include_str!("quaternion.wgsl"),
                include_str!("render.wgsl")
            ))),
        };

        let compute_shader = device.create_shader_module(&compute_shader_desc);
        let render_shader = device.create_shader_module(&render_shader_desc);

        let pipelines = Lazy::new(Box::new(
            move |(device, format): (Arc<Device>, TextureFormat)| {
                let compute_pipeline =
                    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: None,
                        layout: Some(&compute_pipeline_layout),
                        module: &compute_shader,
                        entry_point: "main",
                    });

                let cube_pipeline =
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &render_shader,
                            entry_point: "vs_main",
                            buffers: &Self::VERTEX_BUFFER_LAYOUTS,
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &render_shader,
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
                            layout: Some(&render_pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &render_shader,
                                entry_point: "vs_main",
                                buffers: &Self::VERTEX_BUFFER_LAYOUTS,
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &render_shader,
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

                (compute_pipeline, cube_pipeline, wire_pipeline)
            },
        ));

        CubeRenderer {
            compute_bind_group,
            render_bind_group,
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
        let (compute_pipeline, cube_pipeline, wire_pipeline) =
            self.pipelines.get((device, config.format));

        // Compute
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(compute_pipeline);
        cpass.set_bind_group(0, &self.compute_bind_group, &[]);
        cpass.dispatch(1, 1, 1);
        drop(cpass);

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

        rpass.push_debug_group("Prepare data for draw.");
        rpass.set_pipeline(cube_pipeline);
        rpass.set_bind_group(0, &self.render_bind_group, &[]);
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
    let pos = cgmath::Point3::new(total.sin() * 5.0, 5.0, total.cos() * 1.5);
    *eye_position = pos;
}

#[legion::system(par_for_each)]
pub fn update_projection(field_of_view: &mut FieldOfView, #[resource] timing: &Timing) {
    let total = timing.total_time().as_secs_f32();
    let fov = 90.0;//((total * 0.2).sin() * 90.0) + 90.0;
    field_of_view.set_fov(cgmath::Deg(fov));
}

#[legion::system(par_for_each)]
pub fn update_instances(
    position: &Position3d,
    orientation: &antigen_cgmath::components::Orientation,
    visible: &crate::components::Visible,
    sphere_bounds: &crate::components::SphereBounds,
    instance: &mut InstanceComponent,
) {
    let inst = *instance.get();

    let pos: [f32; 3] = *(*position).as_ref();
    let pos = [pos[0], pos[1], pos[2], 0.0];

    let quat: [f32; 4] = *(*orientation).as_ref();

    let visible = **visible;

    let radius = **sphere_bounds;

    instance.set(Instance {
        _position: pos,
        _orientation: quat,
        _visible: visible as u32,
        _radius: radius,
        ..inst
    })
}

#[legion::system(par_for_each)]
pub fn update_uniforms(
    projection_matrix: &ProjectionMatrix,
    eye_position: &EyePosition,
    orientation: &antigen_cgmath::components::Orientation,
    uniforms: &mut UniformsComponent,
) {
    let mx: [f32; 16] = *(*projection_matrix).as_ref();

    let pos: [f32; 3] = *(*eye_position).as_ref();
    let pos = [pos[0], pos[1], pos[2], 0.0];

    let quat: [f32; 4] = *(orientation).as_ref();

    uniforms.set(Uniforms {
        _position: pos,
        _orientation: quat,
        _projection: mx,
    })
}
