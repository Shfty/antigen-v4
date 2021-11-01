use legion::Entity;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PerspectiveProjection {
    pub fov_entity: Option<Entity>,
    pub aspect_ratio_entity: Option<Entity>,
    pub near_plane_entity: Option<Entity>,
    pub far_plane_entity: Option<Entity>,
}

legion_debugger::register_component!(PerspectiveProjection);
