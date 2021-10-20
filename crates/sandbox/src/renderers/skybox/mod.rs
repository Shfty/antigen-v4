use std::sync::Arc;

use antigen_wgpu::{RenderPass, WgpuManager};
use antigen_cgmath::OPENGL_TO_WGPU_MATRIX;

use cgmath::SquareMatrix;
use lazy::Lazy;
use wgpu::{util::DeviceExt, TextureFormat, Device};

const IMAGE_SIZE: u32 = 128;

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
}

struct Entity {
    vertex_count: u32,
    vertex_buf: wgpu::Buffer,
}

// Note: we use the Y=up coordinate space in this example.
struct Camera {
    screen_size: (u32, u32),
    angle_y: f32,
    angle_xz: f32,
    dist: f32,
}

const MODEL_CENTER_Y: f32 = 2.0;

impl Camera {
    fn to_uniform_data(&self) -> [f32; 16 * 3 + 4] {
        let aspect = self.screen_size.0 as f32 / self.screen_size.1 as f32;
        let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect, 1.0, 600.0);
        let cam_pos = cgmath::Point3::new(
            self.angle_xz.cos() * self.angle_y.sin() * self.dist,
            self.angle_xz.sin() * self.dist + MODEL_CENTER_Y,
            self.angle_xz.cos() * self.angle_y.cos() * self.dist,
        );
        let mx_view = cgmath::Matrix4::look_at_rh(
            cam_pos,
            cgmath::Point3::new(0f32, MODEL_CENTER_Y, 0.0),
            cgmath::Vector3::unit_y(),
        );
        let proj = OPENGL_TO_WGPU_MATRIX * mx_projection;
        let proj_inv = proj.invert().unwrap();
        let view = OPENGL_TO_WGPU_MATRIX * mx_view;

        let mut raw = [0f32; 16 * 3 + 4];
        raw[..16].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj)[..]);
        raw[16..32].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj_inv)[..]);
        raw[32..48].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&view)[..]);
        raw[48..51].copy_from_slice(AsRef::<[f32; 3]>::as_ref(&cam_pos));
        raw[51] = 1.0;
        raw
    }
}

pub struct SkyboxRenderer {
    camera: Camera,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    entities: Vec<Entity>,
    pipelines: Lazy<(wgpu::RenderPipeline, wgpu::RenderPipeline), (Arc<Device>, TextureFormat)>,
    depth_view: Option<wgpu::TextureView>,
}

impl SkyboxRenderer {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

    pub fn new(wgpu_manager: &WgpuManager) -> Self {
        let device = wgpu_manager.device();
        let queue = wgpu_manager.queue();

        let mut entities = Vec::new();
        {
            let source = include_bytes!("models/marauder.obj");
            let data = obj::ObjData::load_buf(&source[..]).unwrap();
            let mut vertices = Vec::new();
            for object in data.objects {
                for group in object.groups {
                    vertices.clear();
                    for poly in group.polys {
                        for end_index in 2..poly.0.len() {
                            for &index in &[0, end_index - 1, end_index] {
                                let obj::IndexTuple(position_id, _texture_id, normal_id) =
                                    poly.0[index];
                                vertices.push(Vertex {
                                    pos: data.position[position_id],
                                    normal: data.normal[normal_id.unwrap()],
                                })
                            }
                        }
                    }
                    let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                    entities.push(Entity {
                        vertex_count: vertices.len() as u32,
                        vertex_buf,
                    });
                }
            }
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::Cube,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            ],
        });

        // Create the render pipeline
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
        });

        let camera = Camera {
            screen_size: (1, 1),
            angle_xz: 0.2,
            angle_y: 0.2,
            dist: 500.0,
        };
        let raw_uniforms = camera.to_uniform_data();
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Buffer"),
            contents: bytemuck::cast_slice(&raw_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let device_features = device.features();

        let skybox_format =
            if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC_LDR) {
                log::info!("Using ASTC_LDR");
                wgpu::TextureFormat::Astc4x4RgbaUnormSrgb
            } else if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2) {
                log::info!("Using ETC2");
                wgpu::TextureFormat::Etc2RgbUnormSrgb
            } else if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC) {
                log::info!("Using BC");
                wgpu::TextureFormat::Bc1RgbaUnormSrgb
            } else {
                log::info!("Using plain");
                wgpu::TextureFormat::Bgra8UnormSrgb
            };

        let size = wgpu::Extent3d {
            width: IMAGE_SIZE,
            height: IMAGE_SIZE,
            depth_or_array_layers: 6,
        };

        let layer_size = wgpu::Extent3d {
            depth_or_array_layers: 1,
            ..size
        };
        let max_mips = layer_size.max_mips();

        log::debug!(
            "Copying {:?} skybox images of size {}, {}, 6 with {} mips to gpu",
            skybox_format,
            IMAGE_SIZE,
            IMAGE_SIZE,
            max_mips,
        );

        let bytes = match skybox_format {
            wgpu::TextureFormat::Astc4x4RgbaUnormSrgb => &include_bytes!("images/astc.dds")[..],
            wgpu::TextureFormat::Etc2RgbUnormSrgb => &include_bytes!("images/etc2.dds")[..],
            wgpu::TextureFormat::Bc1RgbaUnormSrgb => &include_bytes!("images/bc1.dds")[..],
            wgpu::TextureFormat::Bgra8UnormSrgb => &include_bytes!("images/bgra.dds")[..],
            _ => unreachable!(),
        };

        let image = ddsfile::Dds::read(&mut std::io::Cursor::new(&bytes)).unwrap();

        let texture = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                size,
                mip_level_count: max_mips as u32,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: skybox_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: None,
            },
            &image.data,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..wgpu::TextureViewDescriptor::default()
        });
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        let pipelines = Lazy::new(Box::new(
            move |(device, format): (Arc<Device>, TextureFormat)| {
                // Create the render pipelines
                let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Sky"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_sky",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_sky",
                        targets: &[format.clone().into()],
                    }),
                    primitive: wgpu::PrimitiveState {
                        front_face: wgpu::FrontFace::Cw,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: Self::DEPTH_FORMAT,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                });

                let entity_pipeline = device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("Entity"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_entity",
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                        }],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_entity",
                        targets: &[format.into()],
                    }),
                    primitive: wgpu::PrimitiveState {
                        front_face: wgpu::FrontFace::Cw,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: Self::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                },
            );

                (sky_pipeline, entity_pipeline)
            },
        ));

        SkyboxRenderer {
            camera,
            bind_group,
            uniform_buf,
            pipelines,
            entities,
            depth_view: None,
        }
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

impl RenderPass for SkyboxRenderer {
    fn render(
        &mut self,
        wgpu_manager: &WgpuManager,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        config: &wgpu::SurfaceConfiguration,
    ) {
        let device = wgpu_manager.device();
        let queue = wgpu_manager.queue();

        if self.depth_view.is_none()
            || config.width != self.camera.screen_size.0
            || config.height != self.camera.screen_size.1
        {
            self.depth_view = Some(Self::create_depth_texture(config, &device));

            self.camera.screen_size.0 = config.width;
            self.camera.screen_size.1 = config.height;
        }

        // update rotation
        let raw_uniforms = self.camera.to_uniform_data();
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&raw_uniforms));

        let (sky_pipeline, entity_pipeline) = self.pipelines.get((device, config.format));
        let depth_view = self.depth_view.as_ref().unwrap();

        {
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
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_pipeline(entity_pipeline);

            for entity in self.entities.iter() {
                rpass.set_vertex_buffer(0, entity.vertex_buf.slice(..));
                rpass.draw(0..entity.vertex_count, 0..1);
            }

            rpass.set_pipeline(sky_pipeline);
            rpass.draw(0..3, 0..1);
        }
    }
}
