use antigen_wgpu::{components::BufferWrite, CastSlice, RenderPass, WgpuManager};
use bytemuck::{Pod, Zeroable};
use cgmath::{One, Zero};
use lazy::Lazy;
use legion::{world::SubWorld, Entity, IntoQuery};
use on_change::{OnChange, OnChangeTrait};
use rayon::iter::ParallelIterator;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    sync::{atomic::Ordering, Arc},
};
use wgpu::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, BufferAddress,
    BufferDescriptor, ComputePipeline, Device, RenderPipeline, ShaderModuleDescriptor,
    ShaderSource, Texture, TextureFormat, TextureView, VertexBufferLayout,
};

use antigen_resources::Timing;

use antigen_cgmath::components::{EyePosition, LookAt, Position3d, ProjectionMatrix};
use antigen_wgpu::DrawIndexedIndirect;

use crate::assemblages::{MeshId, MeshNormals, MeshTriangleIndices, MeshUvs, MeshVertices};

pub type BufferWriteVertices =
    BufferWrite<crate::renderers::cube::Vertices, Vec<crate::renderers::cube::Vertex>>;
legion_debugger::register_component!(BufferWriteVertices);

pub type BufferWriteIndices = BufferWrite<crate::renderers::cube::Indices, Vec<u16>>;
legion_debugger::register_component!(BufferWriteIndices);

pub type BufferWriteInstances =
    BufferWrite<crate::renderers::cube::InstanceComponent, crate::renderers::cube::Instance>;
legion_debugger::register_component!(BufferWriteInstances);

pub type BufferWriteIndexedIndirect = BufferWrite<
    crate::renderers::cube::IndexedIndirectComponent,
    antigen_wgpu::DrawIndexedIndirect,
>;
legion_debugger::register_component!(BufferWriteIndexedIndirect);

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct Vertex {
    pub pos: [f32; 3],
    _pad_0: f32,
    pub normal: [f32; 3],
    _pad_1: f32,
    pub tex_coord: [f32; 2],
    pub texture: i32,
}

pub fn vertex(pos: [f32; 3], normal: [f32; 3], tc: [f32; 2], ti: i32) -> Vertex {
    Vertex {
        pos,
        _pad_0: 1.0,
        normal,
        _pad_1: 0.0,
        tex_coord: tc,
        texture: ti,
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
#[derive(
    Default, Clone, Copy, PartialEq, PartialOrd, Pod, Zeroable, serde::Serialize, serde::Deserialize,
)]
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
#[derive(
    Clone, Copy, PartialEq, PartialOrd, Pod, Zeroable, serde::Serialize, serde::Deserialize,
)]
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

    depth_texture: Option<TextureView>,

    pipelines: Lazy<
        (ComputePipeline, RenderPipeline, Option<RenderPipeline>),
        (Arc<Device>, TextureFormat),
    >,

    indirect_count: BufferAddress,
    prev_width: u32,
    prev_height: u32,
}

impl CubeRenderer {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

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
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 4,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 8,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Sint32,
                    offset: 4 * 10,
                    shader_location: 3,
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
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 4,
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
        texture_count: u32,
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
        // Create uniform buffer
        let uniform_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // Create the texture
        let texture = Arc::new(device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: texture_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        }));

        // Create texture view
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        // Create pipeline layout
        let compute_bind_group_layout =
            device.create_bind_group_layout(&Self::COMPUTE_BIND_GROUP_LAYOUT_DESC);

        let render_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<Uniforms>() as u64,
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
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                ],
            });

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

        let compute_shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Owned(format!(
                "{}\n{}\n{}\n{}",
                include_str!("quaternion.wgsl"),
                include_str!("plane.wgsl"),
                include_str!("frustum.wgsl"),
                include_str!("compute.wgsl")
            ))),
        });

        let render_shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Owned(format!(
                "{}\n{}",
                include_str!("quaternion.wgsl"),
                include_str!("render.wgsl")
            ))),
        });

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
                        depth_stencil: Some(wgpu::DepthStencilState {
                            format: Self::DEPTH_FORMAT,
                            depth_write_enabled: true,
                            depth_compare: wgpu::CompareFunction::Less,
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        }),
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
                                cull_mode: None,
                                polygon_mode: wgpu::PolygonMode::Line,
                                ..Default::default()
                            },
                            depth_stencil: Some(wgpu::DepthStencilState {
                                format: Self::DEPTH_FORMAT,
                                depth_write_enabled: false,
                                depth_compare: wgpu::CompareFunction::Always,
                                stencil: wgpu::StencilState::default(),
                                bias: wgpu::DepthBiasState::default(),
                            }),
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
            depth_texture: None,
            pipelines,
            indirect_count,
            prev_width: 0,
            prev_height: 0,
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
            vertex([-0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 0.0], 0),
            vertex([0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 0.0], 0),
            vertex([0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 1.0], 0),
            vertex([-0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 1.0], 0),
            // bottom (0, 0, -0.5)
            vertex([-0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 0.0], 0),
            vertex([0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 0.0], 0),
            vertex([0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 1.0], 0),
            vertex([-0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 1.0], 0),
            // right (0.5, 0, 0)
            vertex([0.5, -0.5, -0.5], [1.0, 0.0, 0.0], [0.0, 0.0], 0),
            vertex([0.5, 0.5, -0.5], [1.0, 0.0, 0.0], [1.0, 0.0], 0),
            vertex([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], [1.0, 1.0], 0),
            vertex([0.5, -0.5, 0.5], [1.0, 0.0, 0.0], [0.0, 1.0], 0),
            // left (-0.5, 0, 0)
            vertex([-0.5, -0.5, 0.5], [-1.0, 0.0, 0.0], [1.0, 0.0], 0),
            vertex([-0.5, 0.5, 0.5], [-1.0, 0.0, 0.0], [0.0, 0.0], 0),
            vertex([-0.5, 0.5, -0.5], [-1.0, 0.0, 0.0], [0.0, 1.0], 0),
            vertex([-0.5, -0.5, -0.5], [-1.0, 0.0, 0.0], [1.0, 1.0], 0),
            // front (0, 0.5, 0)
            vertex([0.5, 0.5, -0.5], [0.0, 1.0, 0.0], [1.0, 0.0], 0),
            vertex([-0.5, 0.5, -0.5], [0.0, 1.0, 0.0], [0.0, 0.0], 0),
            vertex([-0.5, 0.5, 0.5], [0.0, 1.0, 0.0], [0.0, 1.0], 0),
            vertex([0.5, 0.5, 0.5], [0.0, 1.0, 0.0], [1.0, 1.0], 0),
            // back (0, -0.5, 0)
            vertex([0.5, -0.5, 0.5], [0.0, -1.0, 0.0], [0.0, 0.0], 0),
            vertex([-0.5, -0.5, 0.5], [0.0, -1.0, 0.0], [1.0, 0.0], 0),
            vertex([-0.5, -0.5, -0.5], [0.0, -1.0, 0.0], [1.0, 1.0], 0),
            vertex([0.5, -0.5, -0.5], [0.0, -1.0, 0.0], [0.0, 1.0], 0),
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
            vertex([0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [0.0, 0.0], 1),
            vertex([-c, d, -a], [0.0, 1.0, 0.0], [1.0, 0.0], 1),
            vertex([-c, -d, -a], [0.0, 1.0, 0.0], [1.0, 1.0], 1),
            vertex([b, 0.0, a], [0.0, 1.0, 0.0], [0.0, 1.0], 1),
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

    fn create_depth_texture(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
    ) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        });

        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
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

        // Recreate depth texture
        if self.depth_texture.is_none()
            || config.width != self.prev_width
            || config.height != self.prev_height
        {
            self.depth_texture = Some(Self::create_depth_texture(config, &wgpu_manager.device()));
            self.prev_width = config.width;
            self.prev_height = config.height;
        }

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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: self.depth_texture.as_ref().unwrap(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        rpass.push_debug_group("Prepare data for draw.");
        rpass.set_pipeline(cube_pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_bind_group(0, &self.render_bind_group, &[]);
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
    let pos = cgmath::Point3::new(total.sin() * 5.0, 2.5, total.cos() * 1.5);
    *eye_position = pos;
}

#[legion::system(par_for_each)]
pub fn update_instances(
    position: Option<&Position3d>,
    orientation: Option<&antigen_cgmath::components::Orientation>,
    visible: Option<&crate::components::Visible>,
    sphere_bounds: Option<&crate::components::SphereBounds>,
    instance: &mut InstanceComponent,
) {
    let mut inst = *instance.get();

    if let Some(position) = position {
        let pos: [f32; 3] = *(*position).as_ref();
        inst._position = [pos[0], pos[1], pos[2], 0.0];
    }

    if let Some(orientation) = orientation {
        inst._orientation = *(*orientation).as_ref();
    }

    if let Some(visible) = visible {
        inst._visible = **visible as u32;
    }

    if let Some(sphere_bounds) = sphere_bounds {
        inst._radius = **sphere_bounds;
    }

    instance.set_checked(inst)
}

#[legion::system(par_for_each)]
pub fn update_uniforms(
    projection_matrix: Option<&ProjectionMatrix>,
    eye_position: Option<&EyePosition>,
    orientation: Option<&antigen_cgmath::components::Orientation>,
    uniforms: &mut UniformsComponent,
) {
    let mut u = *uniforms.get();

    if let Some(eye_position) = eye_position {
        let pos: [f32; 3] = *(*eye_position).as_ref();
        let pos = [pos[0], pos[1], pos[2], 0.0];
        u._position = pos;
    }

    if let Some(orientation) = orientation {
        let quat: [f32; 4] = *(orientation).as_ref();
        u._orientation = quat;
    }

    if let Some(projection_matrix) = projection_matrix {
        let mx: [f32; 16] = *(*projection_matrix).as_ref();
        u._projection = mx;
    }

    uniforms.set_checked(u)
}

/// Iterate over mesh entities, translate generic components into renderer vertices
#[legion::system]
#[read_component(MeshId)]
#[read_component(MeshVertices<nalgebra::Vector3::<f32>>)]
#[read_component(MeshNormals<nalgebra::Vector3::<f32>>)]
#[read_component(MeshUvs<nalgebra::Vector2::<f32>>)]
#[write_component(Vertices)]
#[write_component(BufferWriteVertices)]
pub fn collect_vertices(world: &mut SubWorld) {
    let mut query = <(
        &MeshId,
        &MeshVertices<nalgebra::Vector3<f32>>,
        &MeshNormals<nalgebra::Vector3<f32>>,
        &MeshUvs<nalgebra::Vector2<f32>>,
        &mut Vertices,
    )>::query();

    // Iterate through entities with mesh data and renderer vertices,
    // collect into a sorted map of mesh id -> buffer length
    let lengths = query
        .par_iter_mut(world)
        .map(
            |(mesh_id, mesh_vertices, mesh_normals, mesh_uvs, Vertices(cube_vertices))| {
                let mut verts = vec![];
                for (vertex, (normal, uv)) in mesh_vertices
                    .iter()
                    .zip(mesh_normals.iter().zip(mesh_uvs.iter()))
                {
                    verts.push(Vertex {
                        pos: [vertex.x, vertex.y, vertex.z],
                        _pad_0: 1.0,
                        normal: [normal.x, normal.y, normal.z],
                        _pad_1: 0.0,
                        tex_coord: [uv.x, uv.y],
                        texture: 0,
                    });
                }

                cube_vertices.set(verts);
                (*mesh_id, mesh_vertices.len())
            },
        )
        .collect::<BTreeMap<_, _>>();

    // Calculate offsets from buffer lengths
    let mut offset = 0;
    let offsets = lengths
        .into_iter()
        .map(|(mesh_id, len)| {
            let prev_offset = offset;
            offset += len * std::mem::size_of::<Vertex>();
            (mesh_id, prev_offset)
        })
        .collect::<BTreeMap<_, _>>();

    // Write offsets to buffer write components
    let mut query = <(&MeshId, &mut Vertices, &mut BufferWriteVertices)>::query();
    query.par_for_each_mut(world, |(mesh_id, _, buffer_write)| {
        buffer_write.set_offset(offsets[mesh_id] as u64);
    });
}

/// Iterate over mesh entities, translate generic components into renderer indices
#[legion::system]
#[read_component(MeshId)]
#[read_component(MeshTriangleIndices<usize>)]
#[write_component(Indices)]
#[write_component(BufferWriteIndices)]
pub fn collect_indices(world: &mut SubWorld) {
    let mut query = <(&MeshId, &MeshTriangleIndices<usize>, &mut Indices)>::query();

    // Iterate through entities with mesh data and renderer indices,
    // collect into a sorted map of mesh id -> buffer length
    let lengths = query
        .par_iter_mut(world)
        .map(|(mesh_id, mesh_indices, Indices(cube_indices))| {
            let mut inds = vec![];
            for index in mesh_indices.iter()
            {
                inds.push(*index as u16);
            }

            cube_indices.set(inds);
            (*mesh_id, mesh_indices.len())
        })
        .collect::<BTreeMap<_, _>>();

    // Calculate offsets from buffer lengths
    let mut offset = 0;
    let offsets = lengths
        .into_iter()
        .map(|(mesh_id, len)| {
            let prev_offset = offset;
            offset += len * std::mem::size_of::<Index>();
            (mesh_id, prev_offset)
        })
        .collect::<BTreeMap<_, _>>();

    // Write offsets to buffer write components
    let mut query = <(&MeshId, &mut Indices, &mut BufferWriteIndices)>::query();
    query.par_for_each_mut(world, |(mesh_id, _, buffer_write)| {
        buffer_write.set_offset(offsets[mesh_id] as u64);
    });
}

/// Iterate over mesh entities, translate generic components into instance and indirect data
#[legion::system]
#[read_component(InstanceComponent)]
#[write_component(IndexedIndirectComponent)]
#[write_component(BufferWriteInstances)]
#[write_component(BufferWriteIndexedIndirect)]
pub fn collect_instances_indirects(world: &mut SubWorld) {
    let mut query = <(
        &InstanceComponent,
        &mut IndexedIndirectComponent,
        &mut BufferWriteInstances,
        &mut BufferWriteIndexedIndirect,
    )>::query();

    let mut offset = 0 as wgpu::BufferAddress;
    query.for_each_mut(
        world,
        |(
            _,
            IndexedIndirectComponent(indexed_indirect),
            buffer_write_instances,
            buffer_write_indirects,
        )| {
            let indirect = *indexed_indirect.get();
            indexed_indirect.set(DrawIndexedIndirect {
                base_instance: offset as u32,
                ..indirect
            });
            buffer_write_instances
                .set_offset(std::mem::size_of::<Instance>() as wgpu::BufferAddress * offset);
            buffer_write_indirects.set_offset(
                std::mem::size_of::<DrawIndexedIndirect>() as wgpu::BufferAddress * offset,
            );
            offset += 1;
        },
    );
}
