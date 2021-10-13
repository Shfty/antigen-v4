use antigen_wgpu::{
    RenderPass, RenderPipelineConstructor, RenderPipelineId, WgpuManager,
};

pub fn triangle_render_pipeline(wgpu_manager: &WgpuManager) -> impl RenderPipelineConstructor {
    let triangle_shader_id = wgpu_manager.load_shader(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let device = wgpu_manager.device();

    let triangle_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    move |wgpu_manager: &WgpuManager, format: wgpu::ColorTargetState| {
        let triangle_shader = wgpu_manager.shader_module(&triangle_shader_id).unwrap();

        wgpu_manager
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&triangle_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &triangle_shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &triangle_shader,
                    entry_point: "fs_main",
                    targets: &[format],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
            })
    }
}

pub fn triangle_render_pass(pipeline: RenderPipelineId) -> impl RenderPass {
    move |encoder: &mut wgpu::CommandEncoder,
          wgpu_manager: &WgpuManager,
          view: &wgpu::TextureView,
          format: wgpu::ColorTargetState| {
        let render_pipeline = wgpu_manager.render_pipeline(&pipeline, format).unwrap();

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

        rpass.set_pipeline(&render_pipeline);
        rpass.draw(0..3, 0..1);
    }
}
