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
use wgpu::BufferAddress;

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
type BufferWriteInstances =
    BufferWrite<crate::renderers::cube::InstanceComponent, crate::renderers::cube::Instance>;
type BufferWriteIndexedIndirect = BufferWrite<
    crate::renderers::cube::IndexedIndirectComponent,
    antigen_wgpu::DrawIndexedIndirect,
>;
type TextureWriteImage = TextureWrite<ImageComponent, Image>;

legion_debugger::register_component!(BufferWriteVertices);
legion_debugger::register_component!(BufferWriteIndices);
legion_debugger::register_component!(BufferWriteInstances);
legion_debugger::register_component!(BufferWriteViewProjectionMatrix);
legion_debugger::register_component!(BufferWriteIndexedIndirect);
legion_debugger::register_component!(TextureWriteImage);

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
    let (tetrahedron_vertices, tetrahedron_indices) = CubeRenderer::tetrahedron_vertices();
    let (cube_vertices, cube_indices) = CubeRenderer::cube_vertices();

    let tetrahedron_count = 16u32;
    let cube_count = 16u32;

    // Cube renderer
    let cube_renderer = CubeRenderer::new(
        wgpu_manager,
        (tetrahedron_vertices.len() + cube_vertices.len()) as BufferAddress,
        (tetrahedron_indices.len() + cube_indices.len()) as BufferAddress,
        2,
        (tetrahedron_count + cube_count) as BufferAddress,
    );

    let uniform_buffer_component =
        BufferComponent::from(cube_renderer.take_uniform_buffer_handle());

    let vertex_buffer_component = BufferComponent::from(cube_renderer.take_vertex_buffer_handle());
    let index_buffer_component = BufferComponent::from(cube_renderer.take_index_buffer_handle());
    let instance_buffer_component =
        BufferComponent::from(cube_renderer.take_instance_buffer_handle());
    let indirect_buffer_component =
        BufferComponent::from(cube_renderer.take_indirect_buffer_handle());
    let texture_component = TextureComponent::from(cube_renderer.take_texture_handle());

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
        BufferWriteViewProjectionMatrix::new(None, None, 0),
    ));

    let vertex_buffer_entity = world.push((vertex_buffer_component,));
    let index_buffer_entity = world.push((index_buffer_component,));
    let instance_buffer_entity = world.push((instance_buffer_component,));
    let indirect_buffer_entity = world.push((indirect_buffer_component,));

    // Tetrahedron mesh
    let tetrahedron_vertices_entity = world.push((
        crate::renderers::cube::Vertices::new(tetrahedron_vertices),
        BufferWriteVertices::new(None, Some(vertex_buffer_entity), 0),
    ));

    let tetrahedron_indices_entity = world.push((
        crate::renderers::cube::Indices::new(tetrahedron_indices),
        BufferWriteIndices::new(None, Some(index_buffer_entity), 0),
    ));

    // Cube mesh
    let cube_vertices_entity = world.push((
        crate::renderers::cube::Vertices::new(cube_vertices),
        BufferWriteVertices::new(
            None,
            Some(vertex_buffer_entity),
            (std::mem::size_of::<crate::renderers::cube::Vertex>() * 4) as wgpu::BufferAddress,
        ),
    ));

    let cube_indices_entity = world.push((
        crate::renderers::cube::Indices::new(cube_indices),
        BufferWriteIndices::new(
            None,
            Some(index_buffer_entity),
            (std::mem::size_of::<u16>() * 12) as wgpu::BufferAddress,
        ),
    ));

    // Instances
    let mut dir = cgmath::Vector4::unit_x();
    for i in 0..tetrahedron_count {
        let foo: cgmath::Vector3<f32> = dir.xyz();
        world.push((
            crate::renderers::cube::InstanceComponent::new(
                cgmath::Matrix4::<f32>::from_translation(foo * 3.0),
            ),
            BufferWriteInstances::new(
                None,
                Some(instance_buffer_entity),
                std::mem::size_of::<crate::renderers::cube::Instance>() as wgpu::BufferAddress
                    * i as wgpu::BufferAddress,
            ),
        ));

        dir = cgmath::Matrix4::from_angle_z(cgmath::Deg(360.0 / tetrahedron_count as f32)) * dir;
    }

    let mut dir = cgmath::Vector4::unit_z();
    for i in 0..cube_count {
        let foo: cgmath::Vector3<f32> = dir.xyz();
        world.push((
            crate::renderers::cube::InstanceComponent::new(
                cgmath::Matrix4::<f32>::from_translation(foo * 3.0),
            ),
            BufferWriteInstances::new(
                None,
                Some(instance_buffer_entity),
                std::mem::size_of::<crate::renderers::cube::Instance>() as wgpu::BufferAddress
                    * (i + tetrahedron_count) as wgpu::BufferAddress,
            ),
        ));

        dir = cgmath::Matrix4::from_angle_x(cgmath::Deg(360.0 / cube_count as f32)) * dir;
    }

    // Indirect draw data
    let tetrahedron_indirect = antigen_wgpu::DrawIndexedIndirect {
        vertex_count: 12,
        instance_count: tetrahedron_count,
        base_index: 0,
        vertex_offset: 0,
        base_instance: 0,
    };

    let tetrahedron_indirect_entity = world.push((
        crate::renderers::cube::IndexedIndirectComponent::new(tetrahedron_indirect),
        BufferWriteIndexedIndirect::new(None, Some(indirect_buffer_entity), 0),
    ));

    let cube_indirect = antigen_wgpu::DrawIndexedIndirect {
        vertex_count: 36,
        instance_count: cube_count,
        base_index: 12,
        vertex_offset: 4,
        base_instance: tetrahedron_count,
    };

    let cube_indirect_entity = world.push((
        crate::renderers::cube::IndexedIndirectComponent::new(cube_indirect),
        BufferWriteIndexedIndirect::new(
            None,
            Some(indirect_buffer_entity),
            (std::mem::size_of::<antigen_wgpu::DrawIndexedIndirect>()) as wgpu::BufferAddress,
        ),
    ));

    // Mandelbrot texture
    let texture_entity = world.push((
        ImageComponent::from(Image::mandelbrot_r8(256)),
        TextureWriteImage::new(
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
