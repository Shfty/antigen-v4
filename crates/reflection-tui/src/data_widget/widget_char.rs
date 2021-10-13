use std::any::TypeId;

use reflection::data::Data;
use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use crate::{DataWidget, ReflectionWidgetState, WidgetState};

pub struct CharWidget<'a>(&'a mut char);

impl<'a> From<&'a mut char> for CharWidget<'a> {
    fn from(v: &'a mut char) -> Self {
        CharWidget(v)
    }
}

impl WidgetState for CharWidget<'_> {}

impl DataWidget for CharWidget<'_>
{
    fn size_complex(
        &mut self,
        area: Rect,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        (1.min(area.width), 1.min(area.height))
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
            Paragraph::new(self.0.to_string()).render(area, buf)
        }
    }
}
