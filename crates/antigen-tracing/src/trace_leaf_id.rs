/// Unique identifier for trace variant entries
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TraceLeafId(usize);

impl From<usize> for TraceLeafId {
    fn from(id: usize) -> Self {
        TraceLeafId(id)
    }
}

impl From<TraceLeafId> for usize {
    fn from(trace_leaf_id: TraceLeafId) -> Self {
        trace_leaf_id.0
    }
}

