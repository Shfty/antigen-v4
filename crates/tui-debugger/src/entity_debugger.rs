use legion::{
    query::EntityFilter,
    serialize::{CustomEntitySerializer, WorldSerializer},
    World,
};

use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{ListItem, ListState, StatefulWidget},
};

use legion_debugger::{Entities, ParseEntitiesError};
use reflection::data::Data;

use super::parse_world_data;

#[derive(Debug, Copy, Clone)]
pub enum EntitiesFocus {
    None,
    Entities,
    Components,
}

impl Default for EntitiesFocus {
    fn default() -> Self {
        EntitiesFocus::None
    }
}

#[derive(Debug, Default, Clone)]
pub struct EntityState {
    focus: EntitiesFocus,
    entity_list_state: ListState,
    component_list_state: ListState,
    entity_count: usize,
}

impl EntityState {
    pub fn set_focus(&mut self, focus: EntitiesFocus) {
        self.focus = focus;

        match self.focus {
            EntitiesFocus::None => (),
            EntitiesFocus::Entities => {
                if self.entity_list_state.selected().is_none() {
                    self.entity_list_state.select(Some(0));
                }
                self.component_list_state.select(None);
            }
            EntitiesFocus::Components => {
                self.component_list_state.select(Some(0));
            }
        }
    }

    pub fn handle_input(&mut self, input: char) -> EntitiesFocus {
        match self.focus {
            EntitiesFocus::None => (),
            EntitiesFocus::Entities => match input {
                'h' => {
                    self.set_focus(EntitiesFocus::None);
                }
                'j' => self.entity_list_state.select(Some(
                    self.entity_list_state
                        .selected()
                        .unwrap_or_default()
                        .wrapping_add(1)
                        .wrapping_rem(self.entity_count),
                )),
                'k' => self.entity_list_state.select(Some(
                    self.entity_list_state
                        .selected()
                        .unwrap_or_default()
                        .checked_sub(1)
                        .unwrap_or(self.entity_count - 1),
                )),
                'l' => {
                    self.set_focus(EntitiesFocus::Components);
                }
                _ => (),
            },
            EntitiesFocus::Components => match input {
                'h' => {
                    self.set_focus(EntitiesFocus::Entities);
                }
                'j' => self.component_list_state.select(Some(
                    self.component_list_state
                        .selected()
                        .unwrap_or_default()
                        .wrapping_add(1)
                        .wrapping_rem(self.entity_count),
                )),
                'k' => self.component_list_state.select(Some(
                    self.component_list_state
                        .selected()
                        .unwrap_or_default()
                        .checked_sub(1)
                        .unwrap_or(self.entity_count - 1),
                )),
                _ => (),
            },
        }

        self.focus
    }
}

pub struct EntityDebugger<'a> {
    entities: &'a Entities,
}

impl<'a> EntityDebugger<'a> {
    pub fn new(entities: &'a Entities) -> Self {
        EntityDebugger { entities }
    }
}

impl<'a> StatefulWidget for EntityDebugger<'a> {
    type State = EntityState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        /*
        let mut entities = vec![];

        for (key, _) in self.entities.iter() {
            entities.push(ListItem::new(format!("{:?}", key)));
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Min(16), Constraint::Ratio(1, 1)])
            .split(area);

        super::style::list(entities, matches!(state.focus, EntitiesFocus::Entities))
            .block(super::style::block("Entities"))
            .render(chunks[0], buf, &mut state.entity_list_state);

        let mut components = vec![];

        if let Some(selected) = state.entity_list_state.selected() {
            let (_, entity_components) = &self.entities[selected];
            for (key, value) in entity_components.iter() {
                components.push(ListItem::new(format!("{}: {:#?}", key, value)));
            }
        }

        super::style::list(components, matches!(state.focus, EntitiesFocus::Components))
            .block(super::style::block("Components"))
            .render(chunks[1], buf, &mut state.component_list_state);
        */
    }
}
