use std::{borrow::Cow, rc::Rc};

use antigen_wgpu::{RenderPass, WgpuManager};
use lazy::Lazy;
use wgpu::{
    BindGroup, BindGroupLayout, Device, PipelineLayoutDescriptor, RenderPipeline,
    ShaderModuleDescriptor, TextureFormat, TextureView,
};

const RENDER_TARGET_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

pub struct ConservativeRasterRenderer {
    low_res_target: Option<TextureView>,
    bind_group_upscale: Option<BindGroup>,

    pipelines_triangle:
        Lazy<(RenderPipeline, RenderPipeline, Option<RenderPipeline>), (Rc<Device>, TextureFormat)>,
    pipeline_upscale: Lazy<RenderPipeline, (Rc<Device>, TextureFormat)>,

    bind_group_layout_upscale: BindGroupLayout,

    prev_width: u32,
    prev_height: u32,
}

impl ConservativeRasterRenderer {
    pub fn new(wgpu_manager: &WgpuManager) -> Self {
        let device = wgpu_manager.device();

        let pipeline_layout_empty = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let shader_triangle_and_lines = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "triangle_and_lines.wgsl"
            ))),
        });

        let pipelines = Lazy::new(Box::new(
            move |(device, format): (Rc<Device>, TextureFormat)| {
                let pipeline_conservative =
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Conservative Rasterization"),
                        layout: Some(&pipeline_layout_empty),
                        vertex: wgpu::VertexState {
                            module: &shader_triangle_and_lines,
                            entry_point: "vs_main",
                            buffers: &[],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader_triangle_and_lines,
                            entry_point: "fs_main_red",
                            targets: &[RENDER_TARGET_FORMAT.into()],
                        }),
                        primitive: wgpu::PrimitiveState {
                            conservative: true,
                            ..Default::default()
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState::default(),
                    });
                let pipeline_regular =
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Regular Rasterization"),
                        layout: Some(&pipeline_layout_empty),
                        vertex: wgpu::VertexState {
                            module: &shader_triangle_and_lines,
                            entry_point: "vs_main",
                            buffers: &[],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader_triangle_and_lines,
                            entry_point: "fs_main_blue",
                            targets: &[RENDER_TARGET_FORMAT.into()],
                        }),
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState::default(),
                    });
                let pipeline_lines = if device
                    .features()
                    .contains(wgpu::Features::POLYGON_MODE_LINE)
                {
                    Some(
                        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some("Lines"),
                            layout: Some(&pipeline_layout_empty),
                            vertex: wgpu::VertexState {
                                module: &shader_triangle_and_lines,
                                entry_point: "vs_main",
                                buffers: &[],
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &shader_triangle_and_lines,
                                entry_point: "fs_main_white",
                                targets: &[format.clone().into()],
                            }),
                            primitive: wgpu::PrimitiveState {
                                polygon_mode: wgpu::PolygonMode::Line,
                                topology: wgpu::PrimitiveTopology::LineStrip,
                                ..Default::default()
                            },
                            depth_stencil: None,
                            multisample: wgpu::MultisampleState::default(),
                        }),
                    )
                } else {
                    None
                };

                (pipeline_conservative, pipeline_regular, pipeline_lines)
            },
        ));

        let bind_group_layout_upscale =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("upscale bindgroup"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: false,
                            comparison: false,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout_upscale = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout_upscale],
            push_constant_ranges: &[],
        });
        let shader_upscale = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("upscale.wgsl"))),
        });

        let pipeline_upscale = Lazy::new(Box::new(
            move |(device, format): (Rc<Device>, TextureFormat)| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Upscale"),
                    layout: Some(&pipeline_layout_upscale),
                    vertex: wgpu::VertexState {
                        module: &shader_upscale,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_upscale,
                        entry_point: "fs_main",
                        targets: &[format.into()],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                })
            },
        ));

        ConservativeRasterRenderer {
            low_res_target: None,
            bind_group_upscale: None,

            pipelines_triangle: pipelines,
            pipeline_upscale,
            bind_group_layout_upscale,

            prev_width: Default::default(),
            prev_height: Default::default(),
        }
    }

    fn create_low_res_target(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        bind_group_layout_upscale: &BindGroupLayout,
    ) -> (TextureView, BindGroup) {
        let texture_view = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Low Resolution Target"),
                size: wgpu::Extent3d {
                    width: config.width / 16,
                    height: config.height / 16,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: RENDER_TARGET_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
            })
            .create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Nearest Neighbor Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("upscale bind group"),
            layout: bind_group_layout_upscale,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        (texture_view, bind_group)
    }
}

impl RenderPass for ConservativeRasterRenderer {
    fn render(
        &mut self,
        wgpu_manager: &WgpuManager,
        encoder: &mut wgpu::CommandEncoder,
        view: &TextureView,
        config: &wgpu::SurfaceConfiguration,
    ) {
        let device = wgpu_manager.device();

        if self.low_res_target.is_none()
            || self.bind_group_upscale.is_none()
            || config.width != self.prev_width
            || config.height != self.prev_height
        {
            let (low_res_target, bind_group_upscale) =
                Self::create_low_res_target(config, &device, &self.bind_group_layout_upscale);
            self.low_res_target = Some(low_res_target);
            self.bind_group_upscale = Some(bind_group_upscale);

            self.prev_width = config.width;
            self.prev_height = config.height;
        }

        let (pipeline_triangle_conservative, pipeline_triangle_regular, pipeline_lines) =
            self.pipelines_triangle.get((device.clone(), config.format));

        let low_res_target = self.low_res_target.as_ref().unwrap();
        let bind_group_upscale = self.bind_group_upscale.as_ref().unwrap();

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("low resolution"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: low_res_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(pipeline_triangle_conservative);
            rpass.draw(0..3, 0..1);
            rpass.set_pipeline(pipeline_triangle_regular);
            rpass.draw(0..3, 0..1);
        }

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("full resolution"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(self.pipeline_upscale.get((device, config.format)));
            rpass.set_bind_group(0, bind_group_upscale, &[]);
            rpass.draw(0..3, 0..1);

            if let Some(pipeline_lines) = pipeline_lines {
                rpass.set_pipeline(pipeline_lines);
                rpass.draw(0..4, 0..1);
            }
        }
    }
}
