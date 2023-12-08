use antigen_rapier3d::rapier3d::parry::utils::hashmap::HashMap;
use antigen_wgpu::{
    components::{BufferComponent, BufferWrite},
    CastSlice, RenderPass, WgpuManager,
};
use bytemuck::{Pod, Zeroable};
use cgmath::{One, Zero};
use lazy::Lazy;
use legion::{world::SubWorld, Entity, IntoQuery};
use on_change::{OnChange, OnChangeTrait};
use rayon::iter::ParallelIterator;
use std::{borrow::Cow, collections::BTreeMap, sync::Arc};
use wgpu::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, BufferAddress,
    BufferDescriptor, ComputePipeline, Device, RenderPipeline, ShaderModuleDescriptor,
    ShaderSource, Texture, TextureFormat, TextureView, VertexBufferLayout,
};

use antigen_resources::Timing;

use antigen_cgmath::components::{EyePosition, LookAt, ProjectionMatrix};
use antigen_wgpu::DrawIndexedIndirect;

use crate::assemblages::{
    IndexBuffer, MeshEntity, MeshId, MeshNormals, MeshTextureIds, MeshTextureIdsI32,
    MeshTriangleIndices, MeshUvs, MeshVertices, VertexBuffer,
};

pub type BufferWriteMeshVertices =
    BufferWrite<MeshVertices<nalgebra::Vector3<f32>>, Vec<nalgebra::Vector3<f32>>>;
legion_debugger::register_component!(BufferWriteMeshVertices);

pub type BufferWriteMeshNormals =
    BufferWrite<MeshNormals<nalgebra::Vector3<f32>>, Vec<nalgebra::Vector3<f32>>>;
legion_debugger::register_component!(BufferWriteMeshNormals);

pub type BufferWriteMeshUvs =
    BufferWrite<MeshUvs<nalgebra::Vector2<f32>>, Vec<nalgebra::Vector2<f32>>>;
legion_debugger::register_component!(BufferWriteMeshUvs);

pub type BufferWriteMeshTextureIds = BufferWrite<MeshTextureIdsI32, Vec<i32>>;
legion_debugger::register_component!(BufferWriteMeshTextureIds);

pub type BufferWriteMeshTriangleIndices = BufferWrite<MeshTriangleIndices<Index>, Vec<Index>>;
legion_debugger::register_component!(BufferWriteMeshTriangleIndices);

pub type BufferWritePosition =
    BufferWrite<antigen_cgmath::components::Position3d, antigen_cgmath::cgmath::Vector3<f32>>;
legion_debugger::register_component!(BufferWritePosition);

pub type BufferWriteOrientation =
    BufferWrite<antigen_cgmath::components::Orientation, antigen_cgmath::cgmath::Quaternion<f32>>;
legion_debugger::register_component!(BufferWriteOrientation);

pub type BufferWriteInstances =
    BufferWrite<crate::renderers::cube::InstanceComponent, crate::renderers::cube::Instance>;
legion_debugger::register_component!(BufferWriteInstances);

pub type BufferWriteIndexedIndirect = BufferWrite<
    crate::renderers::cube::IndexedIndirectComponent,
    antigen_wgpu::DrawIndexedIndirect,
>;
legion_debugger::register_component!(BufferWriteIndexedIndirect);

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
    _visible: u32,
    _radius: f32,
    _pad: [u32; 2],
}

impl Default for Instance {
    fn default() -> Self {
        Instance {
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
    pub fn new(radius: f32, visible: bool) -> Self {
        let visible: u32 = if visible { 1 } else { 0 };

        InstanceComponent(OnChange::new_dirty(Instance {
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
pub struct IndexedIndirectComponent(pub OnChange<DrawIndexedIndirect>);

impl IndexedIndirectComponent {
    pub fn new(indirect: DrawIndexedIndirect) -> Self {
        IndexedIndirectComponent(OnChange::new_dirty(indirect))
    }
}

impl Default for IndexedIndirectComponent {
    fn default() -> Self {
        Self::new(Default::default())
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

    vertex_count: BufferAddress,
    indirect_count: BufferAddress,

    prev_width: u32,
    prev_height: u32,
}

impl CubeRenderer {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    const VERTEX_BUFFER_LAYOUTS: [VertexBufferLayout<'static>; 5] = [
        // Position
        VertexBufferLayout {
            array_stride: 4 * 3,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        },
        // Normal
        VertexBufferLayout {
            array_stride: 4 * 3,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 1,
            }],
        },
        // Texture coordinate
        VertexBufferLayout {
            array_stride: 4 * 2,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 2,
            }],
        },
        // Texture ID
        VertexBufferLayout {
            array_stride: 4,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Sint32,
                offset: 0,
                shader_location: 3,
            }],
        },
        // Instance
        VertexBufferLayout {
            array_stride: (4 * 8) + std::mem::size_of::<Instance>() as BufferAddress,
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
                            (4 * 8) + std::mem::size_of::<Instance>() as u64,
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
            size: 4 * 9 * vertex_count,
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
            size: ((4 * 8) + std::mem::size_of::<Instance>() as BufferAddress)
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

                let vertex_buffer_layouts = Self::VERTEX_BUFFER_LAYOUTS;

                let cube_pipeline =
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &render_shader,
                            entry_point: "vs_main",
                            buffers: &vertex_buffer_layouts,
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
                                buffers: &vertex_buffer_layouts,
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
            vertex_count,
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

        let vertex_len = self.vertex_count * 4 * 3;
        let normal_len = self.vertex_count * 4 * 3;
        let uv_len = self.vertex_count * 4 * 2;
        let texture_len = self.vertex_count * 4;

        rpass.push_debug_group("Prepare data for draw.");
        rpass.set_pipeline(cube_pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(0..vertex_len));
        rpass.set_vertex_buffer(
            1,
            self.vertex_buffer
                .slice(vertex_len..vertex_len + normal_len),
        );
        rpass.set_vertex_buffer(
            2,
            self.vertex_buffer
                .slice(vertex_len + normal_len..vertex_len + normal_len + uv_len),
        );
        rpass.set_vertex_buffer(
            3,
            self.vertex_buffer.slice(
                vertex_len + normal_len + uv_len..vertex_len + normal_len + uv_len + texture_len,
            ),
        );
        rpass.set_vertex_buffer(4, self.instance_buffer.slice(..));
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
    let pos = cgmath::Point3::new(total.sin() * 5.0, 4.5, total.cos() * 1.5);
    *eye_position = pos;
}

#[legion::system(par_for_each)]
pub fn update_instances(
    //position: Option<&Position3d>,
    //orientation: Option<&antigen_cgmath::components::Orientation>,
    visible: Option<&crate::components::Visible>,
    sphere_bounds: Option<&crate::components::SphereBounds>,
    instance: &mut InstanceComponent,
) {
    let mut inst = *instance.get();

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
        let quat: [f32; 4] = *orientation.get().as_ref();
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
#[read_component(VertexBuffer)]
#[read_component(BufferComponent)]
#[read_component(MeshId)]
#[read_component(MeshVertices<nalgebra::Vector3::<f32>>)]
#[read_component(MeshNormals<nalgebra::Vector3::<f32>>)]
#[read_component(MeshUvs<nalgebra::Vector2::<f32>>)]
#[read_component(MeshTextureIds<i32>)]
#[write_component(BufferWriteMeshVertices)]
#[write_component(BufferWriteMeshNormals)]
#[write_component(BufferWriteMeshUvs)]
#[write_component(BufferWriteMeshTextureIds)]
#[write_component(IndexedIndirectComponent)]
pub fn collect_vertices(world: &mut SubWorld) {
    let offsets = <(Entity, &VertexBuffer, &BufferComponent)>::query()
        .par_iter(world)
        .map(|(buffer_entity, _, _)| {
            let mut query = <(
                Entity,
                &MeshId,
                &MeshVertices<nalgebra::Vector3<f32>>,
                &MeshNormals<nalgebra::Vector3<f32>>,
                &MeshUvs<nalgebra::Vector2<f32>>,
                &MeshTextureIds<i32>,
                &BufferWriteMeshVertices,
                &BufferWriteMeshNormals,
                &BufferWriteMeshUvs,
                &BufferWriteMeshTextureIds,
                &IndexedIndirectComponent,
            )>::query();

            let meshes = query
                .par_iter(world)
                .filter_map(
                    |(entity, mesh_id, vertices, _, _, _, write_vertices, _, _, _, _)| {
                        let target_entity = write_vertices.to_entity().unwrap_or(entity);
                        if *target_entity == *buffer_entity {
                            Some((mesh_id, (entity, vertices)))
                        } else {
                            None
                        }
                    },
                )
                .collect::<BTreeMap<_, _>>();

            let mut offsets = vec![];
            let mut offset = 0;
            for (entity, vertices) in meshes.values() {
                let len = vertices.get().len();
                offsets.push((**entity, offset));
                offset += len;
            }

            (*buffer_entity, (offsets, offset))
        })
        .collect::<HashMap<_, _>>();

    for (offsets, total) in offsets.into_values() {
        for (mesh_entity, offset) in offsets {
            let (write_vertices, write_normals, write_uvs, write_texture_ids, indirect) = <(
                &mut BufferWriteMeshVertices,
                &mut BufferWriteMeshNormals,
                &mut BufferWriteMeshUvs,
                &mut BufferWriteMeshTextureIds,
                &mut IndexedIndirectComponent,
            )>::query(
            )
            .get_mut(world, mesh_entity)
            .unwrap();

            let vertex_offset = 4 * 3 * offset as u64;
            let normal_offset = 4 * 3 * offset as u64;
            let uv_offset = 4 * 2 * offset as u64;
            let texture_id_offset = 4 * offset as u64;

            let vertex_total = 4 * 3 * total as u64;
            let normal_total = 4 * 3 * total as u64;
            let uv_total = 4 * 2 * total as u64;

            write_vertices.set_offset(vertex_offset);
            write_normals.set_offset(normal_offset + vertex_total);
            write_uvs.set_offset(uv_offset + vertex_total + normal_total);
            write_texture_ids
                .set_offset(texture_id_offset + vertex_total + normal_total + uv_total);

            indirect.0.set_checked(DrawIndexedIndirect {
                vertex_offset: offset as i32,
                ..*indirect.0.get()
            });
        }
    }
}

/// Iterate over mesh entities, translate generic components into renderer indices
#[legion::system]
#[read_component(IndexBuffer)]
#[read_component(BufferComponent)]
#[read_component(MeshId)]
#[read_component(MeshTriangleIndices<Index>)]
#[write_component(BufferWriteMeshTriangleIndices)]
#[write_component(IndexedIndirectComponent)]
pub fn collect_indices(world: &mut SubWorld) {
    let offsets = <(Entity, &IndexBuffer, &BufferComponent)>::query()
        .par_iter(world)
        .map(|(buffer_entity, _, _)| {
            let mut query = <(
                Entity,
                &MeshId,
                &MeshTriangleIndices<Index>,
                &BufferWriteMeshTriangleIndices,
                &IndexedIndirectComponent,
            )>::query();

            let meshes = query
                .par_iter(world)
                .filter_map(|(entity, mesh_id, indices, write_indices, _)| {
                    let target_entity = write_indices.to_entity().unwrap_or(entity);
                    if *target_entity == *buffer_entity {
                        Some((mesh_id, (entity, indices)))
                    } else {
                        None
                    }
                })
                .collect::<BTreeMap<_, _>>();

            let mut offsets = vec![];
            let mut offset = 0;
            for (entity, indices) in meshes.values() {
                let len = indices.get().len();
                offsets.push((**entity, offset, len));
                offset += len;
            }

            (*buffer_entity, offsets)
        })
        .collect::<HashMap<_, _>>();

    for offsets in offsets.into_values() {
        for (mesh_entity, offset, len) in offsets.iter() {
            let (write_indices, indirect) = <(
                &mut BufferWriteMeshTriangleIndices,
                &mut IndexedIndirectComponent,
            )>::query()
            .get_mut(world, *mesh_entity)
            .unwrap();

            write_indices.set_offset((*offset * std::mem::size_of::<Index>()) as u64);
            indirect.0.set_checked(DrawIndexedIndirect {
                vertex_count: *len as u32,
                base_index: *offset as u32,
                ..*indirect.0.get()
            });
        }
    }
}

/// Iterate over mesh entities, translate generic components into instance and indirect data
#[legion::system]
#[read_component(InstanceComponent)]
#[read_component(MeshEntity)]
#[write_component(IndexedIndirectComponent)]
#[write_component(BufferWritePosition)]
#[write_component(BufferWriteOrientation)]
#[write_component(BufferWriteInstances)]
#[write_component(BufferWriteIndexedIndirect)]
pub fn collect_instances_indirects(world: &mut SubWorld) {
    let mut query = <(
        Entity,
        &InstanceComponent,
        &MeshEntity,
        &IndexedIndirectComponent,
        &BufferWritePosition,
        &BufferWriteOrientation,
        &BufferWriteInstances,
        &BufferWriteIndexedIndirect,
    )>::query();

    let position_size: wgpu::BufferAddress = 4 * 4;
    let orientation_size: wgpu::BufferAddress = 4 * 4;
    let instance_size = std::mem::size_of::<Instance>() as wgpu::BufferAddress;
    let total_size = position_size + orientation_size + instance_size;

    let offsets = query
        .iter(world)
        .enumerate()
        .map(
            |(offset, (entity, _, MeshEntity(mesh_entity), _, _, _, _, _))| {
                let mesh_indirect = <&IndexedIndirectComponent>::query()
                    .get(world, *mesh_entity)
                    .unwrap();
                (*entity, offset as u64, *mesh_indirect.0.get())
            },
        )
        .collect::<Vec<_>>();

    for (entity, offset, indirect) in offsets {
        let (
            indexed_indirect,
            buffer_write_position,
            buffer_write_orientation,
            buffer_write_instances,
            buffer_write_indexed_indirect,
        ) = <(
            &mut IndexedIndirectComponent,
            &mut BufferWritePosition,
            &mut BufferWriteOrientation,
            &mut BufferWriteInstances,
            &mut BufferWriteIndexedIndirect,
        )>::query()
        .get_mut(world, entity)
        .unwrap();
        let total_offset = total_size * offset;

        // Set instance write offsets
        buffer_write_position.set_offset(total_offset);
        buffer_write_orientation.set_offset(total_offset + position_size);
        buffer_write_instances.set_offset(total_offset + position_size + orientation_size);

        // Set indirect data
        indexed_indirect.0.set(DrawIndexedIndirect {
            base_instance: offset as u32,
            instance_count: 1,
            ..indirect
        });

        buffer_write_indexed_indirect
            .set_offset(std::mem::size_of::<DrawIndexedIndirect>() as wgpu::BufferAddress * offset);
    }
}
