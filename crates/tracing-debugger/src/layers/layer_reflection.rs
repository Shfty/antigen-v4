use std::sync::Arc;

use tracing::{Event, Subscriber};
use tracing_core::span::{Attributes, Id, Record};
use tracing_subscriber::layer::{Context, Layer};

use crate::{Trace, TraceTree};

pub struct ReflectionLayer {
    trace_tree: Arc<TraceTree>,
}

impl ReflectionLayer {
    pub fn new(trace_tree: Arc<TraceTree>) -> Self {
        ReflectionLayer { trace_tree }
    }
}

impl<S: Subscriber> Layer<S> for ReflectionLayer {
    fn new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span_id = id.into_u64();
        let trace_id = self.trace_tree.next_trace_id(Some(span_id));

        let mut parent_span_id = None;
        if attrs.is_contextual() {
            let current = ctx.current_span();
            if let Some((id, _)) = current.into_inner() {
                parent_span_id = Some(id.into_u64());
            }
        } else if let Some(parent_id) = attrs.parent() {
            parent_span_id = Some(parent_id.into_u64());
        }

        let parent_trace_id = parent_span_id.map(|id| self.trace_tree.span_to_trace_id(&id));

        self.trace_tree
            .traces()
            .write()
            .insert(trace_id, Trace::span(attrs.metadata(), parent_trace_id));

        if let Some(parent_trace_id) = parent_trace_id {
            self.trace_tree
                .traces()
                .write()
                .get_mut(&parent_trace_id)
                .unwrap()
                .insert_child(trace_id);
        }
    }

    fn on_record(&self, id: &Id, record: &Record<'_>, _ctx: Context<'_, S>) {
        let span_id = id.into_u64();
        let trace_id = self.trace_tree.span_to_trace_id(&span_id);

        let mut traces = self.trace_tree.traces().write();
        let trace = traces.get_mut(&trace_id).unwrap();
        record.record(trace);
    }

    fn on_follows_from(&self, _span: &Id, _follows: &Id, _ctx: Context<'_, S>) {
        panic!("Follows-from annotations are unimplemented");
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let trace_id = self.trace_tree.next_trace_id(None);

        let mut trace_tree = self.trace_tree.traces().write();
        let metadata = event.metadata();

        let mut parent_span_id = None;
        if event.is_contextual() {
            let current = ctx.current_span();
            if let Some((id, _)) = current.into_inner() {
                parent_span_id = Some(id.into_u64());
            }
        } else if let Some(parent_id) = event.parent() {
            parent_span_id = Some(parent_id.into_u64());
        }

        let parent_trace_id = parent_span_id.map(|id| self.trace_tree.span_to_trace_id(&id));
        let mut trace = Trace::event(metadata, parent_trace_id);
        event.record(&mut trace);
        trace_tree.insert(trace_id, trace);

        if let Some(parent_trace_id) = parent_trace_id {
            trace_tree
                .get_mut(&parent_trace_id)
                .unwrap()
                .insert_child(trace_id);
        }
    }

    fn on_enter(&self, id: &Id, _ctx: Context<'_, S>) {
        let span_id = id.into_u64();
        let trace_id = self.trace_tree.span_to_trace_id(&span_id);

        let mut traces = self.trace_tree.traces().write();
        let span = traces.get_mut(&trace_id).unwrap();

        span.enter();
    }

    fn on_exit(&self, id: &Id, _ctx: Context<'_, S>) {
        let span_id = id.into_u64();
        let trace_id = self.trace_tree.span_to_trace_id(&span_id);

        let mut traces = self.trace_tree.traces().write();
        let span = traces.get_mut(&trace_id).unwrap();

        span.exit();
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        let span_id = id.into_u64();
        let trace_id = self.trace_tree.span_to_trace_id(&span_id);

        let mut traces = self.trace_tree.traces().write();
        let span = traces.get_mut(&trace_id).unwrap();

        span.close();
    }
}
