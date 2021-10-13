use legion_debugger::Archetypes;

use tui_widgets::TabContainer;

use tui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Style,
    text::Spans,
    widgets::{Row, StatefulWidget, TableState, Widget},
};

#[derive(Debug, Copy, Clone)]
pub enum ArchetypesFocus {
    None,
    Tabs,
    Table,
}

impl Default for ArchetypesFocus {
    fn default() -> Self {
        ArchetypesFocus::None
    }
}

#[derive(Debug, Default, Clone)]
pub struct ArchetypeState {
    focus: ArchetypesFocus,
    table_state: TableState,
    active_table: usize,
    archetype_count: usize,
    entity_count: usize,
}

impl ArchetypeState {
    pub fn set_focus(&mut self, focus: ArchetypesFocus) {
        self.focus = focus;
    }

    pub fn handle_input(&mut self, input: char) -> ArchetypesFocus {
        match self.focus {
            ArchetypesFocus::None => (),
            ArchetypesFocus::Tabs => match input {
                'h' => {
                    self.active_table = self
                        .active_table
                        .checked_sub(1)
                        .unwrap_or(self.archetype_count - 1)
                }
                'l' => {
                    self.active_table = self
                        .active_table
                        .wrapping_add(1)
                        .wrapping_rem(self.archetype_count)
                }
                'j' => {
                    self.table_state.select(Some(0));
                    self.focus = ArchetypesFocus::Table;
                }
                'k' => self.focus = ArchetypesFocus::None,
                _ => (),
            },
            ArchetypesFocus::Table => match input {
                'h' => {
                    self.table_state.select(None);
                    self.focus = ArchetypesFocus::Tabs;
                }
                'j' => self.table_state.select(Some(
                    self.table_state
                        .selected()
                        .unwrap_or_default()
                        .wrapping_add(1)
                        .wrapping_rem(self.entity_count),
                )),
                'k' => self.table_state.select(Some(
                    self.table_state
                        .selected()
                        .unwrap_or_default()
                        .checked_sub(1)
                        .unwrap_or(self.entity_count),
                )),
                _ => (),
            },
        }

        self.focus
    }
}

pub struct ArchetypeDebugger<'a> {
    archetypes: &'a Archetypes,
}

impl<'a> ArchetypeDebugger<'a> {
    pub fn new(archetypes: &'a Archetypes) -> Self {
        Self { archetypes }
    }
}

impl<'a> StatefulWidget for ArchetypeDebugger<'a> {
    type State = ArchetypeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        /*
        TabContainer::new(super::style::tabs)
            .titles(
                self.archetypes
                    .iter()
                    .map(|archetype| {
                        archetype
                            .components()
                            .keys()
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .map(Spans::from)
                    .collect::<Vec<_>>(),
            )
            .block(super::style::block("Archetypes"))
            .select(state.active_table)
            .highlight(matches!(state.focus, ArchetypesFocus::Tabs))
            .next(|area, buf, index| {
                if self.archetypes.is_empty() {
                    return;
                }

                let archetype = &self.archetypes[index];
                let entities = archetype.entities();
                let components = archetype.components();

                let header = std::iter::once(String::new())
                    .chain(components.keys().cloned())
                    .collect::<Vec<_>>();

                let column_count = components.len();
                let table_constraints = std::iter::once(Constraint::Min(12))
                    .chain(
                        std::iter::repeat(Constraint::Ratio(1, column_count as u32))
                            .take(column_count),
                    )
                    .collect::<Vec<_>>();

                let mut rows = entities
                    .iter()
                    .map(|entity| vec![format!("{:?}", entity,)])
                    .collect::<Vec<_>>();

                for (i, row) in rows.iter_mut().enumerate() {
                    for column in components.values() {
                        row.push(format!("{:#?}", column[i]))
                    }
                }

                let rows = rows.into_iter().map(|row| {
                    let height = row
                        .iter()
                        .map(|item| item.matches('\n').count())
                        .max()
                        .expect("No row items")
                        + 1;

                    Row::new(row).height(height as u16)
                });

                let header_row = Row::new(header)
                    .style(Style::default().fg(super::style::COLOR_INFO))
                    .bottom_margin(1);

                let table =
                    super::style::table(rows, matches!(state.focus, ArchetypesFocus::Table))
                        .header(header_row)
                        .block(super::style::block(
                            components
                                .keys()
                                .map(String::as_str)
                                .collect::<Vec<_>>()
                                .join(", "),
                        ))
                        .widths(&*table_constraints)
                        .column_spacing(1);

                StatefulWidget::render(table, area, buf, &mut state.table_state);
            })
            .render(area, buf);
            */
    }
}
