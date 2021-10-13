mod widget_bool;
mod widget_byte_array;
mod widget_char;
mod widget_float;
mod widget_int;
mod widget_list;
mod widget_map;
mod widget_newtype;
mod widget_option;
mod widget_string;
mod widget_struct;
mod widget_unit;

use std::any::TypeId;

use reflection::data::Data;
use tui::buffer::Buffer;
use tui::layout::Rect;

use crate::{LayoutBuilder, LayoutIterator, ReflectionWidgetState};

pub use widget_bool::*;
pub use widget_byte_array::*;
pub use widget_char::*;
pub use widget_float::*;
pub use widget_int::*;
pub use widget_list::*;
pub use widget_map::*;
pub use widget_newtype::*;
pub use widget_option::*;
pub use widget_string::*;
pub use widget_struct::*;
pub use widget_unit::*;

pub trait WidgetState {
    fn wants_init_state(&mut self, _state: &ReflectionWidgetState) -> bool {
        false
    }

    fn default_state(&mut self) -> ReflectionWidgetState {
        ReflectionWidgetState::None
    }

    fn try_init_state(&mut self, state: &mut ReflectionWidgetState) {
        if self.wants_init_state(state) {
            *state = self.default_state()
        }
    }
}

pub type BoxedDataWidget<'a> = Box<dyn DataWidget + 'a>;

/// A dynamically sized widget
pub trait DataWidget: WidgetState {
    fn size_complex(
        &mut self,
        _area: Rect,
        _predicate: &dyn Fn(&mut Data, TypeId) -> Option<BoxedDataWidget<'_>>,
    ) -> (u16, u16) {
        (0, 1)
    }

    fn allocate_complex(
        &mut self,
        builder: &mut LayoutBuilder,
        state: &mut ReflectionWidgetState,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<BoxedDataWidget<'_>>,
    ) {
        self.try_init_state(state);
        self.allocate_complex_impl(builder, state, predicate)
    }

    fn allocate_complex_impl(
        &mut self,
        builder: &mut LayoutBuilder,
        _state: &mut ReflectionWidgetState,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<BoxedDataWidget<'_>>,
    ) {
        let (width, height) = self.size_complex(builder.area(), predicate);
        builder.allocate_size(width, height)
    }

    fn render_complex(
        &mut self,
        layout: LayoutIterator,
        buf: &mut Buffer,
        state: &mut ReflectionWidgetState,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<BoxedDataWidget<'_>>,
    ) {
        self.try_init_state(state);
        self.render_complex_impl(layout, buf, state, predicate)
    }

    fn render_complex_impl(
        &mut self,
        layout: LayoutIterator,
        buf: &mut Buffer,
        state: &mut ReflectionWidgetState,
        predicate: &dyn Fn(&mut Data, TypeId) -> Option<BoxedDataWidget<'_>>,
    );
}

pub fn standard_widgets(
    predicate: &dyn Fn(&mut Data, TypeId) -> Option<BoxedDataWidget<'_>>,
) -> impl Fn(&mut Data, TypeId) -> Option<BoxedDataWidget<'_>> + '_ {
    move |data: &mut Data, parent_type: TypeId| {
        if parent_type == TypeId::of::<ListItemSlot>()
            || parent_type == TypeId::of::<StructValueSlot>()
            || parent_type == TypeId::of::<MapKeySlot>()
            || parent_type == TypeId::of::<MapValueSlot>()
            || parent_type == TypeId::of::<NewtypeSlot>()
            || parent_type == TypeId::of::<OptionSlot>()
        {
            simple_widgets(data, parent_type)
        } else {
            detail_widgets(predicate)(data, parent_type)
        }
    }
}

pub fn detail_widgets(
    predicate: &dyn Fn(&mut Data, TypeId) -> Option<BoxedDataWidget<'_>>,
) -> impl Fn(&mut Data, TypeId) -> Option<BoxedDataWidget<'_>> + '_ {
    move |data: &mut Data, parent_type: TypeId| match data {
        Data::ByteArray(v) => Some(Box::new(ByteArrayWidget::from(v))),
        Data::Option(v) => match v {
            Some(v) => predicate(v, parent_type),
            None => None,
        },
        Data::NewtypeStruct { data, .. } => predicate(data, parent_type),
        Data::NewtypeVariant { data, .. } => predicate(data, parent_type),
        Data::Seq(v) => Some(Box::new(ListWidget::new("Sequence", v))),
        Data::Tuple(v) => Some(Box::new(ListWidget::new("Tuple", v))),
        Data::TupleStruct { name, data } => Some(Box::new(ListWidget::new(*name, data))),
        Data::TupleVariant { variant, data, .. } => Some(Box::new(ListWidget::new(variant, data))),
        Data::Map(v) => Some(Box::new(MapWidget::from(v))),
        Data::Struct { name, fields } => Some(Box::new(StructWidget::new(*name, fields))),
        Data::StructVariant {
            variant, fields, ..
        } => Some(Box::new(StructWidget::new(variant, fields))),
        _ => None,
    }
}

pub fn simple_widgets(data: &mut Data, _parent_type: TypeId) -> Option<BoxedDataWidget<'_>> {
    match data {
        Data::Bool(v) => Some(Box::new(BoolWidget::from(v))),
        Data::I8(v) => Some(Box::new(IntWidget::from(v))),
        Data::I16(v) => Some(Box::new(IntWidget::from(v))),
        Data::I32(v) => Some(Box::new(IntWidget::from(v))),
        Data::I64(v) => Some(Box::new(IntWidget::from(v))),
        Data::I128(v) => Some(Box::new(IntWidget::from(v))),
        Data::U8(v) => Some(Box::new(IntWidget::from(v))),
        Data::U16(v) => Some(Box::new(IntWidget::from(v))),
        Data::U32(v) => Some(Box::new(IntWidget::from(v))),
        Data::U64(v) => Some(Box::new(IntWidget::from(v))),
        Data::U128(v) => Some(Box::new(IntWidget::from(v))),
        Data::F32(v) => Some(Box::new(FloatWidget::from(v))),
        Data::F64(v) => Some(Box::new(FloatWidget::from(v))),
        Data::Char(v) => Some(Box::new(CharWidget::from(v))),
        Data::String(v) => Some(Box::new(StringWidget::from(v))),
        Data::ByteArray(_) => Some(Box::new(StrWidget::from("ByteArray[]"))),
        Data::Option(v) => match v {
            Some(v) => Some(Box::new(NewtypeWidget::new("Some", None, v))),
            None => Some(Box::new(StrWidget::from("None"))),
        },
        Data::Unit => Some(Box::new(StrWidget::from("()"))),
        Data::UnitStruct { name } => Some(Box::new(StrWidget::from(*name))),
        Data::UnitVariant { name, variant, .. } => {
            Some(Box::new(VariantWidget::new(*name, *variant)))
        }
        Data::NewtypeStruct { name, data } => Some(Box::new(NewtypeWidget::new(*name, None, data))),
        Data::NewtypeVariant {
            name,
            variant,
            data,
            ..
        } => Some(Box::new(NewtypeWidget::new(*name, Some(variant), data))),
        Data::Seq(_) => Some(Box::new(StrWidget::from("Sequence[]"))),
        Data::Tuple(_) => Some(Box::new(StrWidget::from("Tuple[]"))),
        Data::TupleStruct { name, .. } => Some(Box::new(StrWidget::from(*name))),
        Data::TupleVariant { name, variant, .. } => {
            Some(Box::new(VariantWidget::new(*name, *variant)))
        }
        Data::Map(_) => Some(Box::new(StrWidget::from("Map"))),
        Data::Struct { name, .. } => Some(Box::new(StrWidget::from(*name))),
        Data::StructVariant { name, variant, .. } => {
            Some(Box::new(VariantWidget::new(*name, *variant)))
        }
    }
}
