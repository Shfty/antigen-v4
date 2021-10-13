use std::{
    collections::{BTreeMap, BTreeSet},
    thread::Thread,
    time::{Duration, Instant},
};

use tracing::{Metadata, field::Visit};

#[derive(Debug, Clone)]
pub enum TraceField {
    Debug(String),
    I64(i64),
    U64(u64),
    Bool(bool),
    Str(String),
    Error(String)
}

#[derive(Debug, Clone)]
pub struct Trace {
    thread: Thread,
    metadata: &'static Metadata<'static>,
    fields: BTreeMap<&'static str, TraceField>,
    parent: Option<usize>,
    variant: TraceVariant,
}

#[derive(Debug, Clone)]
pub enum TraceVariant {
    Event {
        occurred: Instant,
    },
    Span {
        children: BTreeSet<usize>,

        open: bool,
        entered: Option<Instant>,
        exited: Option<Instant>,
    },
}

impl Trace {
    pub fn span(metadata: &'static Metadata<'static>, parent: Option<usize>) -> Self {
        Trace {
            thread: std::thread::current(),
            metadata,
            fields: Default::default(),
            parent,
            variant: TraceVariant::Span {
                children: Default::default(),

                open: true,
                entered: None,
                exited: None,
            },
        }
    }

    pub fn event(metadata: &'static Metadata<'static>, parent: Option<usize>) -> Self {
        Trace {
            thread: std::thread::current(),
            metadata,
            fields: Default::default(),
            parent,
            variant: TraceVariant::Event {
                occurred: Instant::now(),
            },
        }
    }

    pub fn clear(&mut self) {
        self.fields.clear();

        if let TraceVariant::Span { children, .. } = &mut self.variant {
            children.clear();
        }
    }

    pub fn thread(&self) -> &Thread {
        &self.thread
    }

    pub fn metadata(&self) -> &'static Metadata<'static> {
        self.metadata
    }

    pub fn occurred(&self) -> &Instant {
        if let TraceVariant::Event { occurred, .. } = &self.variant {
            occurred
        }
        else {
            panic!("TraceVariant is not an Event");
        }
    }

    pub fn duration(&self) -> Duration {
        if let TraceVariant::Span { entered, exited, .. } = &self.variant {
            match (entered, exited) {
                (None, None) => Default::default(),
                (None, Some(_)) => unreachable!(),
                (Some(entered), None) => Instant::now().duration_since(*entered),
                (Some(entered), Some(exited)) => exited.duration_since(*entered),
            }
        }
        else {
            panic!("TraceVariant is not a Span");
        }
    }

    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    pub fn children(&self) -> &BTreeSet<usize> {
        if let TraceVariant::Span { children, .. } = &self.variant {
            &children
        }
        else {
            panic!("TraceVariant is not a Span");
        }
    }

    pub fn insert_child(&mut self, child: usize) -> bool {
        if let TraceVariant::Span { children, .. } = &mut self.variant {
            children.insert(child)
        }
        else {
            panic!("TraceVariant is not a Span")
        }
    }

    pub fn remove_child(&mut self, child: &usize) -> bool {
        if let TraceVariant::Span { children, .. } = &mut self.variant {
            children.remove(child)
        }
        else {
            panic!("TraceVariant is not a Span")
        }
    }

    pub fn is_open(&self) -> bool {
        if let TraceVariant::Span { open, .. } = &self.variant {
            *open
        }
        else {
            panic!("TraceVariant is not a Span")
        }
    }

    pub fn fields(&self) -> &BTreeMap<&'static str, TraceField> {
        &self.fields
    }

    pub fn enter(&mut self) {
        if let TraceVariant::Span { entered, .. } = &mut self.variant {
            *entered = Some(Instant::now())
        }
        else {
            panic!("TraceVariant is not a Span")
        }
    }

    pub fn exit(&mut self) {
        if let TraceVariant::Span { exited, .. } = &mut self.variant {
            *exited = Some(Instant::now())
        }
        else {
            panic!("TraceVariant is not a Span")
        }
    }

    pub fn close(&mut self) {
        if let TraceVariant::Span { open, .. } = &mut self.variant {
            *open = false
        }
        else {
            panic!("TraceVariant is not a Span")
        }
    }

    pub fn variant(&self) -> &TraceVariant {
        &self.variant
    }

    pub fn variant_mut(&mut self) -> &mut TraceVariant {
        &mut self.variant
    }
}

impl Visit for Trace {
    fn record_debug(&mut self, field: &tracing_core::Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(field.name(), TraceField::Debug(format!("{:?}", value)));
    }

    fn record_i64(&mut self, field: &tracing_core::Field, value: i64) {
        self.fields.insert(field.name(), TraceField::I64(value));
    }

    fn record_u64(&mut self, field: &tracing_core::Field, value: u64) {
        self.fields.insert(field.name(), TraceField::U64(value));
    }

    fn record_bool(&mut self, field: &tracing_core::Field, value: bool) {
        self.fields.insert(field.name(), TraceField::Bool(value));
    }

    fn record_str(&mut self, field: &tracing_core::Field, value: &str) {
        self.fields.insert(field.name(), TraceField::Str(value.to_string()));
    }

    fn record_error(&mut self, field: &tracing_core::Field, value: &(dyn std::error::Error + 'static)) {
        self.fields.insert(field.name(), TraceField::Error(value.to_string()));
    }
}
