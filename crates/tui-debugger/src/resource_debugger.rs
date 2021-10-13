use std::ops::Deref;

use reflection::data::Data;
use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{ListItem, ListState, StatefulWidget},
};

#[derive(Debug, Copy, Clone)]
pub enum ResourcesFocus {
    None,
    List,
}

impl Default for ResourcesFocus {
    fn default() -> Self {
        ResourcesFocus::None
    }
}

#[derive(Debug, Default, Clone)]
pub struct Resources(Option<Data>);

impl Resources {
    pub fn resources(&self) -> Option<&Data> {
        self.0.as_ref()
    }

    pub fn resources_mut(&mut self) -> Option<&mut Data> {
        self.0.as_mut()
    }

    pub fn parse_resources(&mut self, resources: &legion::Resources) {
        self.0 = Some(
            legion_debugger::serialize_resources(resources).expect("Failed to serialize resources"),
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct ResourceState {
    focus: ResourcesFocus,
    list_state: ListState,
    resource_count: usize,
}

impl ResourceState {
    pub fn set_focus(&mut self, focus: ResourcesFocus) {
        self.focus = focus;

        match self.focus {
            ResourcesFocus::None => {
                self.list_state.select(None);
            }
            ResourcesFocus::List => {
                if self.list_state.selected().is_none() {
                    self.list_state.select(Some(0));
                }
            }
        }
    }

    pub fn handle_input(&mut self, input: char) -> ResourcesFocus {
        match self.focus {
            ResourcesFocus::None => (),
            ResourcesFocus::List => match input {
                'h' => self.set_focus(ResourcesFocus::None),
                'j' => self.list_state.select(Some(
                    self.list_state
                        .selected()
                        .unwrap_or_default()
                        .wrapping_add(1)
                        .wrapping_rem(self.resource_count),
                )),
                'k' => self.list_state.select(Some(
                    self.list_state
                        .selected()
                        .unwrap_or_default()
                        .checked_sub(1)
                        .unwrap_or(self.resource_count - 1),
                )),
                _ => (),
            },
        }
        self.focus
    }
}

#[derive(Debug, Clone)]
pub struct ResourceDebugger<'a> {
    resources: &'a Resources,
}

impl<'a> ResourceDebugger<'a> {
    pub fn new(resources: &'a Resources) -> Self {
        Self { resources }
    }
}

impl<'a> StatefulWidget for ResourceDebugger<'a> {
    type State = ResourceState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        /*
        let mut resources = vec![];

        for resource in self.resources.iter() {
            resources.push(ListItem::new(format!(
                "{}: {:#?}",
                resource.name(),
                resource
            )));
        }

        super::style::list(resources, matches!(state.focus, ResourcesFocus::List))
            .block(super::style::block("Resources"))
            .render(area, buf, &mut state.list_state);
        */
    }
}
