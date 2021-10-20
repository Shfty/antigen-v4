use std::{
    any::TypeId,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crossbeam_channel::Receiver;
use crossterm::event::Event;
use reflection::data::Data;
use reflection_tui::{standard_widgets, DataWidget, ReflectionWidget, ReflectionWidgetState};
use tui_debugger::TuiDebugger;

use crate::{
    resources::CrosstermEventQueue,
    spin_loop,
    systems::{crossterm_input_buffer_clear, crossterm_input_buffer_fill},
    Shared,
};

const TUI_TICK_HZ: f64 = 60.0;
const TUI_TICK_SECS: f64 = 1.0 / TUI_TICK_HZ;

#[profiling::function]
pub fn tui_render_thread(
    shared_state: Shared,
    crossterm_rx: Receiver<Event>,
    main_loop_break: Arc<AtomicBool>,
) -> impl FnOnce() {
    let mut tui_debugger = TuiDebugger::start().unwrap();
    let mut crossterm_event_queue = CrosstermEventQueue::default();
    let mut reflection_widget_state = ReflectionWidgetState::None;

    spin_loop(Duration::from_secs_f64(TUI_TICK_SECS), move || {
        crossterm_input_buffer_fill(&crossterm_rx, &mut crossterm_event_queue);
        for event in crossterm_event_queue.iter() {
            reflection_widget_state.handle_input(event);
        }
        crossterm_input_buffer_clear(&mut crossterm_event_queue);

        let archetypes = shared_state.trace_archetypes.read();
        let entities = shared_state.trace_entities.read();
        let trace_resources = shared_state.trace_resources.read();

        if let (Some(archetypes), Some(entities), Some(resources)) = (
            archetypes.archetypes(),
            entities.entities(),
            trace_resources.resources(),
        ) {
            let mut debugger_data = Data::Struct {
                name: "Legion Debugger",
                fields: vec![
                    ("Archetypes", archetypes.clone()),
                    ("Entities", entities.clone()),
                    ("Resources", resources.clone()),
                ],
            };

            tui_debugger
                .terminal()
                .draw(|f| {
                    f.render_stateful_widget(
                        ReflectionWidget::new(&mut debugger_data, &widget_rules),
                        f.size(),
                        &mut reflection_widget_state,
                    )
                })
                .unwrap();
        }

        main_loop_break.load(Ordering::Relaxed)
    })
}

pub fn widget_rules(data: &mut Data, parent_type: TypeId) -> Option<Box<dyn DataWidget + '_>> {
    standard_widgets(&widget_rules)(data, parent_type)
}
