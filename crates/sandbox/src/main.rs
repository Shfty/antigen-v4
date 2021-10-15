/// Sandbox
///
/// Development sandbox for antigen functionality
mod components;
mod renderers;
mod resources;
mod systems;

use components::*;
use resources::*;
use systems::*;

use renderers::cube::*;
use renderers::hello_triangle::*;

use crossbeam_channel::{Receiver, Sender};
use legion_debugger::{Archetypes, Entities};
use reflection::data::Data;

use antigen_wgpu::*;
use antigen_winit::*;
use reflection_tui::{standard_widgets, DataWidget, ReflectionWidget, ReflectionWidgetState};
use remote_channel::*;
use tui_debugger::{Resources as TraceResources, TuiDebugger};

use legion::*;
use parking_lot::RwLock;
use std::{
    any::TypeId,
    cell::RefCell,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
};

legion_debugger::register_component!(WindowComponent);
legion_debugger::register_component!(SurfaceComponent);
legion_debugger::register_component!(RenderPassComponent);
legion_debugger::register_component!(CubeRenderStateComponent);

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

    // Triangle renderer
    let triangle_pass_id =
        wgpu_manager.add_render_pass(Box::new(TriangleRenderer::new(&wgpu_manager)));

    let mut triangle_pass_component = RenderPassComponent::default();
    triangle_pass_component.add_render_pass(triangle_pass_id);

    world.push((
        WindowComponent::default(),
        SurfaceComponent::default(),
        triangle_pass_component,
    ));

    // Cube renderer
    let cube_render_entity = world.push(());

    let cube_renderer = CubeRenderer::new(wgpu_manager, cube_render_entity);
    let cube_render_state_component =
        CubeRenderStateComponent::from(cube_renderer.take_state_handle());
    let cube_pass_id = wgpu_manager.add_render_pass(Box::new(cube_renderer));

    let mut cube_pass_component = RenderPassComponent::default();
    cube_pass_component.add_render_pass(cube_pass_id);

    world.push_with_id(
        cube_render_entity,
        (
            WindowComponent::default(),
            SurfaceComponent::default(),
            cube_pass_component,
            cube_render_state_component,
        ),
    );

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
    let (wm_requester, wm_responder) = remote_channel(window_manager);

    let instance = wgpu::Instance::new(wgpu::Backends::all());

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::NON_FILL_POLYGON_MODE,
            limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
        },
        None,
    ))
    .unwrap();

    let wgpu_manager = WgpuManager::new(instance, adapter, device, queue);

    let world = build_world(&wgpu_manager);

    let (wgpu_requester, wgpu_responder) = remote_channel(wgpu_manager);

    std::thread::spawn(game_thread(
        world,
        shared_state.clone(),
        wm_requester,
        wgpu_requester,
    ));
    std::thread::spawn(tui_input_thread(crossterm_tx));
    winit::event_loop::EventLoop::new().run(winit_thread(
        shared_state.clone(),
        crossterm_rx,
        wm_responder,
        wgpu_responder,
    ));
}

fn game_thread<'a>(
    mut world: World,
    shared_state: Shared,
    winit_requester: WinitRequester,
    wgpu_requester: WgpuRequester,
) -> impl FnOnce() + 'a {
    move || {
        let mut schedule = Schedule::builder()
            .add_system(timing_update_system(Instant::now()))
            .flush()
            .add_system(create_windows_system())
            .add_system(create_surfaces_system())
            .add_system(register_render_passes_system())
            .add_system(game_update_positions_system())
            .add_system(integrate_cube_renderer_system())
            .build();

        let mut resources = shared_state.resources();
        resources.insert(winit_requester);
        resources.insert(wgpu_requester);
        resources.insert(Timing::default());

        let tick_duration = Duration::from_secs_f64(GAME_TICK_SECS);
        loop {
            let timestamp = Instant::now();
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
    shared_state: Shared,
    crossterm_rx: Receiver<crossterm::event::Event>,
    mut wm_responder: WinitResponder,
    mut wgpu_responder: WgpuResponder,
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

                wm_responder.receive_requests(window_target);
                wgpu_responder.receive_requests(&wm_responder);

                let mut archetypes = shared_state.trace_archetypes.write();
                let mut entities = shared_state.trace_entities.write();
                let mut trace_resources = shared_state.trace_resources.write();

                let mut debugger_data = Data::Struct {
                    name: "Legion Debugger",
                    fields: vec![
                        ("Archetypes", archetypes.archetypes_mut().unwrap().clone()),
                        ("Entities", entities.entities_mut().unwrap().clone()),
                        (
                            "Resources",
                            trace_resources.resources_mut().unwrap().clone(),
                        ),
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

                // Exit if requested by the main loop state
                if let MainLoopState::Break = main_loop_state {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::RedrawRequested(window_id) => {
                profiling::scope!("RedrawRequested");

                let entity = wm_responder.entity_id(&window_id).unwrap();

                let surface = if let Some(surface) = wgpu_responder.surface(&entity) {
                    surface
                } else {
                    return;
                };

                let frame = if let Ok(frame) = surface.get_current_frame() {
                    frame.output
                } else {
                    return;
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = wgpu_responder
                    .device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let format = surface
                    .get_preferred_format(wgpu_responder.adapter())
                    .unwrap();

                if let Some(render_passes) = wgpu_responder.entity_render_passes(&entity) {
                    for render_pass_id in render_passes.iter() {
                        for render_pass in wgpu_responder.render_passes().get_mut(render_pass_id) {
                            render_pass.render(&mut encoder, &wgpu_responder, &view, format.into());
                        }
                    }
                }

                wgpu_responder
                    .queue()
                    .submit(std::iter::once(encoder.finish()));
            }
            Event::WindowEvent { window_id, event } => match event {
                WindowEvent::Resized(size) => {
                    profiling::scope!("WindowEvent::Resized");
                    let entity = wm_responder
                        .entity_id(&window_id)
                        .expect("No entity for resized window");

                    wgpu_responder.try_resize_surface(&entity, size);
                }
                WindowEvent::CloseRequested => {
                    profiling::scope!("WindowEvent::CloseRequested");
                    let entity = wm_responder
                        .entity_id(&window_id)
                        .expect("No entity for closed window");

                    wm_responder.close_window(&window_id);
                    wm_responder.send_response(Box::new(move |world: &mut World| {
                        if let Some(mut entry) = world.entry(entity) {
                            if let Ok(window) = entry.get_component_mut::<WindowComponent>() {
                                window.set_closed()
                            }
                        }
                    }));

                    wgpu_responder.destroy_surface(&entity);
                    wgpu_responder.send_response(Box::new(move |world: &mut World| {
                        if let Some(mut entry) = world.entry(entity) {
                            if let Ok(surface) = entry.get_component_mut::<SurfaceComponent>() {
                                surface.set_destroyed()
                            }
                        }
                    }));
                }
                _ => (),
            },
            _ => {}
        }
    }
}

pub fn widget_rules(data: &mut Data, parent_type: TypeId) -> Option<Box<dyn DataWidget + '_>> {
    standard_widgets(&widget_rules)(data, parent_type)
}
