use tracing::field::{Field, Visit};

use crate::{TraceField, TraceFields};

#[derive(Default)]
pub struct TraceVisitor(TraceFields);

impl TraceVisitor {
    pub fn into_inner(self) -> TraceFields {
        self.0
    }
}

impl Visit for TraceVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name(), TraceField::Debug(format!("{:?}", value)));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name(), TraceField::I64(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name(), TraceField::U64(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name(), TraceField::Bool(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0
            .insert(field.name(), TraceField::Str(value.to_string()));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.0
            .insert(field.name(), TraceField::Error(value.to_string()));
    }
}

