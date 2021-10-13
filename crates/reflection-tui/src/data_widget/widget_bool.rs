use std::any::TypeId;

use reflection::data::Data;
use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use crate::{DataWidget, ReflectionWidgetState, WidgetState};

pub struct BoolWidget<'a>(&'a mut bool);

impl<'a> From<&'a mut bool> for BoolWidget<'a> {
    fn from(b: &'a mut bool) -> Self {
        BoolWidget(b)
    }
}

// bool implementation
impl WidgetState for BoolWidget<'_> {}

impl DataWidget for BoolWidget<'_> {
    fn size_complex(
        &mut self,
        area: Rect,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        (3.min(area.width), 1.min(area.height))
    }

    fn render_complex_impl(
        &mut self,
        mut layout: crate::LayoutIterator,
        buf: &mut Buffer,
        _state: &mut ReflectionWidgetState,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
        if let Some(area) = layout
            .next()
            .expect("Insufficient layout cells for widget list")
        {
            Paragraph::new(if *self.0 { "[✓]" } else { "[✗]" }).render(area, buf)
        }
    }
}
