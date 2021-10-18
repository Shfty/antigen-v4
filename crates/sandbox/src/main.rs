mod assemblages;
/// Sandbox
///
/// Development sandbox for antigen functionality
mod components;
mod renderers;
mod resources;
mod systems;

use antigen_cgmath::components::ViewProjectionMatrix;
use antigen_components::{Image, ImageComponent};
use antigen_resources::Timing;
use antigen_wgpu::{components::UniformWrite, WgpuManager, WgpuRequester, WgpuResponder};

use components::*;
use resources::*;
use systems::*;

use crossbeam_channel::{Receiver, Sender};
use legion_debugger::{Archetypes, Entities};
use reflection::data::Data;

use antigen_winit::{
    components::RedrawMode, window_manager::WindowManager, WinitRequester, WinitResponder,
};
use reflection_tui::{standard_widgets, DataWidget, ReflectionWidget, ReflectionWidgetState};
use remote_channel::*;
use tui_debugger::{Resources as TraceResources, TuiDebugger};

use legion::*;
use parking_lot::{Mutex, RwLock};
use std::{
    any::TypeId,
    cell::RefCell,
    sync::Arc,
    time::{Duration, Instant},
};
use wgpu::{Queue, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
};

const GAME_TICK_HZ: f64 = 60.0;
const GAME_TICK_SECS: f64 = 1.0 / GAME_TICK_HZ;

const INPUT_TICK_HZ: f64 = 1000.0;
const INPUT_TICK_SECS: f64 = 1.0 / INPUT_TICK_HZ;

thread_local!(static MAIN_THREAD: RefCell<bool> = false.into());

#[profiling::function]
fn build_world(wgpu_manager: &WgpuManager) -> World {
    log::trace!("Building world");
    let mut world = World::default();

    log::trace!("Populating entities");

    //assemblages::hello_triangle_renderer(&mut world, wgpu_manager);
    assemblages::cube_renderer(&mut world, wgpu_manager);
    assemblages::msaa_lines_renderer(&mut world, wgpu_manager);
    assemblages::boids_renderer(&mut world, wgpu_manager);
    //assemblages::conservative_raster_renderer(&mut world, wgpu_manager);
    //assemblages::mipmap_renderer(&mut world, wgpu_manager);
    //assemblages::texture_arrays_renderer(&mut world, wgpu_manager);
    //assemblages::shadow_renderer(&mut world, wgpu_manager);
    //assemblages::bunnymark_renderer(&mut world, wgpu_manager);
    //assemblages::skybox_renderer(&mut world, wgpu_manager);
    //assemblages::water_renderer(&mut world, wgpu_manager);

    // Test entity
    let entity: Entity = world.push((Position { x: 0.0, y: 0.0 }, Velocity { dx: 0.5, dy: 0.0 }));

    let _entities: &[Entity] = world.extend(vec![
        (Position { x: 0.0, y: 0.0 }, Velocity { dx: 1.0, dy: 3.0 }),
        (Position { x: 1.0, y: 1.0 }, Velocity { dx: 2.0, dy: 2.0 }),
        (Position { x: 2.0, y: 2.0 }, Velocity { dx: 3.0, dy: 1.0 }),
    ]);

    if let Some(mut entry) = world.entry(entity) {
        // add an extra component
        entry.add_component(12f32);

        // access the entity's components, returns `None` if the entity does not have the component
        assert_eq!(entry.get_component::<f32>().unwrap(), &12f32);
    }

    // entries return `None` if the entity does not exist
    if let Some(mut entry) = world.entry(entity) {
        // add an extra component
        entry.add_component(12f32);

        // access the entity's components, returns `None` if the entity does not have the component
        assert_eq!(entry.get_component::<f32>().unwrap(), &12f32);
    }

    world
}

/// A type that can be shared across threads and build a set of legion [`Resources`]
///
/// Used to distribute shared resources allocated on the main thread
/// without having to treat them as components
trait SharedState: Send + Sync {
    fn resources(&self) -> legion::Resources;
}

#[derive(Debug, Default, Clone)]
struct Shared {
    trace_archetypes: Arc<RwLock<Archetypes>>,
    trace_entities: Arc<RwLock<Entities>>,
    trace_resources: Arc<RwLock<TraceResources>>,
}

impl SharedState for Shared {
    fn resources(&self) -> legion::Resources {
        let mut resources = legion::Resources::default();
        resources.insert(self.trace_archetypes.clone());
        resources.insert(self.trace_entities.clone());
        resources.insert(self.trace_resources.clone());
        resources
    }
}

fn main() {
    MAIN_THREAD.with(|f| *f.borrow_mut() = true);

    profiling::scope!("Main");

    let shared_state = Shared::default();

    let (crossterm_tx, crossterm_rx) = crossbeam_channel::unbounded();

    let window_manager = WindowManager::default();
    let (winit_requester, winit_responder) = remote_channel(window_manager);

    let instance = wgpu::Instance::new(wgpu::Backends::all());

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::POLYGON_MODE_LINE
                | wgpu::Features::CONSERVATIVE_RASTERIZATION
                | wgpu::Features::SPIRV_SHADER_PASSTHROUGH
                | wgpu::Features::PUSH_CONSTANTS
                | wgpu::Features::TEXTURE_BINDING_ARRAY
                //| wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                | wgpu::Features::UNSIZED_BINDING_ARRAY,
                limits: wgpu::Limits {
                    max_push_constant_size: 4,
                    ..wgpu::Limits::downlevel_defaults()
                }
                .using_resolution(adapter.limits()),
            },
            None,
        ),
    )
    .unwrap();

    let wgpu_manager = WgpuManager::new(instance, adapter, device, queue);
    let queue = wgpu_manager.queue();

    let world = Arc::new(Mutex::new(build_world(&wgpu_manager)));

    let (wgpu_requester, wgpu_responder) = remote_channel(wgpu_manager);

    std::thread::spawn(game_thread(
        world.clone(),
        shared_state.clone(),
        winit_requester,
        wgpu_requester,
        queue,
    ));
    std::thread::spawn(tui_input_thread(crossterm_tx));

    let resize_schedule = Schedule::builder()
        .add_system(antigen_wgpu::systems::aspect_ratio_system())
        .add_system(antigen_cgmath::systems::look_at_system())
        .add_system(antigen_cgmath::systems::perspective_projection_system())
        .flush()
        .add_system(antigen_cgmath::systems::view_projection_matrix_system())
        .add_system(antigen_wgpu::systems::uniform_write_system::<
            ViewProjectionMatrix,
        >())
        .build();

    winit::event_loop::EventLoop::new().run(winit_thread(
        world,
        shared_state.clone(),
        crossterm_rx,
        winit_responder,
        wgpu_responder,
        Some(resize_schedule),
        None,
    ));
}

fn game_thread<'a>(
    world: Arc<Mutex<World>>,
    shared_state: Shared,
    winit_requester: WinitRequester,
    wgpu_requester: WgpuRequester,
    queue: Arc<Queue>,
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
            .add_system(game_update_positions_system())
            .add_system(renderers::cube::update_look_system())
            .add_system(renderers::cube::update_projection_system())
            .flush()
            .add_system(antigen_wgpu::systems::aspect_ratio_system())
            .add_system(antigen_cgmath::systems::look_at_system())
            .add_system(antigen_cgmath::systems::perspective_projection_system())
            .flush()
            .add_system(antigen_cgmath::systems::view_projection_matrix_system())
            .add_system(antigen_wgpu::systems::uniform_write_system::<
                ViewProjectionMatrix,
            >())
            .add_system(antigen_wgpu::systems::texture_write_system::<ImageComponent>());

        let mut schedule = builder.build();

        let mut resources = shared_state.resources();
        resources.insert(winit_requester);
        resources.insert(wgpu_requester);
        resources.insert(Timing::default());
        resources.insert(queue);

        let tick_duration = Duration::from_secs_f64(GAME_TICK_SECS);
        loop {
            let timestamp = Instant::now();

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

            drop(world);

            while timestamp.elapsed() < tick_duration {}
        }
    }
}

#[profiling::function]
fn tui_input_thread(sender: Sender<crossterm::event::Event>) -> impl FnOnce() {
    move || {
        let tick_duration = Duration::from_secs_f64(INPUT_TICK_SECS);
        loop {
            let timestamp = Instant::now();
            crossterm_poll_input(&sender);
            while timestamp.elapsed() < tick_duration {}
        }
    }
}

#[profiling::function]
fn winit_thread<'a>(
    world: Arc<Mutex<World>>,
    shared_state: Shared,
    crossterm_rx: Receiver<crossterm::event::Event>,
    mut winit_responder: WinitResponder,
    mut wgpu_responder: WgpuResponder,
    mut resize_schedule: Option<Schedule>,
    mut close_schedule: Option<Schedule>,
) -> impl FnMut(Event<()>, &EventLoopWindowTarget<()>, &mut ControlFlow) + 'a {
    // Both winit and crossterm/tui must run on the main thread
    MAIN_THREAD.with(|f| {
        assert!(
            *f.borrow(),
            "winit_thread may only be called from the main thread"
        )
    });

    let mut main_loop_state = MainLoopState::default();
    let mut crossterm_event_queue = CrosstermEventQueue::default();
    let mut tui_debugger = TuiDebugger::start().unwrap();
    let mut reflection_widget_state = ReflectionWidgetState::None;

    let mut resources = Resources::default();
    resources.insert(wgpu_responder.queue());

    move |event: Event<()>,
          window_target: &EventLoopWindowTarget<()>,
          control_flow: &mut ControlFlow| {
        profiling::scope!("Winit Event Loop");

        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                profiling::scope!("MainEventsCleared");

                crossterm_input_buffer_fill(&crossterm_rx, &mut crossterm_event_queue);
                crossterm_quit_on_ctrl_c(&crossterm_event_queue, &mut main_loop_state);
                for event in crossterm_event_queue.iter() {
                    reflection_widget_state.handle_input(event);
                }
                crossterm_input_buffer_clear(&mut crossterm_event_queue);

                winit_responder.receive_requests(window_target);
                wgpu_responder.receive_requests(&());

                let archetypes = shared_state.trace_archetypes.read();
                let entities = shared_state.trace_entities.read();
                let trace_resources = shared_state.trace_resources.read();

                let mut debugger_data = Data::Struct {
                    name: "Legion Debugger",
                    fields: vec![
                        ("Archetypes", archetypes.archetypes().unwrap().clone()),
                        ("Entities", entities.entities().unwrap().clone()),
                        ("Resources", trace_resources.resources().unwrap().clone()),
                    ],
                };

                tui_debugger
                    .terminal()
                    .draw(|f| {
                        f.render_stateful_widget(
                            ReflectionWidget::new(&mut debugger_data, &widget_rules),
                            f.size(),
                            &mut reflection_widget_state,
                        )
                    })
                    .unwrap();

                let mut encoder = wgpu_responder
                    .device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let frames = winit_responder
                    .window_redraw_modes()
                    .flat_map(|(window_id, redraw_mode)| {
                        let entity = winit_responder.entity_id(&window_id).unwrap();

                        match *redraw_mode {
                            RedrawMode::MainEventsClearedRequest => {
                                winit_responder.window(&entity).unwrap().request_redraw();
                                return None;
                            }
                            RedrawMode::MainEventsClearedLoop => (),
                            _ => return None,
                        }

                        let surface = if let Some(surface) = wgpu_responder.surface(&entity) {
                            surface
                        } else {
                            return None;
                        };

                        let frame = if let Ok(frame) = surface.get_current_texture() {
                            frame
                        } else {
                            return None;
                        };

                        redraw_window(&wgpu_responder, &entity, &mut encoder, &frame);

                        Some(frame)
                    })
                    .collect::<Vec<SurfaceTexture>>();

                // Submit queue
                wgpu_responder
                    .queue()
                    .submit(std::iter::once(encoder.finish()));

                // Present frames
                frames
                    .into_iter()
                    .map(SurfaceTexture::present)
                    .for_each(drop);

                // Exit if requested by the main loop state
                if let MainLoopState::Break = main_loop_state {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::RedrawRequested(window_id) => {
                profiling::scope!("RedrawRequested");

                let mut encoder = wgpu_responder
                    .device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let entity = winit_responder.entity_id(&window_id).unwrap();

                let surface = if let Some(surface) = wgpu_responder.surface(&entity) {
                    surface
                } else {
                    return;
                };

                let frame = if let Ok(frame) = surface.get_current_texture() {
                    frame
                } else {
                    return;
                };

                redraw_window(&wgpu_responder, &entity, &mut encoder, &frame);

                wgpu_responder
                    .queue()
                    .submit(std::iter::once(encoder.finish()));

                frame.present();
            }
            Event::WindowEvent { window_id, event } => match event {
                WindowEvent::Resized(size) => {
                    profiling::scope!("WindowEvent::Resized");
                    let entity = winit_responder
                        .entity_id(&window_id)
                        .expect("No entity for resized window");

                    let mut world = world.lock();
                    wgpu_responder.try_resize_surface(&entity, size);
                    if let Some(resize_schedule) = resize_schedule.as_mut() {
                        resize_schedule.execute(&mut world, &mut resources);
                    }
                }
                WindowEvent::CloseRequested => {
                    profiling::scope!("WindowEvent::CloseRequested");
                    let entity = winit_responder
                        .entity_id(&window_id)
                        .expect("No entity for closed window");

                    let mut world = world.lock();
                    winit_responder.close_window(&mut world, &window_id);
                    wgpu_responder.destroy_surface(&mut world, &entity);
                    if let Some(close_schedule) = close_schedule.as_mut() {
                        close_schedule.execute(&mut world, &mut resources);
                    }
                }
                _ => (),
            },
            _ => {}
        }
    }
}

fn redraw_window(
    wgpu_responder: &WgpuResponder,
    entity: &Entity,
    encoder: &mut wgpu::CommandEncoder,
    frame: &SurfaceTexture,
) {
    if let Some(render_passes) = wgpu_responder.entity_render_passes(&entity) {
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let config = wgpu_responder.surface_configuration(entity).unwrap().read();

        for render_pass_id in render_passes.iter() {
            for render_pass in wgpu_responder.render_passes().get_mut(render_pass_id) {
                render_pass.render(&wgpu_responder, encoder, &view, &config);
            }
        }
    }
}

pub fn widget_rules(data: &mut Data, parent_type: TypeId) -> Option<Box<dyn DataWidget + '_>> {
    standard_widgets(&widget_rules)(data, parent_type)
}
