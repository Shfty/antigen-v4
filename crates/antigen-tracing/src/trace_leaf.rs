use std::time::Instant;
use tracing::metadata::Metadata;

use super::{TraceLeafId, TraceThread, TraceFields};

/// Trace tree entry
#[derive(Debug)]
pub struct TraceLeaf {
    pub generation: usize,
    pub thread: TraceThread,
    pub parent_id: Option<TraceLeafId>,
    pub metadata: &'static Metadata<'static>,
    pub fields: TraceFields,
    pub variant: TraceLeafVariant,
}

/// TraceLeaf variants
#[derive(Debug)]
pub enum TraceLeafVariant {
    Span {
        open: bool,
        entered: Option<Instant>,
        exited: Option<Instant>,
        follows: Vec<TraceLeafId>,
    },
    Event {
        instant: Instant,
    },
}

