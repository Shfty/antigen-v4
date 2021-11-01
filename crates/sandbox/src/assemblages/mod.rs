use std::num::NonZeroU32;

use antigen_cgmath::components::{
    AspectRatio, EyePosition, FarPlane, FieldOfView, LookAt, NearPlane, PerspectiveProjection,
    ProjectionMatrix, UpVector, ViewProjectionMatrix,
};
use antigen_components::{Image, ImageComponent, Name};
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
use on_change::OnChange;
use wgpu::BufferAddress;

use crate::renderers::{
    boids::BoidsRenderer,
    bunnymark::BunnymarkRenderer,
    conservative_raster::ConservativeRasterRenderer,
    cube::{
        BufferWriteIndexedIndirect, BufferWriteInstances, BufferWriteMeshNormals,
        BufferWriteMeshTextureIds, BufferWriteMeshTriangleIndices, BufferWriteMeshUvs,
        BufferWriteMeshVertices, BufferWriteOrientation, BufferWritePosition, CubeRenderer,
        IndexedIndirectComponent, Uniforms, UniformsComponent,
    },
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
type TextureWriteImage = TextureWrite<ImageComponent, Image>;

legion_debugger::register_component!(BufferWriteViewProjectionMatrix);
legion_debugger::register_component!(BufferWriteUniforms);
legion_debugger::register_component!(TextureWriteImage);

static MESH_ID_HEAD: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct MeshId(usize);

impl MeshId {
    pub fn next() -> Self {
        MeshId(MESH_ID_HEAD.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

legion_debugger::register_component!(MeshId);

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct VertexBuffer;
legion_debugger::register_component!(VertexBuffer);

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct IndexBuffer;
legion_debugger::register_component!(IndexBuffer);

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct InstanceBuffer;
legion_debugger::register_component!(InstanceBuffer);

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct IndirectBuffer;
legion_debugger::register_component!(IndirectBuffer);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MeshVertices<T>(pub OnChange<Vec<T>>);

pub type MeshVerticesVector3 = MeshVertices<nalgebra::Vector3<f32>>;

impl<T> MeshVertices<T> {
    pub fn new(t: Vec<T>) -> Self {
        MeshVertices(OnChange::new_dirty(t))
    }
}

impl<T> std::ops::Deref for MeshVertices<T> {
    type Target = OnChange<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> on_change::OnChangeTrait<Vec<T>> for MeshVertices<T> {
    fn take_change(&self) -> Option<&Vec<T>> {
        self.0.take_change()
    }
}

legion_debugger::register_component!(MeshVerticesVector3);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MeshNormals<T>(pub OnChange<Vec<T>>);

impl<T> MeshNormals<T> {
    pub fn new(t: Vec<T>) -> Self {
        MeshNormals(OnChange::new_dirty(t))
    }
}

impl<T> std::ops::Deref for MeshNormals<T> {
    type Target = OnChange<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> on_change::OnChangeTrait<Vec<T>> for MeshNormals<T> {
    fn take_change(&self) -> Option<&Vec<T>> {
        self.0.take_change()
    }
}

pub type MeshNormalsVector3 = MeshNormals<nalgebra::Vector3<f32>>;

legion_debugger::register_component!(MeshNormalsVector3);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MeshUvs<T>(pub OnChange<Vec<T>>);

impl<T> MeshUvs<T> {
    pub fn new(t: Vec<T>) -> Self {
        MeshUvs(OnChange::new_dirty(t))
    }
}

impl<T> std::ops::Deref for MeshUvs<T> {
    type Target = OnChange<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> on_change::OnChangeTrait<Vec<T>> for MeshUvs<T> {
    fn take_change(&self) -> Option<&Vec<T>> {
        self.0.take_change()
    }
}

pub type MeshUvsVector2 = MeshUvs<nalgebra::Vector2<f32>>;

legion_debugger::register_component!(MeshUvsVector2);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MeshTextureIds<T>(pub OnChange<Vec<T>>);

impl<T> MeshTextureIds<T> {
    pub fn new(t: Vec<T>) -> Self {
        MeshTextureIds(OnChange::new_dirty(t))
    }
}

impl<T> std::ops::Deref for MeshTextureIds<T> {
    type Target = OnChange<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> on_change::OnChangeTrait<Vec<T>> for MeshTextureIds<T> {
    fn take_change(&self) -> Option<&Vec<T>> {
        self.0.take_change()
    }
}

pub type MeshTextureIdsI32 = MeshTextureIds<i32>;

legion_debugger::register_component!(MeshTextureIdsI32);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MeshTriangleIndices<T>(pub OnChange<Vec<T>>);

impl<T> MeshTriangleIndices<T> {
    pub fn new(t: Vec<T>) -> Self {
        MeshTriangleIndices(OnChange::new_dirty(t))
    }
}

impl<T> std::ops::Deref for MeshTriangleIndices<T> {
    type Target = OnChange<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> on_change::OnChangeTrait<Vec<T>> for MeshTriangleIndices<T> {
    fn take_change(&self) -> Option<&Vec<T>> {
        self.0.take_change()
    }
}

pub type MeshTriangleIndicesU16 = MeshTriangleIndices<u16>;

legion_debugger::register_component!(MeshTriangleIndicesU16);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeshLineIndices<T>(pub Vec<T>);

type MeshLineIndicesUsize = MeshLineIndices<usize>;

legion_debugger::register_component!(MeshLineIndicesUsize);

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

/// The list of element offsets into a given buffer
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BufferOffsetsComponent(pub Vec<usize>);

impl Default for BufferOffsetsComponent {
    fn default() -> Self {
        BufferOffsetsComponent(Default::default())
    }
}

legion_debugger::register_component!(BufferOffsetsComponent);

/// The list of element lengths within a given buffer
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BufferLengthsComponent(pub Vec<usize>);

impl Default for BufferLengthsComponent {
    fn default() -> Self {
        BufferLengthsComponent(Default::default())
    }
}

legion_debugger::register_component!(BufferLengthsComponent);

/// The target vertex buffer to store mesh data into
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VertexBufferEntity(pub Entity);

legion_debugger::register_component!(VertexBufferEntity);

/// The target index buffer to store mesh data into
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexBufferEntity(pub Entity);

legion_debugger::register_component!(IndexBufferEntity);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeshEntity(pub Entity);

legion_debugger::register_component!(MeshEntity);

pub fn cube_renderer(world: &mut World, wgpu_manager: &WgpuManager) {
    let tetrahedron_count = 16u32;
    let cube_count = 16u32;

    // Load obj
    let source = include_bytes!("../renderers/skybox/models/marauder.obj");
    let data = obj::ObjData::load_buf(&source[..]).unwrap();

    let mut obj_vertices = Vec::new();
    let mut obj_normals = Vec::new();
    let mut obj_uvs = Vec::new();
    let mut obj_indices = Vec::new();

    for object in data.objects {
        for group in object.groups {
            for poly in group.polys {
                for end_index in 2..poly.0.len() {
                    for &index in &[0, end_index - 1, end_index] {
                        let obj::IndexTuple(position_id, texture_id, normal_id) = poly.0[index];

                        let pos = data.position[position_id];
                        obj_vertices.push(nalgebra::vector![
                            pos[0] / 40.0,
                            pos[1] / 40.0,
                            pos[2] / 40.0
                        ]);

                        let normal = data.normal[normal_id.unwrap()];
                        obj_normals.push(nalgebra::vector![normal[0], normal[1], normal[2]]);

                        let uv = data.texture[texture_id.unwrap()];
                        obj_uvs.push(nalgebra::vector![uv[0], uv[1]]);

                        obj_indices.push(obj_indices.len());
                    }
                }
            }
        }
    }

    let align = wgpu::COPY_BUFFER_ALIGNMENT;
    let unpadded_size = obj_vertices.len() as u64;
    let padding = (align - unpadded_size % align) % align;
    let padded_size = unpadded_size + padding;

    obj_vertices.resize(padded_size as usize, Default::default());

    obj_normals.resize(padded_size as usize, Default::default());

    obj_uvs.resize(padded_size as usize, Default::default());

    let align = wgpu::COPY_BUFFER_ALIGNMENT;
    let unpadded_size = obj_indices.len() as u64;
    let padding = (align - unpadded_size % align) % align;
    let padded_size = unpadded_size + padding;

    obj_indices.resize(padded_size as usize, Default::default());

    let cube_map = include_str!("../../../../../sif/crates/shalrath/test_data/cube.map");
    let cube_map = cube_map.parse::<shambler::shalrath::repr::Map>().unwrap();

    let shambler::Mesh {
        vertices: cube_vertices,
        normals: cube_normals,
        uvs: cube_uvs,
        triangle_indices: cube_triangle_indices,
        ..
    } = shambler::map_mesh(cube_map);

    let tetrahedron_map =
        include_str!("../../../../../sif/crates/shalrath/test_data/tetrahedron.map");
    let tetrahedron_map = tetrahedron_map
        .parse::<shambler::shalrath::repr::Map>()
        .unwrap();

    let shambler::Mesh {
        vertices: tetrahedron_vertices,
        normals: tetrahedron_normals,
        uvs: tetrahedron_uvs,
        triangle_indices: tetrahedron_triangle_indices,
        ..
    } = shambler::map_mesh(tetrahedron_map);

    let abstract_test_map =
        include_str!("../../../../../sif/crates/shalrath/test_data/abstract-test.map");
    let abstract_test_map = abstract_test_map
        .parse::<shambler::shalrath::repr::Map>()
        .unwrap();

    let shambler::Mesh {
        vertices: abstract_test_vertices,
        normals: abstract_test_normals,
        uvs: abstract_test_uvs,
        triangle_indices: abstract_test_triangle_indices,
        ..
    } = shambler::map_mesh(abstract_test_map);

    let tetrahedron_vertices_len = tetrahedron_vertices.len();
    let tetrahedron_indices_len = tetrahedron_triangle_indices.len();

    let cube_vertices_len = cube_vertices.len();
    let cube_indices_len = cube_triangle_indices.len();

    let abstract_test_vertices_len = abstract_test_vertices.len();
    let abstract_test_indices_len = abstract_test_triangle_indices.len();

    let obj_vertices_len = obj_vertices.len();
    let obj_indices_len = obj_indices.len();

    let vertex_count = tetrahedron_vertices_len
        + cube_vertices_len
        + abstract_test_vertices_len
        + obj_vertices_len;

    let index_count =
        tetrahedron_indices_len + cube_indices_len + abstract_test_indices_len + obj_indices_len;

    let instance_count = tetrahedron_count + cube_count + 1 + 1;

    // Physics simulation
    let physics_sim_entity = world.push((
        Name::new("Physics Simulation"),
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

    let vertex_buffer = cube_renderer.take_vertex_buffer_handle();
    let index_buffer = cube_renderer.take_index_buffer_handle();
    let instance_buffer = cube_renderer.take_instance_buffer_handle();
    let indirect_buffer = cube_renderer.take_indirect_buffer_handle();

    // Texture array
    let texture_entity = world.push((
        Name::new("Texture Array"),
        TextureComponent::from(cube_renderer.take_texture_handle()),
    ));

    let mandelbrot_texture_entity = world.push((
        Name::new("Mandelbrot Texture"),
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
        Name::new("Inverse Mandelbrot Texture"),
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
        Name::new("Cube Renderer"),
        WindowComponent::default(),
        WindowTitle::from("Cube"),
        RedrawModeComponent::from(RedrawMode::MainEventsClearedLoop),
        SurfaceComponent::default(),
        AspectRatio::default(),
        cube_pass_component,
    ));

    let camera_entity = world.push((
        EyePosition(cgmath::Point3::new(0.0, 0.0, 5.0)),
        LookAt::default(),
        UpVector::default(),
        antigen_cgmath::components::Orientation::default(),
        PerspectiveProjection {
            aspect_ratio_entity: Some(cube_renderer_entity),
            ..Default::default()
        },
        FieldOfView::default(),
        NearPlane::default(),
        FarPlane(200.0),
        ProjectionMatrix::default(),
        uniform_buffer_component,
        UniformsComponent::new(cgmath::Matrix4::one()),
        BufferWriteUniforms::new(None, None, 0),
    ));

    let vertex_buffer_entity = world.push((
        Name::new("Vertex Buffer"),
        VertexBuffer,
        BufferComponent::from(vertex_buffer),
        BufferOffsetsComponent::default(),
        BufferLengthsComponent::default(),
    ));

    let index_buffer_entity = world.push((
        Name::new("Index Buffer"),
        IndexBuffer,
        BufferComponent::from(index_buffer),
        BufferOffsetsComponent::default(),
        BufferLengthsComponent::default(),
    ));

    let instance_buffer_entity = world.push((
        Name::new("Instance Buffer"),
        InstanceBuffer,
        BufferComponent::from(instance_buffer),
        BufferOffsetsComponent::default(),
        BufferLengthsComponent::default(),
    ));

    let indirect_buffer_entity = world.push((
        Name::new("Indirect Buffer"),
        IndirectBuffer,
        BufferComponent::from(indirect_buffer),
        BufferOffsetsComponent::default(),
        BufferLengthsComponent::default(),
    ));

    // Tetrahedron mesh
    let tetrahedron_mesh_id = MeshId::next();

    let tetrahedron_mesh_entity = world.push((
        Name::new("Tetrahedron Mesh"),
        tetrahedron_mesh_id,
        IndexedIndirectComponent::new(Default::default()),
        MeshVertices::new(tetrahedron_vertices.clone()),
        MeshNormals::new(tetrahedron_normals.clone()),
        MeshUvs::new(tetrahedron_uvs.clone()),
        MeshTextureIds::new(
            std::iter::repeat(1i32)
                .take(tetrahedron_vertices.len())
                .collect(),
        ),
        MeshTriangleIndices::new(
            tetrahedron_triangle_indices
                .iter()
                .copied()
                .map(|i| i as u16)
                .collect(),
        ),
        BufferWriteMeshVertices::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshNormals::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshUvs::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshTextureIds::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshTriangleIndices::new(None, Some(index_buffer_entity), 0),
    ));

    // Cube mesh
    let cube_mesh_id = MeshId::next();

    let cube_mesh_entity = world.push((
        Name::new("Cube Mesh"),
        cube_mesh_id,
        IndexedIndirectComponent::new(Default::default()),
        MeshVertices::new(cube_vertices.clone()),
        MeshNormals::new(cube_normals.clone()),
        MeshUvs::new(cube_uvs.clone()),
        MeshTextureIds::new(std::iter::repeat(0i32).take(cube_vertices.len()).collect()),
        MeshTriangleIndices::new(
            cube_triangle_indices
                .iter()
                .copied()
                .map(|i| i as u16)
                .collect(),
        ),
        BufferWriteMeshVertices::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshNormals::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshUvs::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshTextureIds::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshTriangleIndices::new(None, Some(index_buffer_entity), 0),
    ));

    // Abstract test mesh
    let abstract_test_mesh_id = MeshId::next();

    let abstract_test_mesh_entity = world.push((
        Name::new("Abstract Test Mesh"),
        abstract_test_mesh_id,
        IndexedIndirectComponent::new(Default::default()),
        MeshVertices::new(abstract_test_vertices.clone()),
        MeshNormals::new(abstract_test_normals.clone()),
        MeshUvs::new(abstract_test_uvs.clone()),
        MeshTextureIds::new(
            std::iter::repeat(1i32)
                .take(abstract_test_vertices.len())
                .collect(),
        ),
        MeshTriangleIndices::new(
            abstract_test_triangle_indices
                .iter()
                .copied()
                .map(|i| i as u16)
                .collect(),
        ),
        BufferWriteMeshVertices::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshNormals::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshUvs::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshTextureIds::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshTriangleIndices::new(None, Some(index_buffer_entity), 0),
    ));

    // OBJ mesh
    let obj_mesh_id = MeshId::next();

    let obj_mesh_entity = world.push((
        Name::new("OBJ Mesh"),
        obj_mesh_id,
        IndexedIndirectComponent::new(Default::default()),
        MeshVertices::new(obj_vertices.clone()),
        MeshNormals::new(obj_normals.clone()),
        MeshUvs::new(obj_uvs.clone()),
        MeshTextureIds::new(std::iter::repeat(0i32).take(obj_vertices.len()).collect()),
        MeshTriangleIndices::new(obj_indices.iter().copied().map(|i| i as u16).collect()),
        BufferWriteMeshVertices::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshNormals::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshUvs::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshTextureIds::new(None, Some(vertex_buffer_entity), 0),
        BufferWriteMeshTriangleIndices::new(None, Some(index_buffer_entity), 0),
    ));

    // Floor entity
    let floor_entity = world.push((
        Name::new("Floor Collision"),
        antigen_rapier3d::ColliderComponent {
            physics_sim_entity,
            parent_entity: None,
            pending_collider: Some(
                ColliderBuilder::cuboid(100.0, 0.1, 100.0)
                    .translation(antigen_rapier3d::rapier3d::prelude::vector![0.0, -5.0, 0.0])
                    .build(),
            ),
            parent_handle: None,
            handle: None,
        },
    ));

    // Tetrahedron entities
    let mut dir = cgmath::Vector4::unit_z();

    let tetrahedron_collider = ColliderBuilder::convex_hull(
        &tetrahedron_vertices
            .iter()
            .copied()
            .map(|v| nalgebra::Point3::new(v.x, v.y, v.z))
            .collect::<Vec<_>>()[..],
    )
    .unwrap()
    .restitution(0.7)
    .build();

    for i in 0..tetrahedron_count {
        let offset: cgmath::Vector3<f32> = dir.xyz();
        world.push((
            Name::new(format!("Tetrahedron #{}", i)),
            antigen_cgmath::components::Position3d::new(offset * 3.0),
            antigen_cgmath::components::Orientation::default(),
            crate::components::SphereBounds(1.0),
            // Instance data
            crate::renderers::cube::InstanceComponent::default(),
            BufferWritePosition::new(None, Some(instance_buffer_entity), 0),
            BufferWriteOrientation::new(None, Some(instance_buffer_entity), 0),
            BufferWriteInstances::new(None, Some(instance_buffer_entity), 0),
            // Indirect data
            MeshEntity(tetrahedron_mesh_entity),
            IndexedIndirectComponent::new(Default::default()),
            BufferWriteIndexedIndirect::new(None, Some(indirect_buffer_entity), 0),
            // Collision
            antigen_rapier3d::RigidBodyComponent {
                physics_sim_entity,
                pending_rigid_body: Some(RigidBodyBuilder::new_dynamic().build()),
                handle: None,
            },
            antigen_rapier3d::ColliderComponent {
                physics_sim_entity,
                parent_entity: None,
                pending_collider: Some(tetrahedron_collider.clone()),
                parent_handle: None,
                handle: None,
            },
        ));

        dir = cgmath::Matrix4::from_angle_x(cgmath::Deg(360.0 / tetrahedron_count as f32)) * dir;
    }

    // Cube entities
    let cube_collider = ColliderBuilder::convex_hull(
        &cube_vertices
            .iter()
            .copied()
            .map(|v| nalgebra::point!(v.x, v.y, v.z))
            .collect::<Vec<_>>()[..],
    )
    .unwrap()
    .restitution(0.7)
    .build();

    let mut dir = cgmath::Vector4::unit_z();
    for i in 0..cube_count {
        let offset: cgmath::Vector3<f32> = dir.xyz();
        world.push((
            Name::new(format!("Cube #{}", i)),
            antigen_cgmath::components::Position3d::new(offset * 3.0),
            antigen_cgmath::components::Orientation::default(),
            crate::components::SphereBounds(1.0),
            antigen_rapier3d::RigidBodyComponent {
                physics_sim_entity,
                pending_rigid_body: Some(RigidBodyBuilder::new_kinematic_velocity_based().build()),
                handle: None,
            },
            antigen_rapier3d::ColliderComponent {
                physics_sim_entity,
                parent_entity: None,
                pending_collider: Some(cube_collider.clone()),
                parent_handle: None,
                handle: None,
            },
            antigen_cgmath::components::LinearVelocity3d(offset.clone().normalize() * 3.0),
            crate::renderers::cube::InstanceComponent::default(),
            BufferWritePosition::new(None, Some(instance_buffer_entity), 0),
            BufferWriteOrientation::new(None, Some(instance_buffer_entity), 0),
            BufferWriteInstances::new(None, Some(instance_buffer_entity), 0),
            MeshEntity(cube_mesh_entity),
            IndexedIndirectComponent::new(Default::default()),
            BufferWriteIndexedIndirect::new(None, Some(indirect_buffer_entity), 0),
        ));

        dir = cgmath::Matrix4::from_angle_y(cgmath::Deg(360.0 / tetrahedron_count as f32)) * dir;
    }

    // Abstract test entity
    world.push((
        Name::new("Abstract Test"),
        antigen_cgmath::components::Position3d::new(cgmath::Vector3::new(0.0, -2.5, 0.0)),
        antigen_cgmath::components::Orientation::default(),
        crate::components::SphereBounds(3.0),
        crate::renderers::cube::InstanceComponent::default(),
        BufferWritePosition::new(None, Some(instance_buffer_entity), 0),
        BufferWriteOrientation::new(None, Some(instance_buffer_entity), 0),
        BufferWriteInstances::new(None, Some(instance_buffer_entity), 0),
        MeshEntity(abstract_test_mesh_entity),
        IndexedIndirectComponent::new(Default::default()),
        BufferWriteIndexedIndirect::new(None, Some(indirect_buffer_entity), 0),
    ));

    // OBJ entity
    world.push((
        Name::new("OBJ"),
        antigen_cgmath::components::Position3d::new(cgmath::Vector3::zero()),
        antigen_cgmath::components::Orientation::default(),
        crate::components::SphereBounds(3.0),
        crate::renderers::cube::InstanceComponent::default(),
        BufferWritePosition::new(None, Some(instance_buffer_entity), 0),
        BufferWriteOrientation::new(None, Some(instance_buffer_entity), 0),
        BufferWriteInstances::new(None, Some(instance_buffer_entity), 0),
        MeshEntity(obj_mesh_entity),
        IndexedIndirectComponent::new(Default::default()),
        BufferWriteIndexedIndirect::new(None, Some(indirect_buffer_entity), 0),
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
