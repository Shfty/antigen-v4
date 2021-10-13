use legion::{Entity, World};
use remote_channel::*;
use std::collections::HashMap;
use winit::{event_loop::EventLoopWindowTarget, window::{Window, WindowId}};

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
}

impl serde::Serialize for WindowComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.state {
            WindowState::Invalid => serializer.serialize_str("Invalid"),
            WindowState::Pending => serializer.serialize_str("Pending"),
            WindowState::Valid(window) => {
                serializer.serialize_str(&format!("Valid({:#?})", window))
            }
            WindowState::Closed => serializer.serialize_str("Closed"),
        }
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

    pub fn close_window(&mut self, window_id: &WindowId) {
        let entity = *self.entity_ids.get(window_id).unwrap();
        self
            .windows
            .remove(&entity)
            .unwrap_or_else(|| panic!("No window with ID {:?}", entity));

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

        wm_requester.send_request(Box::new(move |wm, window_target| {
            let window_id = wm.create_window_for(entity, window_target);

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
