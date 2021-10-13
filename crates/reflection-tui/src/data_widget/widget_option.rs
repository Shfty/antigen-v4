use std::any::TypeId;

use reflection::data::Data;
use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use crate::{DataWidget, LayoutIterator, ReflectionWidgetState, WidgetState};

pub enum OptionSlot {}

pub struct OptionWidget<'a>(&'a mut Option<Box<Data>>);

impl<'a> From<&'a mut Option<Box<Data>>> for OptionWidget<'a> {
    fn from(v: &'a mut Option<Box<Data>>) -> Self {
        OptionWidget(v)
    }
}

impl WidgetState for OptionWidget<'_> {}

impl DataWidget for OptionWidget<'_> {
    fn size_complex(
        &mut self,
        area: Rect,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        match self.0 {
            Some(v) => {
                if let Some(mut widget) = predicate(v, TypeId::of::<OptionSlot>()) {
                    let (width, height) = widget.size_complex(area, predicate);
                    (width + 6, height)
                } else {
                    (6, 1)
                }
            }
            None => (4, 1),
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

        if let Some(mut area) = area {
            match self.0 {
                Some(data) => {
                    if let Some(mut widget) = predicate(data, TypeId::of::<OptionSlot>()) {
                        let size = widget.size_complex(area, predicate);

                        buf.set_span(area.x, area.y, &"Some(".into(), 5);

                        area.x += 5;

                        widget.render_complex(vec![Some(area)].into_iter(), buf, state, predicate);

                        buf.set_span(area.x + size.0, area.y, &")".into(), 1);
                    }
                }
                None => Paragraph::new("None").render(area, buf),
            }
        }
    }
}
