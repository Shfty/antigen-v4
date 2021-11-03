use legion::{query::IntoQuery, systems::Builder, world::SubWorld, Entity};
pub use rapier3d;

use antigen_cgmath::cgmath::InnerSpace;
use rapier3d::prelude::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
pub struct Gravity(pub Vector<Real>);

impl std::ops::Deref for Gravity {
    type Target = Vector<Real>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Gravity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl serde::Serialize for Gravity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct("Gravity", &(self.0.x, self.0.y, self.0.z))
    }
}

impl<'de> serde::Deserialize<'de> for Gravity {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

#[derive(Default)]
pub struct PhysicsPipelineComponent(pub PhysicsPipeline);

impl serde::Serialize for PhysicsPipelineComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct("PhysicsPipelineComponent", &())
    }
}

impl<'de> serde::Deserialize<'de> for PhysicsPipelineComponent {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl std::ops::Deref for PhysicsPipelineComponent {
    type Target = PhysicsPipeline;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PhysicsPipelineComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(Gravity);
legion_debugger::register_component!(RigidBodySet);
legion_debugger::register_component!(ColliderSet);
legion_debugger::register_component!(IntegrationParameters);
legion_debugger::register_component!(PhysicsPipelineComponent);
legion_debugger::register_component!(IslandManager);
legion_debugger::register_component!(BroadPhase);
legion_debugger::register_component!(NarrowPhase);
legion_debugger::register_component!(JointSet);
legion_debugger::register_component!(CCDSolver);

#[legion::system(par_for_each)]
pub fn rapier3d_tick(
    gravity: &Gravity,
    rigid_body_set: &mut RigidBodySet,
    collider_set: &mut ColliderSet,
    integration_parameters: &IntegrationParameters,
    physics_pipeline: &mut PhysicsPipelineComponent,
    island_manager: &mut IslandManager,
    broad_phase: &mut BroadPhase,
    narrow_phase: &mut NarrowPhase,
    joint_set: &mut JointSet,
    ccd_solver: &mut CCDSolver,
) {
    physics_pipeline.step(
        &*gravity,
        integration_parameters,
        island_manager,
        broad_phase,
        narrow_phase,
        rigid_body_set,
        collider_set,
        joint_set,
        ccd_solver,
        &(),
        &(),
    )
}

#[legion::system(par_for_each)]
pub fn rapier3d_tick_hooks_events<
    PH: PhysicsHooks<RigidBodySet, ColliderSet> + 'static,
    EH: EventHandler + 'static,
>(
    gravity: &Gravity,
    rigid_body_set: &mut RigidBodySet,
    collider_set: &mut ColliderSet,
    integration_parameters: &IntegrationParameters,
    physics_pipeline: &mut PhysicsPipelineComponent,
    island_manager: &mut IslandManager,
    broad_phase: &mut BroadPhase,
    narrow_phase: &mut NarrowPhase,
    joint_set: &mut JointSet,
    ccd_solver: &mut CCDSolver,
    physics_hooks: &PH,
    event_handler: &EH,
) {
    physics_pipeline.step(
        &*gravity,
        integration_parameters,
        island_manager,
        broad_phase,
        narrow_phase,
        rigid_body_set,
        collider_set,
        joint_set,
        ccd_solver,
        physics_hooks,
        event_handler,
    )
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RigidBodyComponent {
    pub physics_sim_entity: Entity,
    pub pending_rigid_body: Option<RigidBody>,
    pub handle: Option<RigidBodyHandle>,
}

legion_debugger::register_component!(RigidBodyComponent);

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ColliderComponent {
    pub physics_sim_entity: Entity,
    pub parent_entity: Option<Entity>,
    pub pending_collider: Option<Collider>,
    pub handle: Option<ColliderHandle>,
    pub parent_handle: Option<RigidBodyHandle>,
}

legion_debugger::register_component!(ColliderComponent);

#[legion::system(for_each)]
#[read_component(RigidBodyComponent)]
#[write_component(RigidBodySet)]
pub fn create_rigid_bodies(
    world: &mut SubWorld,
    rigid_body: &mut RigidBodyComponent,
    position: Option<&mut antigen_cgmath::components::Position3d>,
    _: Option<&mut antigen_cgmath::components::Orientation>,
) {
    if let Some(mut pending) = rigid_body.pending_rigid_body.take() {
        if let Ok(rigid_body_set) =
            <&mut RigidBodySet>::query().get_mut(world, rigid_body.physics_sim_entity)
        {
            if let Some(position) = position {
                let position = position.get();
                pending.set_translation(vector![position.x, position.y, position.z], true);
            }

            /* TODO: Orientation
            if let Some(orientation) = orientation {
                pending.set_rotation(
                    nalgebra::Quaternion::new(
                        orientation.s,
                        orientation.v.x,
                        orientation.v.y,
                        orientation.v.z,
                    ),
                    true,
                )
            }
            */

            rigid_body.handle = Some(rigid_body_set.insert(pending));
        }
    }
}

#[legion::system(for_each)]
#[read_component(RigidBodyComponent)]
#[write_component(ColliderSet)]
#[write_component(RigidBodySet)]
pub fn create_colliders(
    world: &mut SubWorld,
    entity: &Entity,
    collider_component: &mut ColliderComponent,
) {
    if let Some(collider) = collider_component.pending_collider.take() {
        if let Some(parent) = collider_component.parent_entity {
            // Explicit parent case
            if let Ok(rigid_body) = <&RigidBodyComponent>::query().get(world, parent) {
                if let Some(handle) = rigid_body.handle {
                    if let Ok((rigid_body_set, collider_set)) =
                        <(&mut RigidBodySet, &mut ColliderSet)>::query()
                            .get_mut(world, collider_component.physics_sim_entity)
                    {
                        collider_component.handle =
                            Some(collider_set.insert_with_parent(collider, handle, rigid_body_set));
                    }
                }
            }
        } else {
            if let Ok(rigid_body) = <&RigidBodyComponent>::query().get(world, *entity) {
                // Implement self-parent case
                if let Some(handle) = rigid_body.handle {
                    if let Ok((rigid_body_set, collider_set)) =
                        <(&mut RigidBodySet, &mut ColliderSet)>::query()
                            .get_mut(world, collider_component.physics_sim_entity)
                    {
                        collider_component.handle =
                            Some(collider_set.insert_with_parent(collider, handle, rigid_body_set));
                    }
                }
            } else {
                // No parent case
                if let Ok(collider_set) = <&mut ColliderSet>::query()
                    .get_mut(world, collider_component.physics_sim_entity)
                {
                    collider_component.handle = Some(collider_set.insert(collider));
                }
            }
        }
    }
}

#[legion::system(par_for_each)]
#[read_component(RigidBodySet)]
pub fn rigid_body_readback(
    world: &SubWorld,
    rigid_body: &RigidBodyComponent,
    position: Option<&mut antigen_cgmath::components::Position3d>,
    orientation: Option<&mut antigen_cgmath::components::Orientation>,
    velocity: Option<&mut antigen_cgmath::components::LinearVelocity3d>,
) {
    if let Some(handle) = rigid_body.handle {
        if let Ok(rigid_body_set) =
            <&RigidBodySet>::query().get(world, rigid_body.physics_sim_entity)
        {
            let body = &rigid_body_set[handle];
            match body.body_type() {
                RigidBodyType::Dynamic => {
                    if let Some(position) = position {
                        let pos = body.translation();
                        position
                            .set_checked(antigen_cgmath::cgmath::Vector3::new(pos.x, pos.y, pos.z));
                    }

                    if let Some(orientation) = orientation {
                        let quat = body.rotation();
                        orientation.set_checked(antigen_cgmath::cgmath::Quaternion::new(
                            quat.w, quat.i, quat.j, quat.k,
                        ));
                    }

                    if let Some(velocity) = velocity {
                        let vel = body.linvel();
                        **velocity = antigen_cgmath::cgmath::Vector3::new(vel.x, vel.y, vel.z);
                    }
                }
                RigidBodyType::KinematicPositionBased => {
                    if let Some(velocity) = velocity {
                        let vel = body.linvel();
                        **velocity = antigen_cgmath::cgmath::Vector3::new(vel.x, vel.y, vel.z);
                    }
                }
                RigidBodyType::KinematicVelocityBased => {
                    if let Some(position) = position {
                        let pos = body.translation();
                        position
                            .set_checked(antigen_cgmath::cgmath::Vector3::new(pos.x, pos.y, pos.z));
                    }

                    if let Some(orientation) = orientation {
                        let quat = body.rotation();
                        orientation.set_checked(antigen_cgmath::cgmath::Quaternion::new(
                            quat.w, quat.i, quat.j, quat.k,
                        ));
                    }
                }
                _ => (),
            }
        }
    }
}

#[legion::system(for_each)]
#[write_component(RigidBodySet)]
pub fn rigid_body_kinematic_position(
    world: &mut SubWorld,
    rigid_body: &RigidBodyComponent,
    position: Option<&mut antigen_cgmath::components::Position3d>,
    _: Option<&mut antigen_cgmath::components::Orientation>,
) {
    if let Some(handle) = rigid_body.handle {
        if let Ok(rigid_body_set) =
            <&mut RigidBodySet>::query().get_mut(world, rigid_body.physics_sim_entity)
        {
            let body = &mut rigid_body_set[handle];
            if let RigidBodyType::KinematicPositionBased = body.body_type() {
                if let Some(position) = position {
                    let position = position.get();
                    body.set_next_kinematic_translation(vector![
                        position.x, position.y, position.z
                    ]);
                }

                /* TODO: Orientation
                if let Some(orientation) = orientation {
                    body.set_next_kinematic_rotation(nalgebra::Quaternion::new(
                        orientation.s,
                        orientation.v.x,
                        orientation.v.y,
                        orientation.v.z,
                    ));
                }
                */
            }
        }
    }
}

#[legion::system(for_each)]
#[write_component(RigidBodySet)]
pub fn rigid_body_kinematic_velocity(
    world: &mut SubWorld,
    rigid_body: &RigidBodyComponent,
    velocity: Option<&mut antigen_cgmath::components::LinearVelocity3d>,
) {
    if let Some(handle) = rigid_body.handle {
        if let Ok(rigid_body_set) =
            <&mut RigidBodySet>::query().get_mut(world, rigid_body.physics_sim_entity)
        {
            let body = &mut rigid_body_set[handle];
            if let RigidBodyType::KinematicVelocityBased = body.body_type() {
                if let Some(velocity) = velocity {
                    body.set_linvel(
                        vector![velocity.x, velocity.y, velocity.z],
                        velocity.magnitude() > 0.0,
                    );
                }
            }
        }
    }
}

pub fn systems(builder: &mut Builder) -> &mut Builder {
    builder
        .add_system(create_rigid_bodies_system())
        .add_system(create_colliders_system())
        .add_system(rigid_body_kinematic_position_system())
        .add_system(rigid_body_kinematic_velocity_system())
        .add_system(rapier3d_tick_system())
        .add_system(rigid_body_readback_system())
}
