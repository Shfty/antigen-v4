use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use crate::{DataWidget, ReflectionWidgetState, WidgetState};

pub struct ByteArrayWidget<'a>(&'a mut Vec<u8>);

impl<'a> From<&'a mut Vec<u8>> for ByteArrayWidget<'a> {
    fn from(v: &'a mut Vec<u8>) -> Self {
        ByteArrayWidget(v)
    }
}

impl WidgetState for ByteArrayWidget<'_> {}

impl DataWidget for ByteArrayWidget<'_> {
    fn size_complex(
        &mut self,
        area: Rect,
        _predicate: &dyn Fn(&mut reflection::data::Data, std::any::TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        (
            (format!("{:?}", self.0).len() as u16).min(area.width),
            1.min(area.height),
        )
    }

    fn render_complex_impl(
        &mut self,
        mut layout: crate::LayoutIterator,
        buf: &mut Buffer,
        _state: &mut ReflectionWidgetState,
        _predicate: &dyn Fn(&mut reflection::data::Data, std::any::TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
        if let Some(area) = layout
            .next()
            .expect("Insufficient layout cells for widget list")
        {
            Paragraph::new(format!("{:?}", self.0)).render(area, buf)
        }
    }
}
