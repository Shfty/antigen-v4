use tracing::{Event, span::{Attributes, Record}};

use super::{TraceFields, TraceVisitor};

pub trait Fields {
    fn tracing_fields(&self) -> TraceFields;
}

impl Fields for Attributes<'_> {
    fn tracing_fields(&self) -> TraceFields {
        let mut visitor = TraceVisitor::default();
        self.record(&mut visitor);
        visitor.into_inner()
    }
}

impl Fields for Event<'_> {
    fn tracing_fields(&self) -> TraceFields {
        let mut visitor = TraceVisitor::default();
        self.record(&mut visitor);
        visitor.into_inner()
    }
}

impl Fields for Record<'_> {
    fn tracing_fields(&self) -> TraceFields {
        let mut visitor = TraceVisitor::default();
        self.record(&mut visitor);
        visitor.into_inner()
    }
}

