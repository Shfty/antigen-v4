use legion::{Entity, World};
use remote_channel::*;
use serde::ser::SerializeStruct;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use winit::{
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowId},
};

pub type WinitRequester = RemoteRequester<WindowManager, EventLoopWindowTarget<()>, World>;
pub type WinitResponder = RemoteResponder<WindowManager, EventLoopWindowTarget<()>, World>;

#[derive(Debug)]
pub enum WindowState {
    Invalid,
    Pending,
    Valid(WindowId, Arc<Window>),
    Closed,
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState::Invalid
    }
}

#[derive(Default)]
pub struct WindowComponent {
    state: WindowState,
    always_redraw: bool,
    prev_name: Option<String>,
}

impl WindowComponent {
    pub fn always_redraw() -> Self {
        WindowComponent {
            always_redraw: true,
            ..Default::default()
        }
    }
}

impl serde::Serialize for WindowComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("WindowComponent", 2)?;
        match &self.state {
            WindowState::Invalid => s.serialize_field("state", "Invalid")?,
            WindowState::Pending => s.serialize_field("state", "Pending")?,
            WindowState::Valid(window_id, window) => {
                s.serialize_field("state", &format!("Valid({:?}: {:#?})", window_id, window))?
            }
            WindowState::Closed => s.serialize_field("state", "Closed")?,
        }
        s.serialize_field("always_redraw", &self.always_redraw)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for WindowComponent {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Default::default())
    }
}

impl WindowComponent {
    pub fn id(&self) -> Option<WindowId> {
        if let WindowState::Valid(window_id, _) = self.state {
            Some(window_id)
        } else {
            None
        }
    }

    pub fn window(&self) -> Option<&Arc<Window>> {
        if let WindowState::Valid(_, window) = &self.state {
            Some(window)
        } else {
            None
        }
    }

    pub fn state(&self) -> &WindowState {
        &self.state
    }

    pub fn prev_name(&self) -> Option<&str> {
        self.prev_name.as_deref()
    }

    pub fn set_prev_name(&mut self, prev_name: Option<String>) {
        self.prev_name = prev_name
    }

    pub fn set_invalid(&mut self) {
        self.state = WindowState::Invalid;
    }

    pub fn set_pending(&mut self) {
        self.state = WindowState::Pending;
    }

    pub fn set_valid(&mut self, window_id: WindowId, window: Arc<Window>) {
        self.state = WindowState::Valid(window_id, window);
    }

    pub fn set_closed(&mut self) {
        self.state = WindowState::Closed
    }
}

#[derive(Debug, Default)]
pub struct WindowManager {
    windows: HashMap<Entity, Arc<Window>>,
    entity_ids: HashMap<WindowId, Entity>,
    always_redraw_windows: HashSet<WindowId>,
}

impl WindowManager {
    pub fn window(&self, entity: &Entity) -> Option<&Arc<Window>> {
        self.windows.get(entity)
    }
    pub fn entity_id(&self, window_id: &WindowId) -> Option<Entity> {
        self.entity_ids.get(window_id).map(|entity| *entity)
    }

    pub fn create_window_for(
        &mut self,
        entity: Entity,
        window_target: &EventLoopWindowTarget<()>,
    ) -> (WindowId, Arc<Window>) {
        let window = Arc::new(Window::new(window_target).unwrap());
        let window_id = window.id();
        self.windows.insert(entity, window.clone());
        self.entity_ids.insert(window_id, entity);
        (window_id, window)
    }

    pub fn always_redraw_windows(&self) -> impl Iterator<Item = &WindowId> {
        self.always_redraw_windows.iter()
    }

    pub fn set_window_always_redraw(&mut self, window_id: &WindowId, always_redraw: bool) {
        if always_redraw {
            self.always_redraw_windows.insert(*window_id);
        } else {
            self.always_redraw_windows.remove(window_id);
        }
    }

    pub fn close_window(&mut self, world: &mut World, window_id: &WindowId) {
        let entity = *self.entity_ids.get(window_id).unwrap();
        self.windows
            .remove(&entity)
            .unwrap_or_else(|| panic!("No window with ID {:?}", entity));

        self.always_redraw_windows.remove(window_id);

        self.entity_ids.remove(window_id);

        if let Some(mut entry) = world.entry(entity) {
            if let Ok(window) = entry.get_component_mut::<WindowComponent>() {
                window.set_closed()
            }
        }
    }
}

#[legion::system(par_for_each)]
pub fn create_windows(
    entity: &Entity,
    window: &mut WindowComponent,
    #[resource] wm_requester: &WinitRequester,
) {
    let entity = *entity;
    if let WindowState::Invalid = window.state() {
        window.set_pending();
        let always_redraw = window.always_redraw;

        wm_requester.send_request(Box::new(move |wm, window_target| {
            let (window_id, window) = wm.create_window_for(entity, window_target);
            if always_redraw {
                wm.set_window_always_redraw(&window_id, true);
            }

            Box::new(move |world: &mut World| {
                if let Some(mut entry) = world.entry(entity) {
                    if let Ok(component) = entry.get_component_mut::<WindowComponent>() {
                        component.set_valid(window_id, window);
                    }
                }
            })
        }));
    }
}

#[legion::system(par_for_each)]
pub fn name_windows(
    name: &antigen_components::Name,
    window_component: &mut WindowComponent,
) {
    match window_component.state() {
        WindowState::Valid(_, window) => {
            if window_component.prev_name() != Some(&***name) {
                window.set_title(name);
                window_component.set_prev_name(Some((***name).into()));
            }
        }
        _ => (),
    }
}
