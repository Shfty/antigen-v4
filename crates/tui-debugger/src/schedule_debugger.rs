use tui::{
    text::Text,
    widgets::{ListItem, ListState, StatefulWidget},
};

#[derive(Debug, Copy, Clone)]
pub enum SchedulesFocus {
    None,
    List,
}

impl Default for SchedulesFocus {
    fn default() -> Self {
        SchedulesFocus::None
    }
}

#[derive(Debug, Default, Clone)]
pub struct ScheduleState {
    focus: SchedulesFocus,
    list_state: ListState,
    label_count: usize,
}


impl ScheduleState {
    pub fn set_focus(&mut self, focus: SchedulesFocus) {
        self.focus = focus;

        match self.focus {
            SchedulesFocus::None => (),
            SchedulesFocus::List => self.list_state.select(Some(0)),
        }
    }

    pub fn handle_input(&mut self, input: char) -> SchedulesFocus {
        match self.focus {
            SchedulesFocus::None => (),
            SchedulesFocus::List => match input {
                'h' => {
                    self.set_focus(SchedulesFocus::None);
                }
                'j' => self.list_state.select(Some(
                    self.list_state
                        .selected()
                        .unwrap_or_default()
                        .wrapping_add(1)
                        .wrapping_rem(self.label_count),
                )),
                'k' => self.list_state.select(Some(
                    self.list_state
                        .selected()
                        .unwrap_or_default()
                        .checked_sub(1)
                        .unwrap_or(self.label_count - 1),
                )),
                _ => (),
            },
        }

        self.focus
    }
}

pub struct ScheduleDebugger;

impl StatefulWidget for ScheduleDebugger {
    type State = ScheduleState;

    fn render(
        self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        let mut labels = vec![];

        super::style::list(labels, matches!(state.focus, SchedulesFocus::List))
            .block(super::style::block("Tracing"))
            .render(area, buf, &mut state.list_state);
    }
}
