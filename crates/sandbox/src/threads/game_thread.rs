use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant}};

use antigen_cgmath::components::ViewProjectionMatrix;
use antigen_components::{Image, ImageComponent};
use antigen_resources::Timing;
use antigen_wgpu::{DrawIndexedIndirect, WgpuRequester};
use antigen_winit::WinitRequester;
use legion::{Schedule, World};
use parking_lot::Mutex;
use wgpu::Queue;

use crate::{Shared, SharedState, renderers::cube::{
        IndexedIndirectComponent, Indices, Instance, InstanceComponent, Vertex, Vertices,
    }, spin_loop, systems::{
        tui_debugger_parse_archetypes_thread_local, tui_debugger_parse_entities_thread_local,
        tui_debugger_parse_resources_thread_local,
    }};

const GAME_TICK_HZ: f64 = 60.0;
const GAME_TICK_SECS: f64 = 1.0 / GAME_TICK_HZ;

pub fn game_thread<'a>(
    world: Arc<Mutex<World>>,
    shared_state: Shared,
    winit_requester: WinitRequester,
    wgpu_requester: WgpuRequester,
    queue: Arc<Queue>,
    main_loop_break: Arc<AtomicBool>,
) -> impl FnOnce() + 'a {
    move || {
        let mut builder = Schedule::builder();

        builder
            .add_system(antigen_resources::systems::timing_update_system(
                Instant::now(),
            ))
            .flush();

        antigen_winit::systems::systems(&mut builder);
        antigen_wgpu::systems::systems(&mut builder);

        builder
            .add_system(crate::systems::game_update_positions_system())
            .add_system(crate::renderers::cube::update_look_system())
            .add_system(crate::renderers::cube::update_projection_system())
            .flush()
            .add_system(antigen_wgpu::systems::aspect_ratio_system())
            .add_system(antigen_cgmath::systems::look_at_system())
            .add_system(antigen_cgmath::systems::perspective_projection_system())
            .flush()
            .add_system(antigen_cgmath::systems::view_projection_matrix_system())
            .add_system(antigen_wgpu::systems::buffer_write_system::<
                Vertices,
                Vec<Vertex>,
            >())
            .add_system(antigen_wgpu::systems::buffer_write_system::<
                Indices,
                Vec<u16>,
            >())
            .add_system(antigen_wgpu::systems::buffer_write_system::<
                InstanceComponent,
                Instance,
            >())
            .add_system(antigen_wgpu::systems::buffer_write_system::<
                IndexedIndirectComponent,
                DrawIndexedIndirect,
            >())
            .add_system(antigen_wgpu::systems::texture_write_system::<
                ImageComponent,
                Image,
            >())
            .add_system(antigen_wgpu::systems::buffer_write_system::<
                ViewProjectionMatrix,
                antigen_cgmath::cgmath::Matrix4<f32>,
            >());

        let mut schedule = builder.build();

        let mut resources = shared_state.resources();
        resources.insert(winit_requester);
        resources.insert(wgpu_requester);
        resources.insert(Timing::default());
        resources.insert(queue);

        spin_loop(Duration::from_secs_f64(GAME_TICK_SECS), move || {
            let mut world = world.lock();

            resources
                .get_mut::<WinitRequester>()
                .unwrap()
                .receive_responses(&mut world);

            resources
                .get_mut::<WgpuRequester>()
                .unwrap()
                .receive_responses(&mut world);

            schedule.execute(&mut world, &mut resources);

            tui_debugger_parse_archetypes_thread_local()(&mut world, &mut resources);
            tui_debugger_parse_entities_thread_local()(&mut world, &mut resources);
            tui_debugger_parse_resources_thread_local()(&mut world, &mut resources);

            main_loop_break.load(Ordering::Relaxed)
        })()
    }
}
