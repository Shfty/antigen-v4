use parking_lot::RwLock;
use std::{collections::BTreeMap, sync::atomic::AtomicUsize};

use crate::TraceVariant;

use super::Trace;

#[derive(Debug, Default)]
pub struct TraceTree {
    traces: RwLock<BTreeMap<usize, Trace>>,
    next_trace_id: AtomicUsize,
    span_to_trace_id: RwLock<BTreeMap<u64, usize>>,
}

impl TraceTree {
    pub fn next_trace_id(&self, span_id: Option<u64>) -> usize {
        let trace_id = self
            .next_trace_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut span_to_trace_id = self.span_to_trace_id.write();

        if let Some(span_id) = span_id {
            span_to_trace_id.insert(span_id, trace_id);
        }

        trace_id
    }

    pub fn span_to_trace_id(&self, span_id: &u64) -> usize {
        self.span_to_trace_id.read().get(span_id).copied().unwrap()
    }

    pub fn traces(&self) -> &RwLock<BTreeMap<usize, Trace>> {
        &self.traces
    }

    pub fn prune_closed(&self) {
        let mut traces = self.traces.write();

        // Gather closed span IDs
        let mut remove_ids = vec![];
        for (id, span) in traces
            .iter()
            .filter(|(_id, span)| matches!(span.variant(), TraceVariant::Span { .. }))
            .rev()
        {
            if !span.is_open() {
                remove_ids.push(*id);
            }
        }

        for id in remove_ids.into_iter() {
            // Remove trace from set
            let removed = traces.remove(&id).unwrap();

            // Remove child events
            for child in removed.children() {
                let child_trace = traces.get(child).unwrap();
                if matches!(child_trace.variant(), TraceVariant::Event { .. }) {
                    traces.remove(child);
                }
            }

            // Remove from parent's children array
            if let Some(parent_id) = removed.parent() {
                traces.get_mut(&parent_id).unwrap().remove_child(&id);
            }

            // Remove from span-to-trace lookup table
            let mut span_to_trace_id = self.span_to_trace_id.write();
            let span_id = span_to_trace_id
                .iter()
                .find_map(|(span_id, trace_id)| {
                    if *trace_id == id {
                        Some(*span_id)
                    } else {
                        None
                    }
                })
                .unwrap();
            span_to_trace_id.remove(&span_id);
        }
    }
}
