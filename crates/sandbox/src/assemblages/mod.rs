use std::num::NonZeroU32;

use antigen_cgmath::components::{
    AspectRatio, EyePosition, FarPlane, FieldOfView, LookAt, NearPlane, PerspectiveProjection,
    ProjectionMatrix, UpVector, ViewProjectionMatrix,
};
use antigen_components::{Image, ImageComponent};
use antigen_rapier3d::rapier3d::prelude::*;
use antigen_wgpu::{
    components::{
        BufferComponent, BufferWrite, RenderPassComponent, SurfaceComponent, TextureComponent,
        TextureWrite,
    },
    WgpuManager,
};
use antigen_winit::components::{RedrawMode, RedrawModeComponent, WindowComponent, WindowTitle};

use cgmath::{InnerSpace, One, Zero};
use legion::{Entity, World};
use wgpu::BufferAddress;

use crate::renderers::{
    boids::BoidsRenderer,
    bunnymark::BunnymarkRenderer,
    conservative_raster::ConservativeRasterRenderer,
    cube::{CubeRenderer, Uniforms, UniformsComponent},
    hello_triangle::TriangleRenderer,
    mipmap::MipmapRenderer,
    msaa_lines::MsaaLinesRenderer,
    shadow::ShadowRenderer,
    skybox::SkyboxRenderer,
    texture_arrays::TextureArraysRenderer,
    water::WaterRenderer,
};

type BufferWriteViewProjectionMatrix =
    BufferWrite<ViewProjectionMatrix, antigen_cgmath::cgmath::Matrix4<f32>>;
type BufferWriteUniforms = BufferWrite<UniformsComponent, Uniforms>;
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
legion_debugger::register_component!(BufferWriteUniforms);
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

pub fn cube_renderer(world: &mut World, wgpu_manager: &WgpuManager) {
    let (tetrahedron_vertices, tetrahedron_indices) = CubeRenderer::tetrahedron_vertices();
    let (cube_vertices, cube_indices) = CubeRenderer::cube_vertices();

    let tetrahedron_count = 16u32;
    let cube_count = 16u32;

    let tetrahedron_vertices_len = tetrahedron_vertices.len();
    let tetrahedron_indices_len = tetrahedron_indices.len();

    let cube_vertices_len = cube_vertices.len();
    let cube_indices_len = cube_indices.len();

    // Load obj
    //let source = include_bytes!("../renderers/skybox/models/marauder.obj");
    //let data = obj::ObjData::load_buf(&source[..]).unwrap();

    /*
    let mut obj_vertices = Vec::new();
    let mut obj_indices = Vec::new();

    for object in data.objects {
        for group in object.groups {
            for poly in group.polys {
                for end_index in 2..poly.0.len() {
                    for &index in &[0, end_index - 1, end_index] {
                        let obj::IndexTuple(position_id, texture_id, _) = poly.0[index];

                        let mut pos = data.position[position_id];
                        pos[0] *= 0.025;
                        pos[1] *= 0.025;
                        pos[2] *= 0.025;

                        obj_vertices.push(crate::renderers::cube::vertex(
                            pos,
                            data.texture[texture_id.unwrap()],
                            0,
                        ));

                        obj_indices.push(obj_indices.len() as u16);
                    }
                }
            }
        }
    }
    */

    // Parse map
    let map = include_str!("../../../../../sif/crates/shalrath/test_data/abstract-test.map");
    let map = map.parse::<shambler::shalrath::repr::Map>().unwrap();

    // Build mesh
    let shambler::Mesh {
        vertices,
        normals,
        uvs,
        triangle_indices,
        ..
    } = shambler::map_mesh(map);

    let mut obj_vertices = vertices
        .into_iter()
        .zip(normals.into_iter().zip(uvs.into_iter()))
        .map(|(vertex, (normal, uv))| {
            crate::renderers::cube::vertex(
                [vertex.x / 64.0, vertex.z / 64.0, vertex.y / 64.0],
                [normal.x, normal.z, normal.y],
                [uv.x, uv.y],
                0,
            )
        })
        .collect::<Vec<_>>();

    let mut obj_indices = triangle_indices
        .into_iter()
        .map(|i| i as u16)
        .collect::<Vec<_>>();

    obj_vertices.resize(
        obj_vertices.len() + (obj_vertices.len() % wgpu::COPY_BUFFER_ALIGNMENT as usize),
        Default::default(),
    );
    obj_indices.resize(
        obj_indices.len() + (obj_indices.len() % wgpu::COPY_BUFFER_ALIGNMENT as usize),
        Default::default(),
    );

    let obj_vertices_len = obj_vertices.len();
    let obj_indices_len = obj_indices.len();

    let vertex_count = tetrahedron_vertices_len + cube_vertices_len + obj_vertices_len;
    let index_count = tetrahedron_indices_len + cube_indices_len + obj_indices_len;
    let instance_count = tetrahedron_count + cube_count + 1;

    // Physics simulation
    let physics_sim_entity = world.push((
        antigen_rapier3d::Gravity(antigen_rapier3d::rapier3d::prelude::vector![
            0.0, -9.81, 0.0
        ]),
        RigidBodySet::new(),
        ColliderSet::new(),
        IntegrationParameters::default(),
        antigen_rapier3d::PhysicsPipelineComponent(PhysicsPipeline::new()),
        IslandManager::new(),
        BroadPhase::new(),
        NarrowPhase::new(),
        JointSet::new(),
        CCDSolver::new(),
    ));

    // Cube renderer
    let cube_renderer = CubeRenderer::new(
        wgpu_manager,
        vertex_count as BufferAddress,
        index_count as BufferAddress,
        instance_count as BufferAddress,
        instance_count as BufferAddress,
        2,
    );

    let uniform_buffer_component =
        BufferComponent::from(cube_renderer.take_uniform_buffer_handle());

    let vertex_buffer_component = BufferComponent::from(cube_renderer.take_vertex_buffer_handle());
    let index_buffer_component = BufferComponent::from(cube_renderer.take_index_buffer_handle());
    let instance_buffer_component =
        BufferComponent::from(cube_renderer.take_instance_buffer_handle());
    let indirect_buffer_component =
        BufferComponent::from(cube_renderer.take_indirect_buffer_handle());

    // Texture array
    let texture_entity = world.push((TextureComponent::from(cube_renderer.take_texture_handle()),));

    let mandelbrot_texture_entity = world.push((
        ImageComponent::from(Image::mandelbrot_r8(256)),
        TextureWriteImage::new(
            None,
            Some(texture_entity),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(256).unwrap()),
                rows_per_image: Some(NonZeroU32::new(256).unwrap()),
            },
            wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            wgpu::ImageCopyTextureBase {
                texture: (),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
        ),
    ));

    let inverse_mandelbrot_texture_entity = world.push((
        ImageComponent::from(Image::mandelbrot_r8(256).inverse()),
        TextureWriteImage::new(
            None,
            Some(texture_entity),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(256).unwrap()),
                rows_per_image: Some(NonZeroU32::new(256).unwrap()),
            },
            wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            wgpu::ImageCopyTextureBase {
                texture: (),
                mip_level: 0,
                origin: wgpu::Origin3d {
                    z: 1,
                    ..wgpu::Origin3d::ZERO
                },
                aspect: wgpu::TextureAspect::All,
            },
        ),
    ));

    // Register pass with renderer
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
        antigen_cgmath::components::Orientation::default(),
        PerspectiveProjection,
        FieldOfView::default(),
        AspectRatio::default(),
        NearPlane::default(),
        FarPlane(200.0),
        ProjectionMatrix::default(),
        UniformsComponent::new(cgmath::Matrix4::one()),
        BufferWriteUniforms::new(None, None, 0),
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
            (std::mem::size_of::<crate::renderers::cube::Vertex>() * tetrahedron_vertices_len)
                as wgpu::BufferAddress,
        ),
    ));

    let cube_indices_entity = world.push((
        crate::renderers::cube::Indices::new(cube_indices),
        BufferWriteIndices::new(
            None,
            Some(index_buffer_entity),
            (std::mem::size_of::<crate::renderers::cube::Index>() * tetrahedron_indices_len)
                as wgpu::BufferAddress,
        ),
    ));

    // OBJ mesh
    let obj_vertices_entity = world.push((
        crate::renderers::cube::Vertices::new(obj_vertices),
        BufferWriteVertices::new(
            None,
            Some(vertex_buffer_entity),
            (std::mem::size_of::<crate::renderers::cube::Vertex>()
                * (tetrahedron_vertices_len + cube_vertices_len))
                as wgpu::BufferAddress,
        ),
    ));

    let obj_indices_entity = world.push((
        crate::renderers::cube::Indices::new(obj_indices),
        BufferWriteIndices::new(
            None,
            Some(index_buffer_entity),
            (std::mem::size_of::<crate::renderers::cube::Index>()
                * (tetrahedron_indices_len + cube_indices_len)) as wgpu::BufferAddress,
        ),
    ));

    // Indirect draw data
    let tetrahedron_indirect = antigen_wgpu::DrawIndexedIndirect {
        vertex_count: tetrahedron_indices_len as u32,
        instance_count: 1,
        base_index: 0,
        vertex_offset: 0,
        base_instance: 0,
    };

    let cube_indirect = antigen_wgpu::DrawIndexedIndirect {
        vertex_count: cube_indices_len as u32,
        instance_count: 1,
        base_index: tetrahedron_indices_len as u32,
        vertex_offset: tetrahedron_vertices_len as i32,
        base_instance: 0,
    };

    let obj_indirect = antigen_wgpu::DrawIndexedIndirect {
        vertex_count: obj_indices_len as u32,
        instance_count: 1,
        base_index: (tetrahedron_indices_len + cube_indices_len) as u32,
        vertex_offset: (tetrahedron_vertices_len + cube_vertices_len) as i32,
        base_instance: 0,
    };

    // Floor entity
    let floor_entity = world.push((antigen_rapier3d::ColliderComponent {
        physics_sim_entity,
        parent_entity: None,
        pending_collider: Some(
            ColliderBuilder::cuboid(100.0, 0.1, 100.0)
                .translation(antigen_rapier3d::rapier3d::prelude::vector![0.0, -5.0, 0.0])
                .build(),
        ),
        parent_handle: None,
        handle: None,
    },));

    // Tetrahedron entities
    let mut dir = cgmath::Vector4::unit_z();
    for i in 0..tetrahedron_count {
        let offset: cgmath::Vector3<f32> = dir.xyz();
        let entity = world.push((
            antigen_cgmath::components::Position3d::new(offset * 3.0),
            antigen_cgmath::components::Orientation::default(),
            antigen_rapier3d::RigidBodyComponent {
                physics_sim_entity,
                pending_rigid_body: Some(RigidBodyBuilder::new_dynamic().build()),
                handle: None,
            },
            antigen_rapier3d::ColliderComponent {
                physics_sim_entity,
                parent_entity: None,
                pending_collider: Some(ColliderBuilder::ball(0.5).restitution(0.7).build()),
                parent_handle: None,
                handle: None,
            },
            crate::components::SphereBounds(1.0),
            crate::renderers::cube::InstanceComponent::default(),
            BufferWriteInstances::new(
                None,
                Some(instance_buffer_entity),
                std::mem::size_of::<crate::renderers::cube::Instance>() as wgpu::BufferAddress
                    * i as wgpu::BufferAddress,
            ),
            crate::renderers::cube::IndexedIndirectComponent::new(
                antigen_wgpu::DrawIndexedIndirect {
                    base_instance: i,
                    ..tetrahedron_indirect
                },
            ),
            BufferWriteIndexedIndirect::new(
                None,
                Some(indirect_buffer_entity),
                std::mem::size_of::<antigen_wgpu::DrawIndexedIndirect>() as wgpu::BufferAddress
                    * i as wgpu::BufferAddress,
            ),
        ));

        dir = cgmath::Matrix4::from_angle_x(cgmath::Deg(360.0 / tetrahedron_count as f32)) * dir;
    }

    // Cube entities
    let mut dir = cgmath::Vector4::unit_z();
    for i in 0..cube_count {
        let offset: cgmath::Vector3<f32> = dir.xyz();
        world.push((
            antigen_cgmath::components::Position3d::new(offset * 3.0),
            crate::components::SphereBounds(1.0),
            antigen_rapier3d::RigidBodyComponent {
                physics_sim_entity,
                pending_rigid_body: Some(RigidBodyBuilder::new_kinematic_velocity_based().build()),
                handle: None,
            },
            antigen_rapier3d::ColliderComponent {
                physics_sim_entity,
                parent_entity: None,
                pending_collider: Some(
                    ColliderBuilder::cuboid(0.5, 0.5, 0.5)
                        .restitution(0.7)
                        .build(),
                ),
                parent_handle: None,
                handle: None,
            },
            antigen_cgmath::components::LinearVelocity3d(offset.clone().normalize() * 3.0),
            crate::renderers::cube::InstanceComponent::default(),
            BufferWriteInstances::new(
                None,
                Some(instance_buffer_entity),
                std::mem::size_of::<crate::renderers::cube::Instance>() as wgpu::BufferAddress
                    * (i + tetrahedron_count) as wgpu::BufferAddress,
            ),
            crate::renderers::cube::IndexedIndirectComponent::new(
                antigen_wgpu::DrawIndexedIndirect {
                    base_instance: i + tetrahedron_count,
                    ..cube_indirect
                },
            ),
            BufferWriteIndexedIndirect::new(
                None,
                Some(indirect_buffer_entity),
                std::mem::size_of::<antigen_wgpu::DrawIndexedIndirect>() as wgpu::BufferAddress
                    * (i + tetrahedron_count) as wgpu::BufferAddress,
            ),
        ));

        dir = cgmath::Matrix4::from_angle_y(cgmath::Deg(360.0 / tetrahedron_count as f32)) * dir;
    }

    // OBJ entity
    world.push((
        antigen_cgmath::components::Position3d::new(cgmath::Vector3::zero()),
        crate::components::SphereBounds(3.0),
        crate::renderers::cube::InstanceComponent::default(),
        BufferWriteInstances::new(
            None,
            Some(instance_buffer_entity),
            std::mem::size_of::<crate::renderers::cube::Instance>() as wgpu::BufferAddress
                * (tetrahedron_count + cube_count) as wgpu::BufferAddress,
        ),
        crate::renderers::cube::IndexedIndirectComponent::new(antigen_wgpu::DrawIndexedIndirect {
            base_instance: tetrahedron_count + cube_count,
            ..obj_indirect
        }),
        BufferWriteIndexedIndirect::new(
            None,
            Some(indirect_buffer_entity),
            std::mem::size_of::<antigen_wgpu::DrawIndexedIndirect>() as wgpu::BufferAddress
                * (tetrahedron_count + cube_count) as wgpu::BufferAddress,
        ),
    ));
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
