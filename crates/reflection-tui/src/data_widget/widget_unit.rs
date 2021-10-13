use std::any::TypeId;

use reflection::data::Data;
use tui::{buffer::Buffer, layout::Rect};

use crate::{DataWidget, LayoutIterator, ReflectionWidgetState, WidgetState};

pub struct UnitWidget;

impl WidgetState for UnitWidget {}

impl DataWidget for UnitWidget {
    fn size_complex(
        &mut self,
        _area: Rect,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        (0, 0)
    }

    fn render_complex_impl(
        &mut self,
        _layout: LayoutIterator,
        _buf: &mut Buffer,
        _state: &mut ReflectionWidgetState,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
    }
}
