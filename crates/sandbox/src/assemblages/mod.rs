use antigen_cgmath::components::{
    AspectRatio, EyePosition, FarPlane, FieldOfView, LookAt, NearPlane, PerspectiveProjection,
    ProjectionMatrix, UpVector, ViewMatrix, ViewProjectionMatrix,
};
use antigen_components::{Image, ImageComponent};
use antigen_wgpu::{
    components::{
        RenderPassComponent, SurfaceComponent, TextureComponent, TextureWrite,
        UniformBufferComponent, UniformWrite,
    },
    WgpuManager,
};
use antigen_winit::components::{RedrawMode, RedrawModeComponent, WindowComponent, WindowTitle};
use legion::{Entity, World};

use crate::renderers::{
    boids::BoidsRenderer, bunnymark::BunnymarkRenderer,
    conservative_raster::ConservativeRasterRenderer, cube::CubeRenderer,
    hello_triangle::TriangleRenderer, mipmap::MipmapRenderer, msaa_lines::MsaaLinesRenderer,
    shadow::ShadowRenderer, skybox::SkyboxRenderer, texture_arrays::TextureArraysRenderer,
    water::WaterRenderer,
};

type UniformWriteViewProjectionMatrix = UniformWrite<ViewProjectionMatrix>;
legion_debugger::register_component!(UniformWriteViewProjectionMatrix);

type TextureWriteImageComponent = TextureWrite<ImageComponent>;
legion_debugger::register_component!(TextureWriteImageComponent);

pub fn hello_triangle_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    let triangle_pass_id =
        wgpu_manager.add_render_pass(Box::new(TriangleRenderer::new(&wgpu_manager)));

    let mut triangle_pass_component = RenderPassComponent::default();
    triangle_pass_component.add_render_pass(triangle_pass_id);

    world.push((
        WindowComponent::default(),
        WindowTitle::from("Hello Triangle"),
        SurfaceComponent::default(),
        triangle_pass_component,
    ))
}

pub fn cube_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    // Cube renderer
    let cube_renderer = CubeRenderer::new(wgpu_manager);
    let uniform_buffer_component =
        UniformBufferComponent::from(cube_renderer.take_uniform_buffer_handle());
    let texture_component = TextureComponent::from(cube_renderer.take_texture_handle());

    let cube_pass_id = wgpu_manager.add_render_pass(Box::new(cube_renderer));

    let mut cube_pass_component = RenderPassComponent::default();
    cube_pass_component.add_render_pass(cube_pass_id);

    world.push((
        WindowComponent::default(),
        WindowTitle::from("Cube"),
        RedrawModeComponent::from(RedrawMode::MainEventsClearedLoop),
        SurfaceComponent::default(),
        cube_pass_component,
        uniform_buffer_component,
        texture_component,
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
        ImageComponent::from(Image::mandelbrot_r8(256)),
        TextureWrite::<ImageComponent>::new(
            None,
            None,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(std::num::NonZeroU32::new(256).unwrap()),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
        ),
    ))
}

pub fn msaa_lines_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    let msaa_lines_pass_id =
        wgpu_manager.add_render_pass(Box::new(MsaaLinesRenderer::new(&wgpu_manager)));

    let mut msaa_lines_pass_component = RenderPassComponent::default();
    msaa_lines_pass_component.add_render_pass(msaa_lines_pass_id);

    world.push((
        WindowComponent::default(),
        WindowTitle::from("MSAA Lines"),
        SurfaceComponent::default(),
        msaa_lines_pass_component,
    ))
}

pub fn boids_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    let boids_pass_id = wgpu_manager.add_render_pass(Box::new(BoidsRenderer::new(&wgpu_manager)));

    let mut boids_pass_component = RenderPassComponent::default();
    boids_pass_component.add_render_pass(boids_pass_id);

    world.push((
        WindowComponent::default(),
        WindowTitle::from("Boids"),
        RedrawModeComponent::from(RedrawMode::MainEventsClearedRequest),
        SurfaceComponent::default(),
        boids_pass_component,
    ))
}

pub fn conservative_raster_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    let conservative_raster_pass_id =
        wgpu_manager.add_render_pass(Box::new(ConservativeRasterRenderer::new(&wgpu_manager)));

    let mut conservative_raster_pass_component = RenderPassComponent::default();
    conservative_raster_pass_component.add_render_pass(conservative_raster_pass_id);

    world.push((
        WindowComponent::default(),
        WindowTitle::from("Conservative Raster"),
        SurfaceComponent::default(),
        conservative_raster_pass_component,
    ))
}

pub fn mipmap_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    let mipmap_pass_id = wgpu_manager.add_render_pass(Box::new(MipmapRenderer::new(wgpu_manager)));

    let mut mipmap_pass_component = RenderPassComponent::default();
    mipmap_pass_component.add_render_pass(mipmap_pass_id);

    world.push((
        WindowComponent::default(),
        WindowTitle::from("Mipmaps"),
        SurfaceComponent::default(),
        mipmap_pass_component,
    ))
}

pub fn texture_arrays_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    let texture_arrays_pass_id =
        wgpu_manager.add_render_pass(Box::new(TextureArraysRenderer::new(&wgpu_manager)));

    let mut texture_arrays_pass_component = RenderPassComponent::default();
    texture_arrays_pass_component.add_render_pass(texture_arrays_pass_id);

    world.push((
        WindowComponent::default(),
        WindowTitle::from("Texture Arrays"),
        SurfaceComponent::default(),
        texture_arrays_pass_component,
    ))
}

pub fn shadow_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    let shadow_pass_id = wgpu_manager.add_render_pass(Box::new(ShadowRenderer::new(&wgpu_manager)));
    let mut shadow_pass_component = RenderPassComponent::default();
    shadow_pass_component.add_render_pass(shadow_pass_id);

    world.push((
        WindowComponent::default(),
        RedrawModeComponent::from(RedrawMode::MainEventsClearedLoop),
        WindowTitle::from("Shadows"),
        SurfaceComponent::default(),
        shadow_pass_component,
    ))
}

pub fn bunnymark_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    let bunnymark_pass_id =
        wgpu_manager.add_render_pass(Box::new(BunnymarkRenderer::new(&wgpu_manager)));
    let mut bunnymark_pass_component = RenderPassComponent::default();
    bunnymark_pass_component.add_render_pass(bunnymark_pass_id);

    world.push((
        WindowComponent::default(),
        RedrawModeComponent::from(RedrawMode::MainEventsClearedLoop),
        WindowTitle::from("Bunnymark"),
        SurfaceComponent::default(),
        bunnymark_pass_component,
    ))
}

pub fn skybox_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    let skybox_pass_id = wgpu_manager.add_render_pass(Box::new(SkyboxRenderer::new(&wgpu_manager)));

    let mut skybox_pass_component = RenderPassComponent::default();
    skybox_pass_component.add_render_pass(skybox_pass_id);

    world.push((
        WindowComponent::default(),
        WindowTitle::from("Skybox"),
        SurfaceComponent::default(),
        skybox_pass_component,
    ))
}

pub fn water_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> Entity {
    let water_pass_id = wgpu_manager.add_render_pass(Box::new(WaterRenderer::new(&wgpu_manager)));

    let mut water_pass_component = RenderPassComponent::default();
    water_pass_component.add_render_pass(water_pass_id);

    world.push((
        WindowComponent::default(),
        RedrawModeComponent::from(RedrawMode::MainEventsClearedLoop),
        WindowTitle::from("Water"),
        SurfaceComponent::default(),
        water_pass_component,
    ))
}
