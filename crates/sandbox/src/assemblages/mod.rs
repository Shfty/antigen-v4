use antigen_wgpu::{RenderPassComponent, SurfaceComponent, WgpuManager};
use antigen_winit::WindowComponent;
use antigen_components::Name;
use legion::{Entity, World};

use crate::{
    components::*,
    renderers::cube::{CubeRenderer, UniformBufferComponent},
};

pub fn cube_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    // Cube renderer
    let cube_renderer = CubeRenderer::new(wgpu_manager);
    let uniform_buffer_component =
        UniformBufferComponent::from(cube_renderer.take_uniform_buffer_handle());
    let cube_pass_id = wgpu_manager.add_render_pass(Box::new(cube_renderer));

    let mut cube_pass_component = RenderPassComponent::default();
    cube_pass_component.add_render_pass(cube_pass_id);

    world.push((
        Name("Cube".into()),

        WindowComponent::always_redraw(),
        SurfaceComponent::default(),
        cube_pass_component,
        uniform_buffer_component,

        EyePosition(cgmath::Point3::new(0.0, 0.0, 5.0)),
        LookAt::default(),
        UpVector::default(),
        ViewMatrix::default(),

        PerspectiveProjection,
        FieldOfView::default(),
        AspectRatio::default(),
        NearPlane::default(),
        FarPlane::default(),
        ProjectionMatrix::default(),

        ViewProjectionMatrix::default(),
        UniformWrite::<ViewProjectionMatrix>::new(None, None, 0),
    ))
}
