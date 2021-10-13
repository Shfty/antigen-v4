use crossbeam_channel::Sender;
use tracing::{Id, Subscriber, span::{Attributes, Record}};
use tracing_subscriber::{Layer, layer::Context};

use crate::{TraceEvent, ParentId, Fields};

/// [`tracing-subscriber::layer::Layer`] for collecting [`TraceEvent`]s and sending them to a [`TraceReceiver`]
pub struct TraceSender {
    sender: Sender<TraceEvent>,
}

impl TraceSender {
    pub fn new(sender: Sender<TraceEvent>) -> Self {
        Self { sender }
    }
}

impl<S> Layer<S> for TraceSender
where
    S: Subscriber,
{
    fn new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span_id = id.into();
        let parent_id = attrs.parent_id(ctx);
        let metadata = attrs.metadata();
        let fields = attrs.tracing_fields();

        self.sender
            .send(TraceEvent::new_span(span_id, parent_id, metadata, fields))
            .unwrap();
    }

    fn on_record(&self, id: &Id, record: &Record<'_>, _ctx: Context<'_, S>) {
        let span_id = id.into();
        let fields = record.tracing_fields();
        self.sender
            .send(TraceEvent::record(span_id, fields))
            .unwrap();
    }

    fn on_follows_from(&self, id: &Id, follows: &Id, _ctx: Context<'_, S>) {
        let span_id = id.into();
        let follows_id = follows.into();
        self.sender
            .send(TraceEvent::follows_from(span_id, follows_id))
            .unwrap();
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        let parent_id = event.parent_id(ctx);
        let metadata = event.metadata();
        let fields = event.tracing_fields();
        self.sender
            .send(TraceEvent::event(parent_id, metadata, fields))
            .unwrap();
    }

    fn on_enter(&self, id: &Id, _ctx: Context<'_, S>) {
        let span_id = id.into();
        self.sender.send(TraceEvent::enter(span_id)).unwrap();
    }

    fn on_exit(&self, id: &Id, _ctx: Context<'_, S>) {
        let span_id = id.into();
        self.sender.send(TraceEvent::exit(span_id)).unwrap();
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        let span_id = id.into();
        self.sender.send(TraceEvent::close(span_id)).unwrap();
    }
}

