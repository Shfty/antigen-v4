mod archetype_debugger;
mod entity_debugger;
mod resource_debugger;
mod schedule_debugger;

mod style;

pub use archetype_debugger::*;
pub use entity_debugger::*;
pub use resource_debugger::*;
pub use schedule_debugger::*;

use std::io::Stdout;

use tui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    text::Spans,
    widgets::{StatefulWidget, Widget},
    Terminal,
};

use reflection::data::Data;

use legion_debugger::{Archetypes, Entities, ParseArchetypesError, ParseEntitiesError};

use tui_widgets::TabContainer;

fn parse_world_data(world: Data) -> (Data, Data) {
    // Top-le,vel structure is a Map
    let mut map = if let Data::Map(map) = world {
        map
    } else {
        panic!("World is not a map: {:?}", world);
    };

    // Map contains a single field
    map.pop().expect("No data in top-level map")
}

#[derive(Debug, Copy, Clone)]
pub enum DebuggerFocus {
    RootTabs,
    Archetypes,
    Entities,
    Resources,
    Schedules,
}

impl Default for DebuggerFocus {
    fn default() -> Self {
        DebuggerFocus::RootTabs
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RootTabs {
    Archetype,
    Entity,
    Resource,
    Schedule,
}

impl Default for RootTabs {
    fn default() -> Self {
        RootTabs::Archetype
    }
}

#[derive(Debug, Clone)]
pub enum ParseWorldError {
    ParseArchetypes(ParseArchetypesError),
    ParseEntities(ParseEntitiesError),
}

impl From<ParseArchetypesError> for ParseWorldError {
    fn from(e: ParseArchetypesError) -> Self {
        ParseWorldError::ParseArchetypes(e)
    }
}

impl From<ParseEntitiesError> for ParseWorldError {
    fn from(e: ParseEntitiesError) -> Self {
        ParseWorldError::ParseEntities(e)
    }
}

#[derive(Debug, Clone)]
pub enum ParseResourcesError {}

#[derive(Debug, Clone)]
pub enum ParseTracingError {}

/// Internal state for debug UI
#[derive(Debug, Default, Clone)]
pub struct TuiDebuggerState {
    active_tab: RootTabs,
    focus: DebuggerFocus,
    archetype_state: ArchetypeState,
    entity_state: EntityState,
    resource_state: ResourceState,
    schedule_state: ScheduleState,
}

impl TuiDebuggerState {
    pub fn handle_input(&mut self, input: char) {
        match self.focus {
            DebuggerFocus::RootTabs => match input {
                'h' => self.prev_tab(),
                'l' => self.next_tab(),
                'j' => match self.active_tab {
                    RootTabs::Archetype => {
                        self.archetype_state.set_focus(ArchetypesFocus::Tabs);
                        self.focus = DebuggerFocus::Archetypes;
                    }
                    RootTabs::Entity => {
                        self.entity_state.set_focus(EntitiesFocus::Entities);
                        self.focus = DebuggerFocus::Entities;
                    }
                    RootTabs::Resource => {
                        self.resource_state.set_focus(ResourcesFocus::List);
                        self.focus = DebuggerFocus::Resources;
                    }
                    RootTabs::Schedule => {
                        self.schedule_state.set_focus(SchedulesFocus::List);
                        self.focus = DebuggerFocus::Schedules;
                    }
                },
                _ => (),
            },
            DebuggerFocus::Archetypes => {
                let new_focus = self.archetype_state.handle_input(input);
                if let ArchetypesFocus::None = new_focus {
                    self.focus = DebuggerFocus::RootTabs;
                }
            }
            DebuggerFocus::Entities => {
                let new_focus = self.entity_state.handle_input(input);
                if let EntitiesFocus::None = new_focus {
                    self.focus = DebuggerFocus::RootTabs;
                }
            }
            DebuggerFocus::Resources => {
                let new_focus = self.resource_state.handle_input(input);
                if let ResourcesFocus::None = new_focus {
                    self.focus = DebuggerFocus::RootTabs;
                }
            }
            DebuggerFocus::Schedules => {
                let new_focus = self.schedule_state.handle_input(input);
                if let SchedulesFocus::None = new_focus {
                    self.focus = DebuggerFocus::RootTabs;
                }
            }
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            RootTabs::Archetype => RootTabs::Entity,
            RootTabs::Entity => RootTabs::Resource,
            RootTabs::Resource => RootTabs::Schedule,
            RootTabs::Schedule => RootTabs::Archetype,
        }
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            RootTabs::Archetype => RootTabs::Schedule,
            RootTabs::Entity => RootTabs::Archetype,
            RootTabs::Resource => RootTabs::Entity,
            RootTabs::Schedule => RootTabs::Resource,
        }
    }
}

/// RAII state object for TUI debugging
///
/// Will configure terminal and become ready for rendering when allocated,
/// run shutdown routines when dropped.
pub struct TuiDebugger {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TuiDebugger {
    pub fn start() -> std::io::Result<Self> {
        // Add a panic hook wrapper to disable TUI before print
        // (This prevents loss of panic data printed to the alternate screen)
        let panic_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            Self::stop();
            panic_hook(info);
        }));

        crossterm::terminal::enable_raw_mode().unwrap();

        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )
        .unwrap();

        let backend = tui::backend::CrosstermBackend::new(std::io::stdout());
        let mut terminal = tui::Terminal::new(backend).unwrap();

        terminal.clear().unwrap();

        crossterm::execute!(std::io::stdout(), crossterm::cursor::Hide).unwrap();

        Ok(TuiDebugger { terminal })
    }

    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }

    pub fn draw(
        &mut self,
        state: &mut TuiDebuggerState,
        archetypes: &Archetypes,
        entities: &Entities,
        resources: &Resources,
    ) {
        self.terminal
            .draw(|f| {
                f.render_stateful_widget(
                    TuiDebuggerWidget::new(archetypes, entities, resources),
                    f.size(),
                    state,
                );
            })
            .unwrap();
    }

    fn stop() {
        crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )
        .unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
    }
}

impl Drop for TuiDebugger {
    fn drop(&mut self) {
        Self::stop();
    }
}

/// StatefulWidget implementation for rendering via tui
#[derive(Debug, Clone)]
struct TuiDebuggerWidget<'a> {
    archetypes: &'a Archetypes,
    entities: &'a Entities,
    resources: &'a Resources,
}

impl<'a> TuiDebuggerWidget<'a> {
    pub fn new(
        archetypes: &'a Archetypes,
        entities: &'a Entities,
        resources: &'a Resources,
    ) -> Self {
        Self {
            archetypes,
            entities,
            resources,
        }
    }
}

impl<'a> StatefulWidget for TuiDebuggerWidget<'a> {
    type State = TuiDebuggerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let archetypes = self.archetypes;
        let entities = self.entities;
        let resources = self.resources;

        TabContainer::new(style::tabs)
            .titles(
                std::array::IntoIter::new(["Archetypes", "Entities", "Resources", "Tracing"])
                    .map(Spans::from)
                    .collect(),
            )
            .block(style::block("Legion Debugger"))
            .select(match state.active_tab {
                RootTabs::Archetype => 0,
                RootTabs::Entity => 1,
                RootTabs::Resource => 2,
                RootTabs::Schedule => 3,
            })
            .highlight(matches!(state.focus, DebuggerFocus::RootTabs))
            .next(move |area, buf, index| match index {
                0 => {
                    ArchetypeDebugger::new(archetypes).render(area, buf, &mut state.archetype_state)
                }
                1 => EntityDebugger::new(entities).render(area, buf, &mut state.entity_state),
                2 => ResourceDebugger::new(resources).render(area, buf, &mut state.resource_state),
                _ => (),
            })
            .render(area, buf)
    }
}
