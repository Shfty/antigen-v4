use std::time::Instant;

use tracing::Metadata;

use super::{SpanId, TraceThread, TraceFields};

/// Captured tracing data to be transferred across threads via channel
#[derive(Debug)]
pub struct TraceEvent {
    pub thread: TraceThread,
    pub variant: TraceEventVariant,
}

impl TraceEvent {
    pub fn new_span(
        id: SpanId,
        parent_id: Option<SpanId>,
        metadata: &'static Metadata<'static>,
        fields: TraceFields,
    ) -> Self {
        TraceEvent {
            thread: Self::thread(),
            variant: TraceEventVariant::NewSpan {
                id,
                parent_id,
                metadata,
                fields,
            },
        }
    }

    pub fn record(id: SpanId, fields: TraceFields) -> Self {
        TraceEvent {
            thread: Self::thread(),
            variant: TraceEventVariant::Record { id, fields },
        }
    }

    pub fn follows_from(id: SpanId, follows: SpanId) -> Self {
        TraceEvent {
            thread: Self::thread(),
            variant: TraceEventVariant::FollowsFrom { id, follows },
        }
    }

    pub fn event(
        parent_id: Option<SpanId>,
        metadata: &'static Metadata<'static>,
        fields: TraceFields,
    ) -> Self {
        TraceEvent {
            thread: Self::thread(),
            variant: TraceEventVariant::Event {
                parent_id,
                metadata,
                fields,
                instant: Instant::now(),
            },
        }
    }

    pub fn enter(id: SpanId) -> Self {
        TraceEvent {
            thread: Self::thread(),
            variant: TraceEventVariant::Enter {
                id,
                instant: Instant::now(),
            },
        }
    }

    pub fn exit(id: SpanId) -> Self {
        TraceEvent {
            thread: Self::thread(),
            variant: TraceEventVariant::Exit {
                id,
                instant: Instant::now(),
            },
        }
    }

    pub fn close(id: SpanId) -> Self {
        TraceEvent {
            thread: Self::thread(),
            variant: TraceEventVariant::Close { id },
        }
    }

    pub fn thread() -> TraceThread {
        std::thread::current().into()
    }
}

#[derive(Debug)]
pub enum TraceEventVariant {
    NewSpan {
        id: SpanId,
        parent_id: Option<SpanId>,
        metadata: &'static Metadata<'static>,
        fields: TraceFields,
    },
    Record {
        id: SpanId,
        fields: TraceFields,
    },
    FollowsFrom {
        id: SpanId,
        follows: SpanId,
    },
    Event {
        parent_id: Option<SpanId>,
        metadata: &'static Metadata<'static>,
        fields: TraceFields,
        instant: Instant,
    },
    Enter {
        id: SpanId,
        instant: Instant,
    },
    Exit {
        id: SpanId,
        instant: Instant,
    },
    Close {
        id: SpanId,
    },
}

