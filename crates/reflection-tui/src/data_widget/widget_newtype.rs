use std::any::TypeId;

use reflection::data::Data;
use tui::{buffer::Buffer, layout::Rect};

use crate::{DataWidget, LayoutIterator, ReflectionWidgetState, WidgetState};

pub enum NewtypeSlot {}

pub struct NewtypeWidget<'a, 'b, 'c> {
    name: &'a str,
    variant: Option<&'b str>,
    data: &'c mut Data,
}

impl<'a, 'b, 'c> NewtypeWidget<'a, 'b, 'c> {
    pub fn new(name: &'a str, variant: Option<&'b str>, data: &'c mut Data) -> Self {
        NewtypeWidget {
            name,
            variant,
            data,
        }
    }
}

impl<'a, 'b, 'c> WidgetState for NewtypeWidget<'a, 'b, 'c> {}

impl<'a, 'b, 'c> DataWidget for NewtypeWidget<'a, 'b, 'c> {
    fn size_complex(
        &mut self,
        area: Rect,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) -> (u16, u16) {
        let name_len = self.name.len() as u16;
        let variant_len = match self.variant {
            Some(v) => v.len() as u16 + 2,
            None => 0,
        };

        if let Some(mut widget) = predicate(self.data, TypeId::of::<NewtypeSlot>()) {
            let (width, height) = widget.size_complex(area, predicate);

            (name_len + variant_len + 1 + width + 1, height)
        } else {
            (name_len + variant_len + 2, 1)
        }
    }

    fn render_complex_impl(
        &mut self,
        mut layout: LayoutIterator,
        buf: &mut Buffer,
        state: &mut ReflectionWidgetState,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<Box<dyn DataWidget + '_>>,
    ) {
        if let Some(mut area) = layout.next().unwrap() {
            buf.set_span(area.x, area.y, &self.name.into(), self.name.len() as u16);
            area.x += self.name.len() as u16;

            if let Some(variant) = self.variant {
                buf.set_span(area.x, area.y, &"::".into(), 2);
                area.x += 2;

                buf.set_span(area.x, area.y, &variant.into(), variant.len() as u16);
                area.x += variant.len() as u16;
            }

            buf.set_span(area.x, area.y, &"(".into(), 1);
            area.x += 1;

            if let Some(mut widget) = predicate(self.data, TypeId::of::<NewtypeSlot>()) {
                let size = widget.size_complex(area, predicate);

                widget.render_complex(vec![Some(area)].into_iter(), buf, state, predicate);
                area.x += size.0;
            }

            buf.set_span(area.x, area.y, &")".into(), 1);
        }
    }
}
