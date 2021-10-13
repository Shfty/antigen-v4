use std::any::TypeId;

use reflection::data::Data;

use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Widget},
};

use crate::{DataWidget, LayoutIterator, ReflectionWidgetState, WidgetState};

pub enum StructValueSlot {}
pub enum StructDetailSlot {}

pub struct StructWidget<'a, 'b> {
    name: &'a str,
    fields: &'b mut Vec<(&'static str, Data)>,
}

impl<'a, 'b> StructWidget<'a, 'b> {
    pub fn new(name: &'a str, fields: &'b mut Vec<(&'static str, Data)>) -> Self {
        StructWidget { name, fields }
    }
}

const PADDING: u16 = 1;

// Struct / StructVariant implementation
impl<'a, 'b> WidgetState for StructWidget<'a, 'b> {
    fn wants_init_state(&mut self, state: &ReflectionWidgetState) -> bool {
        !matches!(state, ReflectionWidgetState::Struct { .. })
    }

    fn default_state(&mut self) -> ReflectionWidgetState {
        ReflectionWidgetState::Struct {
            selected: Default::default(),
            focused: false,
            focused_field: None,
            fields: self
                .fields
                .iter()
                .map(|(key, _)| (*key, ReflectionWidgetState::None))
                .collect(),
        }
    }
}

impl<'a, 'b> DataWidget for StructWidget<'a, 'b> {
    fn size_complex(
        &mut self,
        area: Rect,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        // Calculate widths
        let max_key_width = self
            .fields
            .iter()
            .fold(0, |acc, (next, _)| acc.max(next.len()));

        let max_value_width = self.fields.iter_mut().fold(0, |acc, (_, next)| {
            acc.max(
                predicate(next, TypeId::of::<StructValueSlot>())
                    .map(|mut next| next.size_complex(area, predicate))
                    .unwrap_or_default()
                    .0,
            )
        });

        let max_width =
            ((max_key_width as u16 + PADDING * 2) + (max_value_width as u16 + PADDING * 2) + 3)
                .max(self.name.len() as u16 + 2);

        (max_width, area.height.min(self.fields.len() as u16 + 2))
    }

    fn allocate_complex_impl(
        &mut self,
        builder: &mut crate::LayoutBuilder,
        state: &mut ReflectionWidgetState,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
        let (width, height) = self.size_complex(builder.area(), predicate);
        builder.allocate_size(width, height);

        let (selected, focused, state_fields) = if let ReflectionWidgetState::Struct {
            selected,
            focused,
            fields,
            ..
        } = state
        {
            (selected, focused, fields)
        } else {
            unreachable!()
        };

        if *focused {
            // Allocate complex value panel
            let (field_key, data) = self
                .fields
                .get_mut(*selected)
                .unwrap_or_else(|| panic!("No field for selected index {}", selected));

            if let Some(mut widget) = predicate(data, TypeId::of::<StructDetailSlot>()) {
                let (_, state_field) = state_fields
                    .iter_mut()
                    .find(|(key, _)| key == field_key)
                    .unwrap();

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

        let (selected, focused, state_fields) = if let ReflectionWidgetState::Struct {
            selected,
            focused,
            fields,
            ..
        } = state
        {
            (selected, focused, fields)
        } else {
            unreachable!()
        };

        let focused = *focused;
        let selected = *selected;

        if focused {
            // Draw complex value panel
            let (field_key, data) = self.fields.get_mut(selected).unwrap();
            if let Some(mut widget) = predicate(data, TypeId::of::<StructDetailSlot>()) {
                let (_, state) = state_fields
                    .iter_mut()
                    .find(|(key, _)| key == field_key)
                    .unwrap();
                widget.render_complex(layout, buf, state, predicate);
            }
        }

        if let Some(area) = area {
            // Calculate widths
            let max_key_width = self
                .fields
                .iter()
                .fold(0, |acc, (next, _)| acc.max(next.len()));
            let max_value_width = self.fields.iter_mut().fold(0, |acc, (_, next)| {
                acc.max(
                    predicate(next, TypeId::of::<StructValueSlot>())
                        .map(|mut next| next.size_complex(area, predicate))
                        .unwrap_or_default()
                        .0,
                )
            });

            let max_width =
                ((max_key_width as u16 + PADDING * 2) + (max_value_width as u16 + PADDING * 2) + 3)
                    .max(self.name.len() as u16 + 2);

            // Layout chunks
            let layout_chunks = tui::layout::Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Length(max_width), Constraint::Min(0)])
                .split(area);

            // Draw outer block
            let struct_block = Block::default().title(self.name).borders(Borders::ALL);
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
            let key_list_area = struct_chunks[0];
            let key_list_x = key_list_area.x + PADDING;
            let mut key_list_y = key_list_area.y;
            for (key, _) in self.fields.iter() {
                buf.set_span(key_list_x, key_list_y, &(*key).into(), key.len() as u16);
                key_list_y += 1;
                if key_list_y >= key_list_area.bottom() {
                    break;
                }
            }

            // Draw simple value list
            let simple_area = struct_chunks[1];
            let simple_block = Block::default().borders(Borders::LEFT);
            let simple_inner_area = simple_block.inner(simple_area);
            simple_block.render(simple_area, buf);

            let simple_list_x = simple_inner_area.x + PADDING;
            let mut simple_list_y = simple_inner_area.y;
            for (i, (field_key, value)) in self.fields.iter_mut().enumerate() {
                if let Some(mut widget) = predicate(value, TypeId::of::<StructValueSlot>()) {
                    let (_, widget_height) = widget.size_complex(simple_inner_area, predicate);

                    let item_area = Rect {
                        x: simple_list_x,
                        y: simple_list_y,
                        width: max_value_width as u16,
                        height: widget_height,
                    };

                    let (_, state) = state_fields
                        .iter_mut()
                        .find(|(key, _)| key == field_key)
                        .unwrap();

                    widget.render_complex(vec![Some(item_area)].into_iter(), buf, state, predicate);

                    if focused {
                        if selected == i {
                            buf.set_style(
                                item_area,
                                Style::default().add_modifier(Modifier::REVERSED),
                            );
                        }
                    }

                    simple_list_y += widget_height;
                    if simple_list_y >= simple_inner_area.bottom() {
                        break;
                    }
                }
            }
        }
    }
}
