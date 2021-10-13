use std::any::TypeId;

use reflection::data::Data;

use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Widget},
};

use crate::{DataWidget, LayoutIterator, ReflectionWidgetState, WidgetState};

pub enum MapKeySlot {}
pub enum MapValueSlot {}
pub enum MapDetailSlot {}

pub struct MapWidget<'a>(&'a mut Vec<(Data, Data)>);

impl<'a> From<&'a mut Vec<(Data, Data)>> for MapWidget<'a> {
    fn from(v: &'a mut Vec<(Data, Data)>) -> Self {
        MapWidget(v)
    }
}

const PADDING: u16 = 1;

// Struct / StructVariant implementation
impl WidgetState for MapWidget<'_> {
    fn wants_init_state(&mut self, state: &ReflectionWidgetState) -> bool {
        !matches!(state, ReflectionWidgetState::Map { .. })
    }

    fn default_state(&mut self) -> ReflectionWidgetState {
        ReflectionWidgetState::Map {
            column: Default::default(),
            row: Default::default(),
            focused: false,
            focused_field: None,
            fields: self
                .0
                .iter()
                .map(|(_, _)| (ReflectionWidgetState::None, ReflectionWidgetState::None))
                .collect(),
        }
    }
}

impl DataWidget for MapWidget<'_> {
    fn size_complex(
        &mut self,
        area: Rect,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        // Calculate widths
        let max_key_width = self.0.iter_mut().fold(0, |acc, (next, _)| {
            acc.max(
                predicate(next, TypeId::of::<MapKeySlot>())
                    .map(|mut next| next.size_complex(area, predicate))
                    .unwrap_or_default()
                    .0,
            )
        });

        let max_value_width = self.0.iter_mut().fold(0, |acc, (_, next)| {
            acc.max(
                predicate(next, TypeId::of::<MapValueSlot>())
                    .map(|mut next| next.size_complex(area, predicate))
                    .unwrap_or_default()
                    .0,
            )
        });

        let max_width =
            (max_key_width as u16 + PADDING * 2) + (max_value_width as u16 + PADDING * 2) + 3;

        (max_width, area.height.min(self.0.len() as u16 + 2))
    }

    fn allocate_complex_impl(
        &mut self,
        builder: &mut crate::LayoutBuilder,
        state: &mut ReflectionWidgetState,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
        let (width, height) = self.size_complex(builder.area(), predicate);
        builder.allocate_size(width, height);

        let (row, column, focused, state_fields) = if let ReflectionWidgetState::Map {
            row,
            column,
            focused,
            fields,
            ..
        } = state
        {
            (row, column, focused, fields)
        } else {
            unreachable!()
        };

        if *focused {
            // Allocate complex value panel
            let data = self
                .0
                .get_mut(*row)
                .unwrap_or_else(|| panic!("No field for selected index {}", row));

            let data = if *column == 0 {
                &mut data.0
            } else {
                &mut data.1
            };

            if let Some(mut widget) = predicate(data, TypeId::of::<MapDetailSlot>()) {
                let state_field = state_fields.get_mut(*row).unwrap();
                let state_field = if *column == 0 {
                    &mut state_field.0
                } else {
                    &mut state_field.1
                };
                widget.allocate_complex(builder, state_field, predicate);
            }
        }
    }

    fn render_complex_impl(
        &mut self,
        mut layout: LayoutIterator,
        buf: &mut Buffer,
        state: &mut ReflectionWidgetState,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
        let area = layout
            .next()
            .expect("Insufficient layout cells for widget list");

        let name = "Map";
        let fields = &mut *self.0;

        let (column, row, focused, state_fields) = if let ReflectionWidgetState::Map {
            column,
            row,
            focused,
            fields,
            ..
        } = state
        {
            (column, row, focused, fields)
        } else {
            unreachable!()
        };

        if *row == usize::MAX {
            *row = fields.len() - 1
        } else if *row >= fields.len() {
            *row = 0
        }

        let column = *column;
        let row = *row;
        let focused = *focused;

        if focused {
            // Draw complex value panel
            let data = fields.get_mut(row).unwrap();

            let data = if column == 0 {
                &mut data.0
            } else {
                &mut data.1
            };

            if let Some(mut widget) = predicate(data, TypeId::of::<MapDetailSlot>()) {
                let state = state_fields.get_mut(row).unwrap();
                let state = if column == 0 {
                    &mut state.0
                } else {
                    &mut state.1
                };
                widget.render_complex(layout, buf, state, predicate);
            }
        }

        let area = if let Some(area) = area { area } else { return };

        // Calculate widths
        let max_key_width = fields.iter_mut().fold(0, |acc, (next, _)| {
            acc.max(
                predicate(next, TypeId::of::<MapKeySlot>())
                    .map(|mut next| next.size_complex(area, predicate))
                    .unwrap_or_default()
                    .0,
            )
        });

        let max_value_width = fields.iter_mut().fold(0, |acc, (_, next)| {
            acc.max(
                predicate(next, TypeId::of::<MapValueSlot>())
                    .map(|mut next| next.size_complex(area, predicate))
                    .unwrap_or_default()
                    .0,
            )
        });

        // Layout chunks
        let layout_chunks = tui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(
                    ((max_key_width as u16 + PADDING * 2)
                        + (max_value_width as u16 + PADDING * 2)
                        + 3)
                    .max(name.len() as u16 + 2),
                ),
                Constraint::Min(0),
            ])
            .split(area);

        // Draw outer block
        let struct_block = Block::default().title(name).borders(Borders::ALL);
        let struct_inner_area = struct_block.inner(layout_chunks[0]);
        struct_block.render(layout_chunks[0], buf);

        // Struct chunks
        let struct_chunks = tui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(max_key_width as u16 + PADDING * 2),
                Constraint::Min(0),
            ])
            .split(struct_inner_area);

        // Draw key list
        let simple_area = struct_chunks[0];

        let simple_list_x = simple_area.x + PADDING;
        let mut simple_list_y = simple_area.y;
        for (i, (value, _)) in fields.iter_mut().enumerate() {
            if let Some(mut widget) = predicate(value, TypeId::of::<MapKeySlot>()) {
                let (_, widget_height) = widget.size_complex(simple_area, predicate);

                let item_area = Rect {
                    x: simple_list_x,
                    y: simple_list_y,
                    width: max_key_width as u16,
                    height: widget_height,
                };

                widget.render_complex(vec![Some(item_area)].into_iter(), buf, state, predicate);

                if focused {
                    if column == 0 && row == i {
                        buf.set_style(item_area, Style::default().add_modifier(Modifier::REVERSED));
                    }
                }

                simple_list_y += widget_height;
            }
        }

        // Draw value list
        let simple_area = struct_chunks[1];
        let simple_block = Block::default().borders(Borders::LEFT);
        let simple_inner_area = simple_block.inner(simple_area);
        simple_block.render(simple_area, buf);

        let simple_list_x = simple_inner_area.x + PADDING;
        let mut simple_list_y = simple_inner_area.y;
        for (i, (_, value)) in fields.iter_mut().enumerate() {
            if let Some(mut widget) = predicate(value, TypeId::of::<MapValueSlot>()) {
                let (_, widget_height) = widget.size_complex(simple_inner_area, predicate);

                let item_area = Rect {
                    x: simple_list_x,
                    y: simple_list_y,
                    width: max_value_width as u16,
                    height: widget_height,
                };

                widget.render_complex(vec![Some(item_area)].into_iter(), buf, state, predicate);

                if focused {
                    if column == 1 && row == i {
                        buf.set_style(item_area, Style::default().add_modifier(Modifier::REVERSED));
                    }
                }

                simple_list_y += widget_height;
            }
        }
    }
}
