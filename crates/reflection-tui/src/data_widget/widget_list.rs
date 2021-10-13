use std::any::TypeId;

use reflection::data::Data;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Widget},
};

use crate::{DataWidget, LayoutIterator, ReflectionWidgetState, WidgetState};

pub enum ListItemSlot {}
pub enum ListDetailSlot {}

pub struct ListWidget<'a, 'b> {
    name: &'a str,
    data: &'b mut Vec<Data>,
}

impl<'a, 'b> ListWidget<'a, 'b> {
    pub fn new(name: &'a str, data: &'b mut Vec<Data>) -> Self {
        ListWidget { name, data }
    }
}

const PADDING: u16 = 1;

impl<'a, 'b> WidgetState for ListWidget<'a, 'b> {
    fn wants_init_state(&mut self, state: &ReflectionWidgetState) -> bool {
        !matches!(state, ReflectionWidgetState::List { .. })
    }

    fn default_state(&mut self) -> ReflectionWidgetState {
        ReflectionWidgetState::List {
            selected: Default::default(),
            focused: false,
            focused_field: None,
            fields: self
                .data
                .iter()
                .map(|_| ReflectionWidgetState::None)
                .collect(),
        }
    }
}

impl<'a, 'b> DataWidget for ListWidget<'a, 'b> {
    fn size_complex(
        &mut self,
        area: Rect,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        let name_len = self.name.len() as u16;
        let max_width = self.data.iter_mut().fold(0, |acc, next| {
            acc.max(
                (name_len + 2).max(
                    predicate(next, TypeId::of::<ListItemSlot>())
                        .map(|mut next| next.size_complex(area, predicate))
                        .unwrap_or_default()
                        .0
                        + 2
                        + 2 * PADDING,
                ),
            )
        });
        (max_width, area.height.min(self.data.len() as u16 + 2))
    }

    fn allocate_complex_impl(
        &mut self,
        builder: &mut crate::LayoutBuilder,
        state: &mut ReflectionWidgetState,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
        let (width, height) = self.size_complex(builder.area(), predicate);
        builder.allocate_size(width, height);

        let (selected, focused, state_fields) = if let ReflectionWidgetState::List {
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
            let data = self
                .data
                .get_mut(*selected)
                .unwrap_or_else(|| panic!("No field for selected index {}", selected));

            if let Some(mut widget) = predicate(data, TypeId::of::<ListDetailSlot>()) {
                let state_field = state_fields.get_mut(*selected).unwrap();
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

        let (selected, focused, state_fields) = if let ReflectionWidgetState::List {
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

        let selected = *selected;
        let focused = *focused;

        if focused {
            // Draw complex value panel
            let data = self.data.get_mut(selected).unwrap();
            if let Some(mut widget) = predicate(data, TypeId::of::<ListDetailSlot>()) {
                let state = state_fields.get_mut(selected).unwrap();
                widget.render_complex(layout, buf, state, predicate);
            }
        }

        if let Some(area) = area {
            let name_len = self.name.len() as u16;
            let max_width = self.data.iter_mut().fold(0, |acc, next| {
                acc.max(
                    (name_len + 2).max(
                        predicate(next, TypeId::of::<ListItemSlot>())
                            .map(|mut next| next.size_complex(area, predicate))
                            .unwrap_or_default()
                            .0
                            + 2
                            + 2 * PADDING,
                    ),
                )
            });

            let block_area = Rect {
                width: max_width,
                ..area
            };

            let block = Block::default().title(self.name).borders(Borders::ALL);
            let mut inner_area = block.inner(block_area);
            block.render(block_area, buf);

            for (i, item) in self.data.iter_mut().enumerate() {
                if let Some(mut widget) = predicate(item, TypeId::of::<ListItemSlot>()) {
                    let (widget_width, widget_height) = widget.size_complex(inner_area, predicate);

                    let item_area = Rect {
                        x: inner_area.x + 1,
                        width: widget_width,
                        height: widget_height,
                        ..inner_area
                    };

                    widget.render_complex(vec![Some(item_area)].into_iter(), buf, state, predicate);

                    if focused {
                        if selected == i {
                            buf.set_style(
                                item_area,
                                Style::default().add_modifier(Modifier::REVERSED),
                            );
                        }
                    }

                    inner_area.y += widget_height;
                    inner_area.height = inner_area.height.saturating_sub(widget_height);
                    if inner_area.height == 0 {
                        break;
                    }
                }
            }
        }
    }
}
