use std::collections::{BTreeMap, BTreeSet, HashMap};

use tracing::{Metadata, callsite::Identifier};

use crate::{SpanId, TraceLeaf, TraceLeafId, TraceLeafVariant};

/// Collected profiling data
#[derive(Debug, Default)]
pub struct TraceTree {
    pub leaves: BTreeMap<TraceLeafId, TraceLeaf>,
    pub span_id_to_trace_leaf_id: BTreeMap<SpanId, TraceLeafId>,
    pub generation: usize,
}

impl TraceTree {
    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn increment_generation(&mut self) {
        self.generation += 1
    }

    pub fn trace_leaf_id(&self, span_id: &SpanId) -> Result<&TraceLeafId, String> {
        self.span_id_to_trace_leaf_id
            .get(span_id)
            .ok_or_else(|| format!("No such trace leaf ID exists for span ID {:?}", span_id))
    }

    pub fn try_get(&self, trace_leaf_id: &TraceLeafId) -> Result<&TraceLeaf, String> {
        self.leaves
            .get(trace_leaf_id)
            .ok_or_else(|| format!("No trace leaf for ID {:?}", trace_leaf_id))
    }

    pub fn try_get_mut(&mut self, trace_leaf_id: &TraceLeafId) -> Result<&mut TraceLeaf, String> {
        self.leaves
            .get_mut(trace_leaf_id)
            .ok_or_else(|| format!("No trace leaf for ID {:?}", trace_leaf_id))
    }

    pub fn try_get_by_span_id(&self, span_id: &SpanId) -> Result<&TraceLeaf, String> {
        let trace_leaf_id = self.trace_leaf_id(span_id)?;
        self.try_get(trace_leaf_id)
    }

    pub fn try_get_mut_by_span_id(&mut self, span_id: &SpanId) -> Result<&mut TraceLeaf, String> {
        let trace_leaf_id = *self.trace_leaf_id(span_id)?;
        self.try_get_mut(&trace_leaf_id)
    }

    pub fn roots(&self) -> impl Iterator<Item = (&TraceLeafId, &TraceLeaf)> {
        self.leaves
            .iter()
            .filter(|(_, leaf)| matches!(leaf.parent_id, None))
    }

    pub fn children<'a>(
        &'a self,
        id: &'a TraceLeafId,
    ) -> impl Iterator<Item = (&TraceLeafId, &TraceLeaf)> {
        self.leaves
            .iter()
            .filter(move |(_, leaf)| leaf.parent_id == Some(*id))
    }

    pub fn callsites(&self) -> impl Iterator<Item = (Identifier, &'static Metadata<'static>)> {
        self.leaves
            .iter()
            .map(|(_, leaf)| (leaf.metadata.callsite(), leaf.metadata))
            .collect::<HashMap<Identifier, &'static Metadata<'static>>>()
            .into_iter()
    }

    pub fn callsite_leaves<'a>(
        &'a self,
        callsite: Identifier,
    ) -> impl Iterator<Item = (&TraceLeafId, &TraceLeaf)> {
        self.leaves
            .iter()
            .filter(move |(_, leaf)| leaf.metadata.callsite() == callsite)
    }

    pub fn prune_closed(&mut self) {
        self.prune_by(|_, leaf| matches!(leaf.variant, TraceLeafVariant::Span { open: false, .. }))
    }

    pub fn prune_previous_generations(&mut self, skip: usize) {
        let current_generation = self.generation - (1 + skip);
        self.prune_by(|_, leaf| leaf.generation < current_generation)
    }

    pub fn prune_by<F>(&mut self, f: F)
    where
        F: Fn(&TraceLeafId, &TraceLeaf) -> bool,
    {
        // Gather closed leaf IDs
        let mut pruned_leaves = BTreeSet::default();
        for (id, leaf) in self.leaves.iter() {
            if f(id, leaf) {
                pruned_leaves.insert(*id);
            }
        }

        // Gather child leaf IDs for pruned spans
        loop {
            let mut done = true;
            for (id, leaf) in self.leaves.iter() {
                if let Some(parent_id) = leaf.parent_id {
                    if pruned_leaves.contains(&parent_id) && !pruned_leaves.contains(id) {
                        pruned_leaves.insert(*id);
                        done = false;
                    }
                }
            }

            if done {
                break;
            }
        }

        // Remove pruned leaves and their child events
        for id in pruned_leaves.iter().chain(pruned_leaves.iter()) {
            self.leaves.remove(id);
        }

        // Remove closed span IDs from lookup table
        for id in pruned_leaves.iter().copied() {
            if let Some((span_id, _)) = self
                .span_id_to_trace_leaf_id
                .iter()
                .find(|(_span_id, leaf_id)| **leaf_id == id)
            {
                let span_id = *span_id;
                self.span_id_to_trace_leaf_id.remove(&span_id);
            }
        }
    }
}

