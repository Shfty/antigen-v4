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

use cgmath::{InnerSpace, One, Rotation3, Zero};
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

/// Opaque identifier used to establish an ordering for things like vertex / index buffer packing
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

/// Tag component identifying a BufferComponent as a vertex buffer
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct VertexBuffer;
legion_debugger::register_component!(VertexBuffer);

/// Tag component identifying a BufferComponent as an index buffer
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct IndexBuffer;
legion_debugger::register_component!(IndexBuffer);

/// Tag component identifying a BufferComponent as an instance buffer
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct InstanceBuffer;
legion_debugger::register_component!(InstanceBuffer);

/// Tag component identifying a BufferComponent as an indirect buffer
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct IndirectBuffer;
legion_debugger::register_component!(IndirectBuffer);

/// A list of mesh vertex positions
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

/// A list of mesh vertex normals
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

/// A list of mesh vertex UVs
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

/// A list of mesh vertex texture IDs
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

/// A list of mesh triangle indices
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

/// A list of mesh line indices
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeshLineIndices<T>(pub Vec<T>);

type MeshLineIndicesUsize = MeshLineIndices<usize>;

legion_debugger::register_component!(MeshLineIndicesUsize);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeshEntity(pub Entity);

legion_debugger::register_component!(MeshEntity);

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

fn assemble_mesh_entity(
    world: &mut World,
    entity: Entity,
    vertices: Vec<nalgebra::Vector3<f32>>,
    normals: Vec<nalgebra::Vector3<f32>>,
    uvs: Vec<nalgebra::Vector2<f32>>,
    texture_ids: Vec<i32>,
    triangle_indices: Vec<u16>,
    vertex_buffer_entity: Option<Entity>,
    index_buffer_entity: Option<Entity>,
) {
    // Cube mesh
    let mut entry = world.entry(entity).unwrap();
    entry.add_component(MeshId::next());
    entry.add_component(IndexedIndirectComponent::default());
    entry.add_component(MeshVertices::new(vertices));
    entry.add_component(MeshNormals::new(normals));
    entry.add_component(MeshUvs::new(uvs));
    entry.add_component(MeshTextureIds::new(texture_ids));
    entry.add_component(MeshTriangleIndices::new(triangle_indices));
    entry.add_component(BufferWriteMeshVertices::new(None, vertex_buffer_entity, 0));
    entry.add_component(BufferWriteMeshNormals::new(None, vertex_buffer_entity, 0));
    entry.add_component(BufferWriteMeshUvs::new(None, vertex_buffer_entity, 0));
    entry.add_component(BufferWriteMeshTextureIds::new(
        None,
        vertex_buffer_entity,
        0,
    ));
    entry.add_component(BufferWriteMeshTriangleIndices::new(
        None,
        index_buffer_entity,
        0,
    ));
}

pub fn align_vertex_data<T: Clone + Default>(mut data: Vec<T>) -> Vec<T> {
    let align = wgpu::COPY_BUFFER_ALIGNMENT;
    let unpadded_size = data.len() as u64;
    let padding = (align - unpadded_size % align) % align;
    let padded_size = unpadded_size + padding;
    data.resize(padded_size as usize, Default::default());
    data
}

pub fn load_obj(
    source: &[u8],
) -> (
    Vec<nalgebra::Vector3<f32>>,
    Vec<nalgebra::Vector3<f32>>,
    Vec<nalgebra::Vector2<f32>>,
    Vec<usize>,
) {
    let data = obj::ObjData::load_buf(source).unwrap();

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

    (
        align_vertex_data(obj_vertices),
        align_vertex_data(obj_normals),
        align_vertex_data(obj_uvs),
        align_vertex_data(obj_indices),
    )
}

#[derive(Debug, Copy, Clone)]
pub enum MeshMode {
    Normal,
    VisualizeDuplicates,
    VisualizeFaceFaceContainment,
    VisualizeBrushFaceContainment,
}

impl Default for MeshMode {
    fn default() -> Self {
        MeshMode::Normal
    }
}

pub struct MapData {
    geo_map: shambler::GeoMap,
    brush_centers: shambler::brush::BrushCenters,
    entity_centers: shambler::entity::EntityCenters,
    vertices: shambler::face::FaceVertices,
    normals: shambler::face::FaceNormals,
    uvs: shambler::face::FaceUvs,
    triangle_indices: shambler::face::FaceTriangleIndices,
    line_indices: shambler::face::FaceLineIndices,
    face_duplicates: shambler::face::FaceDuplicates,
    face_face_containment: shambler::face::FaceFaceContainment,
    brush_face_containment: shambler::brush::BrushFaceContainment,
}

pub fn build_map_data(map: shambler::shalrath::repr::Map) -> MapData {
    // Convert to flat structure
    let geo_map = shambler::GeoMap::from(map);

    // Create geo planes from brush planes
    let face_planes = shambler::face::FacePlanes::new(&geo_map.face_planes);

    // Create per-brush hulls from brush planes
    let brush_hulls = shambler::brush::BrushHulls::new(&geo_map.brush_faces, &face_planes);

    // Generate face vertices
    let face_vertices =
        shambler::face::FaceVertices::new(&geo_map.brush_faces, &face_planes, &brush_hulls);

    // Generate flat face normals
    let face_normals = shambler::face::FaceNormals::flat(&face_vertices, &face_planes);

    // Placeholder texture sizes
    let texture_sizes = shambler::texture::TextureSizes::new(
        &geo_map.textures,
        [("__TB_empty", (256, 256)), ("base/uv_test_512", (512, 512))]
            .iter()
            .copied()
            .collect(),
    );

    // Generate face UVs
    let face_uvs = shambler::face::FaceUvs::new(
        &geo_map.faces,
        &geo_map.textures,
        &geo_map.face_textures,
        &face_vertices,
        &face_planes,
        &geo_map.face_offsets,
        &geo_map.face_angles,
        &geo_map.face_scales,
        &texture_sizes,
    );

    // Find duplicate faces
    let face_duplicates =
        shambler::face::FaceDuplicates::new(&geo_map.faces, &face_planes, &face_vertices);

    // Generate centers
    let face_centers = shambler::face::FaceCenters::new(&face_vertices);

    let brush_centers = shambler::brush::BrushCenters::new(&geo_map.brush_faces, &face_centers);

    let entity_centers =
        shambler::entity::EntityCenters::new(&geo_map.entity_brushes, &brush_centers);

    // Generate per-plane CCW face indices
    let face_indices = shambler::face::FaceIndices::new(
        &geo_map.face_planes,
        &face_planes,
        &face_vertices,
        &face_centers,
        shambler::face::FaceWinding::Clockwise,
    );

    // Generate tangents
    let face_bases = shambler::face::FaceBases::new(
        &geo_map.faces,
        &face_planes,
        &geo_map.face_offsets,
        &geo_map.face_angles,
        &geo_map.face_scales,
    );

    // Generate line indices
    let line_indices = shambler::face::FaceLineIndices::new(&face_indices);

    // Calculate face-face containment
    let face_face_containment = shambler::face::FaceFaceContainment::new(
        &geo_map.faces,
        &face_planes,
        &face_bases,
        &face_vertices,
        &line_indices,
    );

    // Calculate brush-face containment
    let brush_face_containment = shambler::brush::BrushFaceContainment::new(
        &geo_map.brushes,
        &geo_map.faces,
        &geo_map.brush_faces,
        &brush_hulls,
        &face_vertices,
    );

    // Generate triangle indices
    let triangle_indices = shambler::face::FaceTriangleIndices::new(&face_indices);

    MapData {
        geo_map,
        brush_centers,
        entity_centers,
        vertices: face_vertices,
        normals: face_normals,
        uvs: face_uvs,
        triangle_indices,
        line_indices,
        face_duplicates,
        face_face_containment,
        brush_face_containment,
    }
}

pub fn build_map_mesh_single(
    map_data: &MapData,
    mesh_mode: MeshMode,
    inverse_scale_factor: f32,
) -> (
    Vec<nalgebra::Vector3<f32>>,
    Vec<nalgebra::Vector3<f32>>,
    Vec<nalgebra::Vector2<f32>>,
    Vec<usize>,
) {
    let MapData {
        geo_map,
        vertices: face_vertices,
        normals: face_normals,
        uvs: face_uvs,
        triangle_indices,
        line_indices,
        face_duplicates,
        face_face_containment,
        brush_face_containment,
        ..
    } = map_data;

    // Generate mesh
    let mut mesh_normals: Vec<shambler::Vector3> = Default::default();
    let mut mesh_vertices: Vec<shambler::Vector3> = Default::default();
    let mut mesh_uvs: Vec<shambler::Vector2> = Default::default();
    let mut mesh_line_indices: Vec<usize> = Default::default();
    let mut mesh_triangle_indices: Vec<usize> = Default::default();

    for face_id in &geo_map.faces {
        match mesh_mode {
            MeshMode::Normal => {
                if face_duplicates.contains(&face_id) {
                    continue;
                }

                if face_face_containment.is_contained(&face_id) {
                    continue;
                }

                if brush_face_containment.is_contained(&face_id) {
                    continue;
                }
            }
            MeshMode::VisualizeDuplicates => {
                if !face_duplicates.contains(&face_id) {
                    continue;
                }
            }
            MeshMode::VisualizeFaceFaceContainment => {
                if !face_face_containment.is_contained(&face_id) {
                    continue;
                }
            }
            MeshMode::VisualizeBrushFaceContainment => {
                if !brush_face_containment.is_contained(&face_id) {
                    continue;
                }
            }
        }

        let index_offset = mesh_vertices.len();
        let line_indices = &line_indices[&face_id];
        let triangle_indices = &triangle_indices[&face_id];

        mesh_vertices.extend(face_vertices.vertices(&face_id).unwrap().iter().map(|v| {
            nalgebra::vector![
                -v.x / inverse_scale_factor,
                v.z / inverse_scale_factor,
                v.y / inverse_scale_factor
            ]
        }));
        mesh_normals.extend(
            face_normals[&face_id]
                .iter()
                .map(|n| nalgebra::vector![-n.x, n.z, n.y]),
        );
        mesh_uvs.extend(face_uvs[&face_id].iter().copied());
        mesh_line_indices.extend(line_indices.iter().copied().map(|i| i + index_offset));
        mesh_triangle_indices.extend(triangle_indices.iter().copied().map(|i| i + index_offset));
    }

    (
        align_vertex_data(mesh_vertices),
        align_vertex_data(mesh_normals),
        align_vertex_data(mesh_uvs),
        align_vertex_data(mesh_triangle_indices),
    )
}

pub fn build_map_meshes_entities(
    map_data: &MapData,
    mesh_mode: MeshMode,
    inverse_scale_factor: f32,
) -> Vec<(
    nalgebra::Vector3<f32>,
    Vec<nalgebra::Vector3<f32>>,
    Vec<nalgebra::Vector3<f32>>,
    Vec<nalgebra::Vector2<f32>>,
    Vec<usize>,
)> {
    let MapData {
        geo_map,
        entity_centers,
        vertices: face_vertices,
        normals: face_normals,
        uvs: face_uvs,
        triangle_indices,
        line_indices,
        face_duplicates,
        face_face_containment,
        brush_face_containment,
        ..
    } = map_data;

    let mut entity_meshes = vec![];

    for entity_id in &geo_map.entities {
        let entity_brushes = if let Some(entity_brushes) = geo_map.entity_brushes.get(entity_id) {
            entity_brushes
        } else {
            continue;
        };

        let entity_faces = entity_brushes
            .iter()
            .map(|brush_id| &geo_map.brush_faces[brush_id])
            .flatten()
            .collect::<Vec<_>>();

        // Generate mesh
        let mesh_origin = *&entity_centers[&entity_id] / inverse_scale_factor;
        let mut mesh_normals: Vec<shambler::Vector3> = Default::default();
        let mut mesh_vertices: Vec<shambler::Vector3> = Default::default();
        let mut mesh_uvs: Vec<shambler::Vector2> = Default::default();
        let mut mesh_line_indices: Vec<usize> = Default::default();
        let mut mesh_triangle_indices: Vec<usize> = Default::default();

        for brush_id in entity_brushes {
            let brush_faces = &geo_map.brush_faces[&brush_id];
            for face_id in brush_faces {
                match mesh_mode {
                    MeshMode::Normal => {
                        if face_duplicates
                            .iter()
                            .filter(|id| entity_faces.contains(id))
                            .any(|id| id == face_id)
                        {
                            continue;
                        }

                        if face_face_containment
                            .iter()
                            .flat_map(|(_, value)| value)
                            .filter(|id| entity_faces.contains(id))
                            .any(|id| id == face_id)
                        {
                            continue;
                        }

                        if brush_face_containment
                            .iter()
                            .filter(|(id, _)| entity_brushes.contains(id))
                            .flat_map(|(_, value)| value)
                            .filter(|id| entity_faces.contains(id))
                            .any(|id| id == face_id)
                        {
                            continue;
                        }
                    }
                    MeshMode::VisualizeDuplicates => {
                        if !face_duplicates
                            .iter()
                            .filter(|id| entity_faces.contains(id))
                            .any(|id| id == face_id)
                        {
                            continue;
                        }
                    }
                    MeshMode::VisualizeFaceFaceContainment => {
                        if !face_face_containment
                            .iter()
                            .flat_map(|(_, value)| value)
                            .filter(|id| entity_faces.contains(id))
                            .any(|id| id == face_id)
                        {
                            continue;
                        }
                    }
                    MeshMode::VisualizeBrushFaceContainment => {
                        if !brush_face_containment
                            .iter()
                            .filter(|(id, _)| entity_brushes.contains(id))
                            .flat_map(|(_, value)| value)
                            .filter(|id| entity_faces.contains(id))
                            .any(|id| id == face_id)
                        {
                            continue;
                        }
                    }
                }

                let index_offset = mesh_vertices.len();
                let line_indices = &line_indices[&face_id];
                let triangle_indices = &triangle_indices[&face_id];

                mesh_vertices.extend(face_vertices.vertices(&face_id).unwrap().iter().map(|v| {
                    let x = (-v.x / inverse_scale_factor) - mesh_origin.x;
                    let y = (v.z / inverse_scale_factor) - mesh_origin.z;
                    let z = (v.y / inverse_scale_factor) - mesh_origin.y;
                    nalgebra::vector![x, y, z]
                }));
                mesh_normals.extend(
                    face_normals[&face_id]
                        .iter()
                        .map(|n| nalgebra::vector![-n.x, n.z, n.y]),
                );
                mesh_uvs.extend(face_uvs[&face_id].iter().copied());
                mesh_line_indices.extend(line_indices.iter().copied().map(|i| i + index_offset));
                mesh_triangle_indices
                    .extend(triangle_indices.iter().copied().map(|i| i + index_offset));
            }
        }

        entity_meshes.push((
            mesh_origin.xzy(),
            align_vertex_data(mesh_vertices),
            align_vertex_data(mesh_normals),
            align_vertex_data(mesh_uvs),
            align_vertex_data(mesh_triangle_indices),
        ));
    }

    entity_meshes
}

pub fn build_map_collision_brushes(
    map_data: &MapData,
    inverse_scale_factor: f32,
) -> Vec<(nalgebra::Vector3<f32>, Vec<nalgebra::Vector3<f32>>)> {
    let MapData {
        geo_map,
        brush_centers,
        vertices: face_vertices,
        ..
    } = map_data;

    let mut brush_meshes = vec![];

    for (brush_id, face_ids) in &geo_map.brush_faces {
        // Generate mesh
        let brush_origin = *&brush_centers[&brush_id] / inverse_scale_factor;
        let mut mesh_vertices: Vec<shambler::Vector3> = Default::default();

        for face_id in face_ids {
            mesh_vertices.extend(face_vertices.vertices(&face_id).unwrap().iter().map(|v| {
                let x = (-v.x / inverse_scale_factor) - brush_origin.x;
                let y = (v.z / inverse_scale_factor) - brush_origin.z;
                let z = (v.y / inverse_scale_factor) - brush_origin.y;
                nalgebra::vector![x, y, z]
            }));
        }

        brush_meshes.push((brush_origin.xzy(), align_vertex_data(mesh_vertices)));
    }

    brush_meshes
}

pub fn cube_renderer(world: &mut World, wgpu_manager: &WgpuManager) {
    let geo_inverse_scale_factor = 64.0;
    let map_inverse_scale_factor = 32.0;

    // Load meshes
    let tetrahedron_map =
        include_str!("../../../../../sif/crates/shalrath/test_data/tetrahedron.map");
    let tetrahedron_map = tetrahedron_map
        .parse::<shambler::shalrath::repr::Map>()
        .unwrap();
    let tetrahedron_map_data = build_map_data(tetrahedron_map);
    let (tetrahedron_vertices, tetrahedron_normals, tetrahedron_uvs, tetrahedron_triangle_indices) =
        build_map_mesh_single(
            &tetrahedron_map_data,
            MeshMode::Normal,
            geo_inverse_scale_factor,
        );

    let cube_map = include_str!("../../../../../sif/crates/shalrath/test_data/cube.map");
    let cube_map = cube_map.parse::<shambler::shalrath::repr::Map>().unwrap();
    let cube_map_data = build_map_data(cube_map);
    let (cube_vertices, cube_normals, cube_uvs, cube_triangle_indices) =
        build_map_mesh_single(&cube_map_data, MeshMode::Normal, geo_inverse_scale_factor);

    let abstract_test_map =
        include_str!("../../../../../sif/crates/shalrath/test_data/abstract-test.map",);
    let abstract_test_map = abstract_test_map
        .parse::<shambler::shalrath::repr::Map>()
        .unwrap();
    let abstract_test_map_data = build_map_data(abstract_test_map);
    let abstract_test_meshes = build_map_meshes_entities(
        &abstract_test_map_data,
        MeshMode::Normal,
        map_inverse_scale_factor,
    );
    let abstract_test_collision =
        build_map_collision_brushes(&abstract_test_map_data, map_inverse_scale_factor);

    let obj_source = include_bytes!("../renderers/skybox/models/marauder.obj");
    let (obj_vertices, obj_normals, obj_uvs, obj_indices) = load_obj(obj_source);

    // Calculate vertex count
    let tetrahedron_vertices_len = tetrahedron_vertices.len();
    let cube_vertices_len = cube_vertices.len();
    let abstract_test_vertices_len = abstract_test_meshes.iter().map(|mesh| mesh.1.len()).sum();
    let obj_vertices_len = obj_vertices.len();

    let vertex_count = [
        tetrahedron_vertices_len,
        cube_vertices_len,
        abstract_test_vertices_len,
        obj_vertices_len,
    ]
    .iter()
    .sum::<usize>();

    // Calculate index count
    let tetrahedron_indices_len = tetrahedron_triangle_indices.len();
    let cube_indices_len = cube_triangle_indices.len();
    let abstract_test_indices_len = abstract_test_meshes.iter().map(|mesh| mesh.4.len()).sum();
    let obj_indices_len = obj_indices.len();

    let index_count = [
        tetrahedron_indices_len,
        cube_indices_len,
        abstract_test_indices_len,
        obj_indices_len,
    ]
    .iter()
    .sum::<usize>();

    // Calculate instance count
    let tetrahedron_count = 16u32;
    let cube_count = 16u32;

    let instance_count = tetrahedron_count + cube_count + abstract_test_meshes.len() as u32 + 1;

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

    // Colliders
    for (i, (origin, vertices)) in abstract_test_collision.into_iter().enumerate() {
        world.push((
            Name::new(format!("Abstract Test Map Collider {}", i)),
            antigen_rapier3d::ColliderComponent {
                physics_sim_entity,
                parent_entity: None,
                pending_collider: Some(
                    ColliderBuilder::convex_hull(
                        &vertices
                            .iter()
                            .copied()
                            .map(|v| nalgebra::point![v.x, v.y, v.z])
                            .collect::<Vec<_>>()[..],
                    )
                    .unwrap()
                    .translation(antigen_rapier3d::rapier3d::prelude::vector![
                        origin.x, origin.y, origin.z
                    ])
                    .build(),
                ),
                parent_handle: None,
                handle: None,
            },
        ));
    }

    let tetrahedron_collider = ColliderBuilder::convex_hull(
        &tetrahedron_vertices
            .iter()
            .copied()
            .map(|v| nalgebra::point![v.x, v.y, v.z])
            .collect::<Vec<_>>()[..],
    )
    .unwrap()
    .restitution(0.7)
    .build();

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

    world.push((
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

    world.push((
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
        LookAt(cgmath::Point3::new(0.0, 2.0, 0.0)),
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

    let vertex_buffer_entity = world.push((VertexBuffer, BufferComponent::from(vertex_buffer)));

    let index_buffer_entity = world.push((IndexBuffer, BufferComponent::from(index_buffer)));

    let instance_buffer_entity =
        world.push((InstanceBuffer, BufferComponent::from(instance_buffer)));

    let indirect_buffer_entity =
        world.push((IndirectBuffer, BufferComponent::from(indirect_buffer)));

    // Meshes
    let tetrahedron_mesh_entity = world.push((Name::new("Tetrahedron Mesh"),));
    assemble_mesh_entity(
        world,
        tetrahedron_mesh_entity,
        tetrahedron_vertices,
        tetrahedron_normals,
        tetrahedron_uvs,
        std::iter::repeat(1i32)
            .take(tetrahedron_vertices_len)
            .collect(),
        tetrahedron_triangle_indices
            .iter()
            .copied()
            .map(|i| i as u16)
            .collect(),
        Some(vertex_buffer_entity),
        Some(index_buffer_entity),
    );

    let cube_mesh_entity = world.push((Name::new("Cube Mesh"),));
    assemble_mesh_entity(
        world,
        cube_mesh_entity,
        cube_vertices,
        cube_normals,
        cube_uvs,
        std::iter::repeat(0i32).take(cube_vertices_len).collect(),
        cube_triangle_indices
            .iter()
            .copied()
            .map(|i| i as u16)
            .collect(),
        Some(vertex_buffer_entity),
        Some(index_buffer_entity),
    );

    let abstract_test_mesh_entities = abstract_test_meshes
        .into_iter()
        .enumerate()
        .map(|(i, (origin, vertices, normals, uvs, indices))| {
            let abstract_test_mesh_entity =
                world.push((Name::new(format!("Abstract Test Mesh {}", i)),));
            assemble_mesh_entity(
                world,
                abstract_test_mesh_entity,
                vertices,
                normals,
                uvs,
                std::iter::repeat(1i32)
                    .take(abstract_test_vertices_len)
                    .collect(),
                indices.iter().copied().map(|i| i as u16).collect(),
                Some(vertex_buffer_entity),
                Some(index_buffer_entity),
            );
            (abstract_test_mesh_entity, origin)
        })
        .collect::<Vec<_>>();

    let obj_mesh_entity = world.push((Name::new("OBJ Mesh"),));
    assemble_mesh_entity(
        world,
        obj_mesh_entity,
        obj_vertices,
        obj_normals,
        obj_uvs,
        std::iter::repeat(0i32).take(obj_vertices_len).collect(),
        obj_indices.iter().copied().map(|i| i as u16).collect(),
        Some(vertex_buffer_entity),
        Some(index_buffer_entity),
    );

    // Tetrahedron entities
    let mut dir = cgmath::Vector4::unit_z();

    for i in 0..tetrahedron_count {
        let offset: cgmath::Vector3<f32> = dir.xyz();
        world.push((
            Name::new(format!("Tetrahedron #{}", i)),
            antigen_cgmath::components::Position3d::new(
                (cgmath::Vector3::unit_y() * 4.0) + (offset * 3.0),
            ),
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
    for (i, (entity, origin)) in abstract_test_mesh_entities.into_iter().enumerate() {
        world.push((
            Name::new(format!("Abstract Test Entity {}", i)),
            antigen_cgmath::components::Position3d::new(cgmath::Vector3::new(
                origin.x, origin.y, origin.z,
            )),
            antigen_cgmath::components::Orientation::default(),
            crate::components::SphereBounds(3.0),
            crate::renderers::cube::InstanceComponent::default(),
            BufferWritePosition::new(None, Some(instance_buffer_entity), 0),
            BufferWriteOrientation::new(None, Some(instance_buffer_entity), 0),
            BufferWriteInstances::new(None, Some(instance_buffer_entity), 0),
            MeshEntity(entity),
            IndexedIndirectComponent::new(Default::default()),
            BufferWriteIndexedIndirect::new(None, Some(indirect_buffer_entity), 0),
        ));
    }

    // OBJ entity
    world.push((
        Name::new("OBJ"),
        antigen_cgmath::components::Position3d::new(cgmath::Vector3::new(0.0, 2.5, 3.5)),
        antigen_cgmath::components::Orientation::new(cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(180.0))),
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
