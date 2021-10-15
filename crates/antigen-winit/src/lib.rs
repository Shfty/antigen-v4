use legion::{Entity, World};
use remote_channel::*;
use serde::ser::SerializeStruct;
use std::collections::{HashMap, HashSet};
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
    Valid(WindowId),
    Closed,
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState::Invalid
    }
}

impl From<WindowId> for WindowState {
    fn from(w: WindowId) -> Self {
        WindowState::Valid(w)
    }
}

#[derive(Default)]
pub struct WindowComponent {
    state: WindowState,
    always_redraw: bool,
}

impl WindowComponent {
    pub fn always_redraw() -> Self {
        WindowComponent {
            state: Default::default(),
            always_redraw: true,
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
            WindowState::Valid(window) => {
                s.serialize_field("state", &format!("Valid({:#?})", window))?
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
        if let WindowState::Valid(window_id) = self.state {
            Some(window_id)
        } else {
            None
        }
    }

    pub fn state(&self) -> &WindowState {
        &self.state
    }

    pub fn set_invalid(&mut self) {
        self.state = WindowState::Invalid;
    }

    pub fn set_pending(&mut self) {
        self.state = WindowState::Pending;
    }

    pub fn set_valid(&mut self, window_id: WindowId) {
        self.state = WindowState::Valid(window_id);
    }

    pub fn set_closed(&mut self) {
        self.state = WindowState::Closed
    }
}

#[derive(Debug, Default)]
pub struct WindowManager {
    windows: HashMap<Entity, Window>,
    entity_ids: HashMap<WindowId, Entity>,
    always_redraw_windows: HashSet<WindowId>,
}

impl WindowManager {
    pub fn window(&self, entity: &Entity) -> Option<&Window> {
        self.windows.get(entity)
    }
    pub fn entity_id(&self, window_id: &WindowId) -> Option<Entity> {
        self.entity_ids.get(window_id).map(|entity| *entity)
    }

    pub fn create_window_for(
        &mut self,
        entity: Entity,
        window_target: &EventLoopWindowTarget<()>,
    ) -> WindowId {
        let window = Window::new(window_target).unwrap();
        let window_id = window.id();
        self.windows.insert(entity, window);
        self.entity_ids.insert(window_id, entity);
        window_id
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

    pub fn close_window(&mut self, window_id: &WindowId) {
        let entity = *self.entity_ids.get(window_id).unwrap();
        self.windows
            .remove(&entity)
            .unwrap_or_else(|| panic!("No window with ID {:?}", entity));

        self.always_redraw_windows.remove(window_id);

        self.entity_ids.remove(window_id);
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
            let window_id = wm.create_window_for(entity, window_target);
            if always_redraw {
                wm.set_window_always_redraw(&window_id, true);
            }

            Box::new(move |world: &mut World| {
                if let Some(mut entry) = world.entry(entity) {
                    if let Ok(window) = entry.get_component_mut::<WindowComponent>() {
                        window.set_valid(window_id);
                    }
                }
            })
        }));
    }
}
