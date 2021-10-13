use std::{ops::Deref, thread::Thread};

/// Hashable [`Thread`] wrapper
#[derive(Debug)]
pub struct TraceThread(Thread);

impl Deref for TraceThread {
    type Target = Thread;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<TraceThread> for Thread {
    fn from(TraceThread(thread): TraceThread) -> Thread {
        thread
    }
}

impl From<Thread> for TraceThread {
    fn from(thread: Thread) -> Self {
        TraceThread(thread)
    }
}

impl PartialEq for TraceThread {
    fn eq(&self, other: &Self) -> bool {
        self.0.id().eq(&other.0.id())
    }
}

impl Eq for TraceThread {
    fn assert_receiver_is_total_eq(&self) {
        self.0.id().assert_receiver_is_total_eq()
    }
}

impl std::hash::Hash for TraceThread {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.id().hash(state)
    }
}
