use legion_debugger::{Archetypes, Entities};
use parking_lot::RwLock;
use std::sync::Arc;

use crossterm::event::KeyCode;
use legion::{system, Resources as LegionResources, World};

use tui_debugger::{Resources as TuiDebuggerResources, TuiDebugger, TuiDebuggerState};

use crate::resources::CrosstermEventQueue;

#[profiling::function]
pub fn tui_debugger_handle_input(
    events: &CrosstermEventQueue,
    tui_debugger_state: &mut TuiDebuggerState,
) {
    for event in events.iter() {
        if let crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: KeyCode::Char(code),
            ..
        }) = event
        {
            tui_debugger_state.handle_input(*code)
        }
    }
}

#[profiling::function]
pub fn tui_debugger_parse_archetypes_thread_local() -> impl FnMut(&mut World, &mut LegionResources)
{
    let (world_serializer, entity_serializer) = legion_debugger::world_serializers();

    move |world, resources| {
        resources
            .get_mut::<Arc<RwLock<Archetypes>>>()
            .unwrap()
            .write()
            .parse_archetypes(world, &legion::any(), &world_serializer, &entity_serializer)
    }
}

#[profiling::function]
pub fn tui_debugger_parse_entities_thread_local() -> impl FnMut(&mut World, &mut LegionResources) {
    let (world_serializer, entity_serializer) = legion_debugger::world_serializers();

    move |world, resources| {
        resources
            .get_mut::<Arc<RwLock<Entities>>>()
            .unwrap()
            .write()
            .parse_entities(world, &legion::any(), &world_serializer, &entity_serializer)
    }
}

#[profiling::function]
pub fn tui_debugger_parse_resources_thread_local() -> impl FnMut(&mut World, &mut LegionResources) {
    move |_world, resources| {
        resources
            .get_mut::<Arc<RwLock<TuiDebuggerResources>>>()
            .unwrap()
            .write()
            .parse_resources(resources)
    }
}

#[system]
#[profiling::function]
pub fn tui_debugger_draw(
    #[resource] tui_debugger_state: &mut TuiDebuggerState,
    #[resource] tui_debugger: &mut TuiDebugger,
    #[resource] archetypes: &Archetypes,
    #[resource] entities: &Entities,
    #[resource] resources: &TuiDebuggerResources,
) {
    tui_debugger.draw(tui_debugger_state, archetypes, entities, resources);
}
