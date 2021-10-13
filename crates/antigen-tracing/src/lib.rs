mod fields;
mod parent_id;
mod span_id;
mod trace_event;
mod trace_field;
mod trace_leaf;
mod trace_leaf_id;
mod trace_receiver;
mod trace_sender;
mod trace_thread;
mod trace_tree;
mod trace_visitor;

use std::sync::atomic::{AtomicUsize, Ordering};

pub use fields::*;
pub use parent_id::*;
pub use span_id::*;
pub use trace_event::*;
pub use trace_field::*;
pub use trace_leaf::*;
pub use trace_leaf_id::*;
pub use trace_receiver::*;
pub use trace_sender::*;
pub use trace_thread::*;
pub use trace_tree::*;
pub use trace_visitor::*;

/// Atomic incrementing counter for tracking unique trace leaf IDs program-wide
static NEXT_TRACE_LEAF_ID: AtomicUsize = AtomicUsize::new(0);

pub fn next_trace_leaf_id() -> TraceLeafId {
    NEXT_TRACE_LEAF_ID.fetch_add(1, Ordering::Relaxed).into()
}

