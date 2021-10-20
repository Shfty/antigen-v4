use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, thread::JoinHandle};

use antigen_wgpu::WgpuResponder;
use antigen_winit::{WinitResponder, components::RedrawMode};
use legion::{Entity, Resources, Schedule, World};
use parking_lot::Mutex;
use wgpu::SurfaceTexture;
use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoopWindowTarget}};

#[profiling::function]
pub fn winit_thread<'a>(
    world: Arc<Mutex<World>>,
    mut winit_responder: WinitResponder,
    mut wgpu_responder: WgpuResponder,
    mut resize_schedule: Option<Schedule>,
    mut close_schedule: Option<Schedule>,
    main_loop_break: Arc<AtomicBool>,
    mut join_handles: Vec<JoinHandle<()>>,
) -> impl FnMut(Event<()>, &EventLoopWindowTarget<()>, &mut ControlFlow) + 'a {
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

                winit_responder.receive_requests(window_target);
                wgpu_responder.receive_requests(&());

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
                if main_loop_break.load(Ordering::Relaxed) {
                    for join_handle in join_handles.drain(..) {
                        join_handle.join().unwrap();
                    }
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
