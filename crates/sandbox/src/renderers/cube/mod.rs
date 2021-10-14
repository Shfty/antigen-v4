use antigen_wgpu::{RenderPass, WgpuManager};
use antigen_winit::{WindowComponent, WindowManager, WinitRequester};
use bytemuck::{Pod, Zeroable};
use cgmath::Zero;
use legion::Entity;
use parking_lot::RwLock;
use serde::ser::SerializeStruct;
use std::{ops::Deref, sync::Arc};
use wgpu::{util::DeviceExt, SurfaceConfiguration};

use crate::resources::Timing;

#[rustfmt::skip]
#[allow(unused)]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

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

#[derive(Debug, Copy, Clone)]
pub struct CubeRenderState {
    pub fov: f32,
    pub eye: cgmath::Point3<f32>,
    pub z_near: f32,
    pub z_far: f32,
}

impl Default for CubeRenderState {
    fn default() -> Self {
        CubeRenderState {
            fov: 45.0,
            eye: cgmath::Point3::new(1.5, -5.0, 3.0),
            z_near: 1.0,
            z_far: 10.0,
        }
    }
}

impl serde::Serialize for CubeRenderState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("CubeRenderState", 2)?;
        s.serialize_field("fov", &self.fov)?;
        s.serialize_field("eye", &format!("{:?}", self.eye))?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for CubeRenderState {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct CubeRenderStateComponent(Arc<RwLock<CubeRenderState>>);

impl Deref for CubeRenderStateComponent {
    type Target = Arc<RwLock<CubeRenderState>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl serde::Serialize for CubeRenderStateComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.read().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for CubeRenderStateComponent {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl From<Arc<RwLock<CubeRenderState>>> for CubeRenderStateComponent {
    fn from(v: Arc<RwLock<CubeRenderState>>) -> Self {
        CubeRenderStateComponent(v)
    }
}

pub fn cube_render_pass(
    wgpu_manager: &WgpuManager,
    entity: Entity,
    state: Arc<RwLock<CubeRenderState>>,
) -> impl RenderPass {
    // Fetch resources
    let device = wgpu_manager.device();
    let queue = wgpu_manager.queue();

    // Create vertex and index buffers
    let (vertex_data, index_data) = create_vertices();

    let vertex_buffer_id = wgpu_manager.add_buffer(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        },
    ));

    let index_buffer_id = wgpu_manager.add_buffer(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        },
    ));

    // Create pipeline layout
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
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
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    // Create the texture
    let size = 256u32;
    let texels = create_texels(size as usize);
    let texture_extent = wgpu::Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
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

    // Create other resources
    let mx_total = cgmath::Matrix4::zero();
    let mx_ref: &[f32; 16] = mx_total.as_ref();
    let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(mx_ref),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let uniform_buffer_id = wgpu_manager.add_buffer(uniform_buf);
    let uniform_buf = wgpu_manager.buffer(&uniform_buffer_id).unwrap();

    // Create bind group
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
        ],
        label: None,
    });

    let bind_group_id = wgpu_manager.add_bind_group(bind_group);

    let vertex_buffer_layouts = [wgpu::VertexBufferLayout {
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

    let shader_id = wgpu_manager.load_shader(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let mut prev_width = 0u32;
    let mut prev_height = 0u32;
    let mut prev_eye = cgmath::Point3::new(0.0f32, 0.0, 0.0);
    let mut prev_fov = 0f32;

    let mut cube_pipe = None;
    let mut wire_pipe = None;

    move |encoder: &mut wgpu::CommandEncoder,
          wgpu_manager: &WgpuManager,
          view: &wgpu::TextureView,
          format: wgpu::ColorTargetState| {
        let device = wgpu_manager.device();
        let queue = wgpu_manager.queue();

        if cube_pipe.is_none() {
            let cube_shader = wgpu_manager.shader_module(&shader_id).unwrap();

            cube_pipe = Some(
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &cube_shader,
                        entry_point: "vs_main",
                        buffers: &vertex_buffer_layouts,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &cube_shader,
                        entry_point: "fs_main",
                        targets: &[format.clone().into()],
                    }),
                    primitive: wgpu::PrimitiveState {
                        cull_mode: Some(wgpu::Face::Back),
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                }),
            )
        }

        if wire_pipe.is_none()
            && device
                .features()
                .contains(wgpu::Features::NON_FILL_POLYGON_MODE)
        {
            let cube_shader = wgpu_manager.shader_module(&shader_id).unwrap();

            wire_pipe = Some(
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &cube_shader,
                        entry_point: "vs_main",
                        buffers: &vertex_buffer_layouts,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &cube_shader,
                        entry_point: "fs_wire",
                        targets: &[wgpu::ColorTargetState {
                            format: format.format,
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
        }

        // Fetch resources
        let cube_pipeline = cube_pipe.as_ref().unwrap();

        let bind_group = wgpu_manager.bind_group(&bind_group_id).unwrap();
        let index_buf = wgpu_manager.buffer(&index_buffer_id).unwrap();
        let vertex_buf = wgpu_manager.buffer(&vertex_buffer_id).unwrap();

        let state = state.read();
        let fov = state.fov;
        let eye = state.eye;
        drop(state);

        // Update matrix if surface size has changed
        let SurfaceConfiguration { width, height, .. } =
            wgpu_manager.surface_configuration(&entity).unwrap();

        let width = *width;
        let height = *height;
        if width != prev_width || height != prev_height || fov != prev_fov || eye != prev_eye {
            let aspect_ratio = width as f32 / height as f32;
            let mx_projection =
                cgmath::perspective(cgmath::Deg(fov.max(1.0).min(179.0)), aspect_ratio, 1.0, 10.0);
            let mx_view = cgmath::Matrix4::look_at_rh(
                eye,
                cgmath::Point3::new(0f32, 0.0, 0.0),
                cgmath::Vector3::unit_z(),
            );
            let mx_correction = OPENGL_TO_WGPU_MATRIX;
            let mx_total = mx_correction * mx_projection * mx_view;

            let uniform_buf = wgpu_manager.buffer(&uniform_buffer_id).unwrap();

            let mx_ref: &[f32; 16] = mx_total.as_ref();

            queue.write_buffer(&uniform_buf, 0, bytemuck::cast_slice(mx_ref));

            prev_width = width;
            prev_height = height;
            prev_fov = fov;
            prev_eye = eye;
        }

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
        rpass.set_pipeline(&cube_pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_vertex_buffer(0, vertex_buf.slice(..));
        rpass.pop_debug_group();
        rpass.insert_debug_marker("Draw!");
        rpass.draw_indexed(0..36, 0, 0..1);

        if let Some(ref pipeline) = wire_pipe.as_ref() {
            rpass.set_pipeline(pipeline);
            rpass.draw_indexed(0..36, 0, 0..1);
        }
    }
}

#[legion::system(par_for_each)]
pub fn integrate_cube_renderer(
    entity: &Entity,
    _: &mut WindowComponent,
    cube_render_state: &mut CubeRenderStateComponent,
    #[resource] timing: &Timing,
    #[resource] winit_requester: &WinitRequester,
) {
    let entity = *entity;

    let mut guard = cube_render_state.write();

    let total = timing.total_time().as_secs_f32();
    guard.eye.x = total.sin() * 1.5;
    guard.eye.y = total.cos() * -5.0;
    guard.fov = ((total * 0.2).sin() * 90.0) + 90.0;

    winit_requester.send_request(Box::new(move |window_manager: &mut WindowManager, _| {
        if let Some(window) = window_manager.window(&entity) {
            window.request_redraw();
        }
        Box::new(move |_| ())
    }));
}
