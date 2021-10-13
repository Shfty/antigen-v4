use crossbeam_channel::Receiver;

use crate::TraceLeaf;

use super::{TraceEvent, TraceTree, TraceEventVariant, TraceLeafVariant, next_trace_leaf_id};

/// Receives [`TraceEvent`]s from a [`TraceSender`] and uses it to maintain a [`TraceTree`]
pub struct TraceReceiver {
    receiver: Receiver<TraceEvent>,
}

impl TraceReceiver {
    pub fn new(receiver: Receiver<TraceEvent>) -> Self {
        Self { receiver }
    }

    pub fn flush(&self, trace_tree: &mut TraceTree) {
        while let Ok(TraceEvent { thread, variant }) = self.receiver.try_recv() {
            match variant {
                TraceEventVariant::NewSpan {
                    id,
                    parent_id,
                    metadata,
                    fields,
                } => {
                    let trace_leaf_id = next_trace_leaf_id();
                    trace_tree
                        .span_id_to_trace_leaf_id
                        .insert(id, trace_leaf_id);

                    let parent_trace_leaf_id = parent_id
                        .as_ref()
                        .map(|span_id| trace_tree.span_id_to_trace_leaf_id.get(span_id).copied())
                        .flatten();

                    trace_tree.leaves.insert(
                        trace_leaf_id,
                        TraceLeaf {
                            generation: trace_tree.generation(),
                            thread,
                            parent_id: parent_trace_leaf_id,
                            metadata,
                            fields,
                            variant: TraceLeafVariant::Span {
                                open: true,
                                entered: None,
                                exited: None,
                                follows: Default::default(),
                            },
                        },
                    );
                }
                TraceEventVariant::Record { id, fields } => {
                    trace_tree
                        .try_get_mut_by_span_id(&id)
                        .unwrap()
                        .fields
                        .extend(fields.into_iter());
                }
                TraceEventVariant::FollowsFrom { id, follows } => {
                    let follows_trace_leaf_id = *trace_tree.trace_leaf_id(&follows).unwrap();
                    let trace_leaf = trace_tree.try_get_mut_by_span_id(&id).unwrap();

                    if let TraceLeafVariant::Span { follows, .. } = &mut trace_leaf.variant {
                        follows.push(follows_trace_leaf_id)
                    } else {
                        panic!("TraceLeaf with span ID {:?} has a follows-from annotation, but is not a Span", id);
                    }
                }
                TraceEventVariant::Event {
                    parent_id,
                    metadata,
                    fields,
                    instant,
                } => {
                    let trace_leaf_id = next_trace_leaf_id();
                    let parent_trace_leaf_id = parent_id
                        .as_ref()
                        .map(|span_id| trace_tree.span_id_to_trace_leaf_id.get(span_id).copied())
                        .flatten();

                    trace_tree.leaves.insert(
                        trace_leaf_id,
                        TraceLeaf {
                            generation: trace_tree.generation(),
                            thread,
                            parent_id: parent_trace_leaf_id,
                            metadata,
                            fields,
                            variant: TraceLeafVariant::Event { instant },
                        },
                    );
                }
                TraceEventVariant::Enter { id, instant } => {
                    let trace_leaf = trace_tree.try_get_mut_by_span_id(&id).unwrap();
                    if let TraceLeafVariant::Span { entered, .. } = &mut trace_leaf.variant {
                        if let None = entered {
                            *entered = Some(instant);
                        } else {
                            panic!("TraceLeaf with ID {:?} was entered twice", id);
                        }
                    } else {
                        panic!(
                            "TraceLeaf with ID {:?} was entered, but is not a Span variant",
                            id
                        );
                    }
                }
                TraceEventVariant::Exit { id, instant } => {
                    let trace_leaf = trace_tree.try_get_mut_by_span_id(&id).unwrap();
                    if let TraceLeafVariant::Span {
                        entered, exited, ..
                    } = &mut trace_leaf.variant
                    {
                        if let None = entered {
                            panic!(
                                "TraceLeaf with SpanId {:?} was exited before being entered",
                                id
                            );
                        }

                        if let None = exited {
                            *exited = Some(instant);
                        } else {
                            panic!("TraceLeaf with ID {:?} was exited twice", id);
                        }
                    } else {
                        panic!(
                            "TraceLeaf with ID {:?} was exited, but is not a Span variant",
                            id
                        );
                    }
                }
                TraceEventVariant::Close { id } => {
                    let trace_leaf = trace_tree.try_get_mut_by_span_id(&id).unwrap();
                    if let TraceLeafVariant::Span { open, .. } = &mut trace_leaf.variant {
                        if *open {
                            *open = false;
                        } else {
                            panic!("TraceLeaf with span ID {:?} was closed twice", id)
                        }
                    } else {
                        panic!(
                            "TraceLeaf with span ID {:?} was exited, but is not a Span variant",
                            id
                        );
                    }
                }
            }
        }

        trace_tree.increment_generation()
    }
}

