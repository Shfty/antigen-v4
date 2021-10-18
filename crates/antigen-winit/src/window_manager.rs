use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use legion::{Entity, World};
use winit::{
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowId},
};

use crate::{WindowState, components::{RedrawMode, WindowComponent}};

#[derive(Debug, Default)]
pub struct WindowManager {
    windows: HashMap<Entity, Arc<Window>>,
    entity_ids: HashMap<WindowId, Entity>,
    window_redraw_modes: HashMap<WindowId, RedrawMode>,
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
    ) -> Arc<Window> {
        let window = Arc::new(Window::new(window_target).unwrap());
        let window_id = window.id();
        self.windows.insert(entity, window.clone());
        self.entity_ids.insert(window_id, entity);
        window
    }

    pub fn window_redraw_modes(&self) -> impl Iterator<Item = (&WindowId, &RedrawMode)> {
        self.window_redraw_modes.iter()
    }

    pub fn set_window_redraw_mode(&mut self, window_id: WindowId, redraw_mode: RedrawMode) {
        self.window_redraw_modes.insert(window_id, redraw_mode);
    }

    pub fn close_window(&mut self, world: &mut World, window_id: &WindowId) {
        let entity = *self.entity_ids.get(window_id).unwrap();
        self.windows
            .remove(&entity)
            .unwrap_or_else(|| panic!("No window with ID {:?}", entity));

        self.window_redraw_modes.remove(window_id);

        self.entity_ids.remove(window_id);

        if let Some(mut entry) = world.entry(entity) {
            if let Ok(WindowComponent(window_state)) = entry.get_component_mut::<WindowComponent>() {
                *window_state = WindowState::Closed
            }
        }
    }
}
