use std::any::TypeId;

use reflection::data::Data;
use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use crate::{DataWidget, LayoutIterator, ReflectionWidgetState, WidgetState};

pub struct StringWidget<'a>(&'a mut String);

impl<'a> From<&'a mut String> for StringWidget<'a> {
    fn from(v: &'a mut String) -> Self {
        StringWidget(v)
    }
}

impl WidgetState for StringWidget<'_> {}

impl DataWidget for StringWidget<'_> {
    fn size_complex(
        &mut self,
        area: Rect,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        ((self.0.len() as u16).min(area.width), 1.min(area.height))
    }

    fn render_complex_impl(
        &mut self,
        mut layout: LayoutIterator,
        buf: &mut Buffer,
        _state: &mut ReflectionWidgetState,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
        if let Some(area) = layout.next().unwrap() {
            Paragraph::new(self.0.as_str()).render(area, buf)
        }
    }
}

pub struct StrWidget<'a>(&'a str);

impl<'a> From<&'a str> for StrWidget<'a> {
    fn from(v: &'a str) -> Self {
        StrWidget(v)
    }
}

impl WidgetState for StrWidget<'_> {}

impl DataWidget for StrWidget<'_> {
    fn size_complex(
        &mut self,
        area: Rect,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        ((self.0.len() as u16).min(area.width), 1.min(area.height))
    }

    fn render_complex_impl(
        &mut self,
        mut layout: LayoutIterator,
        buf: &mut Buffer,
        _state: &mut ReflectionWidgetState,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
        if let Some(area) = layout.next().unwrap() {
            Paragraph::new(self.0).render(area, buf)
        }
    }
}

pub struct VariantWidget<'a>(&'a str, &'a str);

impl<'a> VariantWidget<'a> {
    pub fn new(name: &'a str, variant: &'a str) -> Self {
        VariantWidget(name, variant)
    }
}

impl WidgetState for VariantWidget<'_> {}

impl DataWidget for VariantWidget<'_> {
    fn size_complex(
        &mut self,
        _area: Rect,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        (self.0.len() as u16 + 2 + self.1.len() as u16, 1)
    }

    fn render_complex_impl(
        &mut self,
        mut layout: LayoutIterator,
        buf: &mut Buffer,
        _state: &mut ReflectionWidgetState,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
        if let Some(area) = layout.next().unwrap() {
            Paragraph::new(self.0.to_string() + "::" + self.1).render(area, buf)
        }
    }
}
