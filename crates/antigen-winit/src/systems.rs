use legion::{systems::Builder, Entity, World};
use on_change::OnChangeTrait;

use crate::{
    components::{
        AlwaysOnTop, CursorGrab, CursorIcon, CursorPosition, CursorVisible, Decorations,
        Fullscreen, ImePosition, InnerSize, MaxInnerSize, Maximized, MinInnerSize, Minimized,
        OuterPosition, RedrawModeComponent, Resizable, Visible, WindowComponent, WindowTitle,
    },
    window_state::WindowState,
    WinitRequester,
};

#[legion::system(par_for_each)]
pub fn create_windows(
    entity: &Entity,
    WindowComponent(window_state): &mut WindowComponent,
    #[resource] wm_requester: &WinitRequester,
) {
    let entity = *entity;
    if let WindowState::Invalid = window_state {
        *window_state = WindowState::Pending;

        wm_requester.send_request(Box::new(move |wm, window_target| {
            let window = wm.create_window_for(entity, window_target);

            Box::new(move |world: &mut World| {
                if let Some(mut entry) = world.entry(entity) {
                    if let Ok(WindowComponent(component)) =
                        entry.get_component_mut::<WindowComponent>()
                    {
                        *component = WindowState::Valid(window)
                    }
                }
            })
        }));
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<WindowTitle>())]
pub fn window_title(WindowComponent(window_state): &WindowComponent, title: &mut WindowTitle) {
    if let WindowState::Valid(window) = window_state {
        if let Some(title) = title.take_change() {
            window.set_title(title);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<AlwaysOnTop>())]
pub fn always_on_top(
    WindowComponent(window_state): &WindowComponent,
    always_on_top: &mut AlwaysOnTop,
) {
    if let WindowState::Valid(window) = window_state {
        if let Some(always_on_top) = always_on_top.take_change() {
            window.set_always_on_top(*always_on_top);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<CursorGrab>())]
pub fn cursor_grab(WindowComponent(window_state): &WindowComponent, cursor_grab: &mut CursorGrab) {
    if let WindowState::Valid(window) = window_state {
        if let Some(cursor_grab) = cursor_grab.take_change() {
            window.set_cursor_grab(*cursor_grab).unwrap();
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<CursorIcon>())]
pub fn cursor_icon(WindowComponent(window_state): &WindowComponent, cursor_icon: &mut CursorIcon) {
    if let WindowState::Valid(window) = window_state {
        if let Some(cursor_icon) = cursor_icon.take_change() {
            window.set_cursor_icon(*cursor_icon);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<CursorVisible>())]
pub fn cursor_visible(
    WindowComponent(window_state): &WindowComponent,
    cursor_visible: &mut CursorVisible,
) {
    if let WindowState::Valid(window) = window_state {
        if let Some(cursor_visible) = cursor_visible.take_change() {
            window.set_cursor_visible(*cursor_visible);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<Decorations>())]
pub fn decorations(WindowComponent(window_state): &WindowComponent, decorations: &mut Decorations) {
    if let WindowState::Valid(window) = window_state {
        if let Some(decorations) = decorations.take_change() {
            window.set_decorations(*decorations);
        }
    }
}

// TODO: Handle Exclusive fullscreen, Borderless on specific monitor
#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<Fullscreen>())]
pub fn fullscreen(WindowComponent(window_state): &WindowComponent, fullscreen: &mut Fullscreen) {
    if let WindowState::Valid(window) = window_state {
        if let Some(fullscreen) = fullscreen.take_change() {
            if *fullscreen {
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            } else {
                window.set_fullscreen(None)
            }
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<Maximized>())]
pub fn maximized(WindowComponent(window_state): &WindowComponent, maximized: &mut Maximized) {
    if let WindowState::Valid(window) = window_state {
        if let Some(maximized) = maximized.take_change() {
            window.set_maximized(*maximized);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<Minimized>())]
pub fn minimized(WindowComponent(window_state): &WindowComponent, minimized: &mut Minimized) {
    if let WindowState::Valid(window) = window_state {
        if let Some(minimized) = minimized.take_change() {
            window.set_minimized(*minimized);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<Resizable>())]
pub fn resizable(WindowComponent(window_state): &WindowComponent, resizable: &mut Resizable) {
    if let WindowState::Valid(window) = window_state {
        if let Some(resizable) = resizable.take_change() {
            window.set_resizable(*resizable);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<Visible>())]
pub fn visible(WindowComponent(window_state): &WindowComponent, visible: &mut Visible) {
    if let WindowState::Valid(window) = window_state {
        if let Some(visible) = visible.take_change() {
            window.set_visible(*visible);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<CursorPosition>())]
pub fn cursor_position(
    WindowComponent(window_state): &WindowComponent,
    cursor_position: &mut CursorPosition,
) {
    if let WindowState::Valid(window) = window_state {
        if let Some(cursor_position) = cursor_position.take_change() {
            window.set_cursor_position(*cursor_position).unwrap();
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<ImePosition>())]
pub fn ime_position(
    WindowComponent(window_state): &WindowComponent,
    ime_position: &mut ImePosition,
) {
    if let WindowState::Valid(window) = window_state {
        if let Some(ime_position) = ime_position.take_change() {
            window.set_ime_position(*ime_position);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<MaxInnerSize>())]
pub fn max_inner_size(
    WindowComponent(window_state): &WindowComponent,
    max_inner_size: &mut MaxInnerSize,
) {
    if let WindowState::Valid(window) = window_state {
        if let Some(max_inner_size) = max_inner_size.take_change() {
            window.set_max_inner_size(*max_inner_size);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<MinInnerSize>())]
pub fn min_inner_size(
    WindowComponent(window_state): &WindowComponent,
    min_inner_size: &mut MinInnerSize,
) {
    if let WindowState::Valid(window) = window_state {
        if let Some(min_inner_size) = min_inner_size.take_change() {
            window.set_min_inner_size(*min_inner_size);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<InnerSize>())]
pub fn inner_size(WindowComponent(window_state): &WindowComponent, inner_size: &mut InnerSize) {
    if let WindowState::Valid(window) = window_state {
        if let Some(inner_size) = inner_size.take_change() {
            window.set_inner_size(*inner_size);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<OuterPosition>())]
pub fn outer_position(
    WindowComponent(window_state): &WindowComponent,
    outer_position: &mut OuterPosition,
) {
    if let WindowState::Valid(window) = window_state {
        if let Some(outer_position) = outer_position.take_change() {
            window.set_outer_position(*outer_position);
        }
    }
}

#[legion::system(par_for_each)]
#[filter(legion::maybe_changed::<RedrawModeComponent>())]
pub fn redraw_mode(
    WindowComponent(window_state): &WindowComponent,
    redraw_mode: &mut RedrawModeComponent,
    #[resource] wm_requester: &WinitRequester,
) {
    if let WindowState::Valid(window) = window_state {
        if let Some(redraw_mode) = redraw_mode.take_change() {
            let window_id = window.id();
            let redraw_mode = *redraw_mode;

            wm_requester.send_request(Box::new(move |wm, _| {
                wm.set_window_redraw_mode(window_id, redraw_mode);

                Box::new(|_| ())
            }))
        }
    }
}

/// Add all winit systems to the provided [`Builder`].
///
/// Systems may also be added individually for client code
/// that only uses a subset of the relevant components.
pub fn systems(builder: &mut Builder) -> &mut Builder {
    builder
        .add_system(create_windows_system())
        .flush()
        .add_system(window_title_system())
        .add_system(always_on_top_system())
        .add_system(cursor_grab_system())
        .add_system(cursor_icon_system())
        .add_system(cursor_visible_system())
        .add_system(decorations_system())
        .add_system(fullscreen_system())
        .add_system(maximized_system())
        .add_system(minimized_system())
        .add_system(resizable_system())
        .add_system(visible_system())
        .add_system(cursor_position_system())
        .add_system(ime_position_system())
        .add_system(max_inner_size_system())
        .add_system(min_inner_size_system())
        .add_system(inner_size_system())
        .add_system(outer_position_system())
        .add_system(redraw_mode_system())
}
