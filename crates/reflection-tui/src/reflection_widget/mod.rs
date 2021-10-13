mod state;
use std::any::TypeId;

pub use state::*;

use reflection::data::Data;
use tui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

use crate::{DataWidget, LayoutBuilder, LayoutHorizontal};

/// [`tui::StatefulWidget`] implementor that can render a [`reflection::Data`] TUI
/// via the [`WidgetState`], [`DataWidget`] traits
pub struct ReflectionWidget<'a, 'b> {
    data: &'a mut Data,
    widget_predicate: &'b dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
}

impl<'a, 'b> ReflectionWidget<'a, 'b> {
    pub fn new(
        data: &'a mut Data,
        widget_predicate: &'b dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> Self {
        ReflectionWidget {
            data,
            widget_predicate,
        }
    }
}

impl<'a, 'b> StatefulWidget for ReflectionWidget<'a, 'b> {
    type State = ReflectionWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(mut widget) = (self.widget_predicate)(self.data, TypeId::of::<()>()) {
            let mut layout_builder = LayoutBuilder::new(area, LayoutHorizontal::default());
            widget.allocate_complex(&mut layout_builder, state, self.widget_predicate);

            let layout = layout_builder.build();
            widget.render_complex_impl(layout, buf, state, self.widget_predicate)
        }
    }
}
