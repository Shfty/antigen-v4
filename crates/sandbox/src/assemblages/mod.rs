use antigen_cgmath::components::{
    AspectRatio, EyePosition, FarPlane, FieldOfView, LookAt, NearPlane, PerspectiveProjection,
    ProjectionMatrix, UpVector, ViewMatrix, ViewProjectionMatrix,
};
use antigen_components::{Image, ImageComponent};
use antigen_wgpu::{
    components::{
        BufferComponent, BufferWrite, RenderPassComponent, SurfaceComponent, TextureComponent,
        TextureWrite,
    },
    WgpuManager,
};
use antigen_winit::components::{RedrawMode, RedrawModeComponent, WindowComponent, WindowTitle};
use legion::{Entity, World};
use on_change::OnChange;

use crate::renderers::{
    boids::BoidsRenderer, bunnymark::BunnymarkRenderer,
    conservative_raster::ConservativeRasterRenderer, cube::CubeRenderer,
    hello_triangle::TriangleRenderer, mipmap::MipmapRenderer, msaa_lines::MsaaLinesRenderer,
    shadow::ShadowRenderer, skybox::SkyboxRenderer, texture_arrays::TextureArraysRenderer,
    water::WaterRenderer,
};

type BufferWriteViewProjectionMatrix =
    BufferWrite<ViewProjectionMatrix, antigen_cgmath::cgmath::Matrix4<f32>>;
type BufferWriteVertices =
    BufferWrite<crate::renderers::cube::Vertices, Vec<crate::renderers::cube::Vertex>>;
type BufferWriteIndices = BufferWrite<crate::renderers::cube::Indices, Vec<u16>>;
type TextureWriteImageComponent = TextureWrite<ImageComponent, Image>;

legion_debugger::register_component!(BufferWriteVertices);
legion_debugger::register_component!(BufferWriteIndices);
legion_debugger::register_component!(BufferWriteViewProjectionMatrix);
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

pub fn cube_renderer(world: &mut World, wgpu_manager: &WgpuManager) -> (Entity, Entity) {
    // Cube renderer
    let cube_renderer = CubeRenderer::new(wgpu_manager);
    let uniform_buffer_component =
        BufferComponent::from(cube_renderer.take_uniform_buffer_handle());

    let texture_component = TextureComponent::from(cube_renderer.take_texture_handle());
    let vertex_buffer_component = BufferComponent::from(cube_renderer.take_vertex_buffer_handle());
    let index_buffer_component = BufferComponent::from(cube_renderer.take_index_buffer_handle());

    let cube_pass_id = wgpu_manager.add_render_pass(Box::new(cube_renderer));

    let mut cube_pass_component = RenderPassComponent::default();
    cube_pass_component.add_render_pass(cube_pass_id);

    let cube_renderer_entity = world.push((
        WindowComponent::default(),
        WindowTitle::from("Cube"),
        RedrawModeComponent::from(RedrawMode::MainEventsClearedLoop),
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
        BufferWrite::<ViewProjectionMatrix, antigen_cgmath::cgmath::Matrix4<f32>>::new(
            None, None, 0,
        ),
    ));

    let (vertices, indices) = CubeRenderer::create_vertices();

    let vertices_entity = world.push((
        crate::renderers::cube::Vertices::new(vertices),
        vertex_buffer_component,
        BufferWrite::<crate::renderers::cube::Vertices, Vec<crate::renderers::cube::Vertex>>::new(
            None, None, 0,
        ),
    ));

    let indices_entity = world.push((
        crate::renderers::cube::Indices::new(indices),
        index_buffer_component,
        BufferWrite::<crate::renderers::cube::Indices, Vec<u16>>::new(None, None, 0),
    ));

    // Mandelbrot texture
    let texture_entity = world.push((
        ImageComponent::from(Image::mandelbrot_r8(256)),
        TextureWrite::<ImageComponent, Image>::new(
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
        texture_component,
    ));

    (cube_renderer_entity, texture_entity)
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
