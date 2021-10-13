use std::num::NonZeroU64;

use tracing::Id;

/// Tracing [`Id`] wrapper, uses underlying [`NonZeroU64`] to avoid cloning
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SpanId(NonZeroU64);

impl From<&Id> for SpanId {
    fn from(id: &Id) -> Self {
        SpanId((*id).into_non_zero_u64())
    }
}

impl From<Id> for SpanId {
    fn from(id: Id) -> Self {
        SpanId(id.into_non_zero_u64())
    }
}

