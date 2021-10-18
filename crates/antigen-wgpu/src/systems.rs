use std::sync::Arc;

use antigen_cgmath::components::AspectRatio;
use antigen_winit::components::WindowComponent;
use legion::{
    query::IntoQuery, storage::Component, systems::Builder, world::SubWorld, Entity, World,
};
use wgpu::{Queue, SurfaceConfiguration};

use crate::{
    components::{
        BufferComponent, BufferWrite, RenderPassComponent, SurfaceComponent, TextureComponent,
        TextureWrite,
    },
    CastSlice, RenderPassState, SurfaceState, WgpuRequester,
};
use antigen_winit::WindowState;
use on_change::OnChangeTrait;

#[legion::system(par_for_each)]
pub fn create_surfaces(
    entity: &Entity,
    WindowComponent(window_state): &WindowComponent,
    surface: &mut SurfaceComponent,
    #[resource] wgpu_requester: &WgpuRequester,
) {
    let entity = *entity;
    if let WindowState::Valid(window) = window_state {
        if let SurfaceState::Invalid = surface.state() {
            surface.set_pending();

            let window = window.clone();
            wgpu_requester.send_request(Box::new(move |wgpu_manager, _| {
                let config = wgpu_manager.create_surface_for(entity, &window);
                window.request_redraw();

                Box::new(move |world: &mut World| {
                    if let Some(mut entry) = world.entry(entity) {
                        if let Ok(surface) = entry.get_component_mut::<SurfaceComponent>() {
                            surface.set_valid(config);
                        }
                    }
                })
            }))
        }
    }
}

#[legion::system(par_for_each)]
pub fn register_render_passes(
    entity: &Entity,
    render_passes: &mut RenderPassComponent,
    #[resource] wgpu_requester: &WgpuRequester,
) {
    let entity = *entity;

    for (state, render_pass) in render_passes.passes_mut().iter_mut() {
        let render_pass = *render_pass;

        if let RenderPassState::Invalid = state {
            *state = RenderPassState::Pending;
            wgpu_requester.send_request(Box::new(move |wgpu_manager, _| {
                wgpu_manager.register_render_pass_for_entity(&render_pass, &entity);
                Box::new(move |world: &mut World| {
                    if let Some(mut entry) = world.entry(entity) {
                        if let Ok(render_passes) = entry.get_component_mut::<RenderPassComponent>()
                        {
                            render_passes
                                .passes_mut()
                                .iter_mut()
                                .find(|(_, pass)| *pass == render_pass)
                                .unwrap()
                                .0 = RenderPassState::Registered;
                        }
                    }
                })
            }));
        }
    }
}

#[legion::system(par_for_each)]
#[read_component(T)]
#[read_component(BufferComponent)]
#[filter(legion::maybe_changed::<T>())]
pub fn buffer_write<
    T: Component + OnChangeTrait<D> + Send + Sync + 'static,
    D: CastSlice<u8> + Send + Sync + 'static,
>(
    world: &SubWorld,
    entity: &Entity,
    buffer_write: &BufferWrite<T, D>,
    #[resource] queue: &Arc<Queue>,
) {
    let from = buffer_write.from_entity().unwrap_or(entity);
    if let Ok(value) = <&T>::query().get(world, *from) {
        let to = buffer_write.to_entity().unwrap_or(entity);
        if let Ok(uniform_buffer) = <&BufferComponent>::query().get(world, *to) {
            if let Some(value) = value.take_change() {
                queue.write_buffer(uniform_buffer, buffer_write.offset(), value.cast_slice());
            }
        }
    }
}

#[legion::system(par_for_each)]
#[read_component(T)]
#[read_component(TextureComponent)]
#[filter(legion::maybe_changed::<T>())]
pub fn texture_write<
    T: Component + OnChangeTrait<D> + Send + Sync + 'static,
    D: CastSlice<u8> + Send + Sync + 'static,
>(
    world: &SubWorld,
    entity: &Entity,
    texture_write: &TextureWrite<T, D>,
    #[resource] queue: &Arc<Queue>,
) {
    let from = texture_write.from_entity().unwrap_or(entity);
    if let Ok(value) = <&T>::query().get(world, *from) {
        let to = texture_write.to_entity().unwrap_or(entity);
        if let Ok(texture) = <&TextureComponent>::query().get(world, *to) {
            if let Some(value) = value.take_change() {
                let data = value.cast_slice();
                let data_layout = *texture_write.data_layout();
                let extent = *texture_write.extent();

                queue.write_texture(texture.as_image_copy(), data, data_layout, extent);
            }
        }
    }
}

#[legion::system(par_for_each)]
pub fn aspect_ratio(surface: &SurfaceComponent, AspectRatio(aspect_ratio): &mut AspectRatio) {
    match surface.state() {
        SurfaceState::Valid(config) => {
            let SurfaceConfiguration { width, height, .. } = *config.read();
            *aspect_ratio = width as f32 / height as f32
        }
        _ => (),
    }
}

pub fn systems(builder: &mut Builder) -> &mut Builder {
    builder
        .add_system(create_surfaces_system())
        .add_system(register_render_passes_system())
}
