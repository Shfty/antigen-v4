use crossterm::event::Event as CrosstermEvent;
use serde::{ser::SerializeTupleStruct, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default, Clone)]
pub struct CrosstermEventQueue(Vec<CrosstermEvent>);

impl Serialize for CrosstermEventQueue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let st = serializer.serialize_tuple_struct("CrosstermEventQueue", 1)?;
        st.end()
    }
}

impl Deref for CrosstermEventQueue {
    type Target = Vec<CrosstermEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CrosstermEventQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

