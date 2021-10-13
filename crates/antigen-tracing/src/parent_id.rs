use super::SpanId;

use tracing::{Event, Subscriber, span::Attributes};
use tracing_subscriber::layer::Context;

/// Retrieves a parent tracing span ID as [`SpanId`]
pub trait ParentId {
    fn parent_id<S: Subscriber>(&self, _ctx: Context<'_, S>) -> Option<SpanId> {
        None
    }
}

impl ParentId for Attributes<'_> {
    fn parent_id<S: Subscriber>(&self, ctx: Context<'_, S>) -> Option<SpanId> {
        if self.is_contextual() {
            let current = ctx.current_span();
            if let Some((id, _)) = current.into_inner() {
                return Some(id.into());
            }
        } else if let Some(parent_id) = self.parent() {
            return Some(parent_id.into());
        }

        None
    }
}

impl ParentId for Event<'_> {
    fn parent_id<S: Subscriber>(&self, ctx: Context<'_, S>) -> Option<SpanId> {
        if self.is_contextual() {
            let current = ctx.current_span();
            if let Some((id, _)) = current.into_inner() {
                return Some(id.into());
            }
        } else if let Some(parent_id) = self.parent() {
            return Some(parent_id.into());
        }

        None
    }
}

