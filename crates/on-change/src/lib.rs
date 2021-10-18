use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OnChange<T> {
    data: T,
    dirty: AtomicBool,
}

impl<T> OnChange<T> {
    /// Create an OnChange<T> with the dirty flag unset.
    ///
    /// Useful for data that shouldn't write its initial state.
    pub fn new_clean(data: T) -> Self {
        OnChange {
            data,
            dirty: false.into(),
        }
    }

    /// Create an OnChange<T> with the dirty flag set.
    ///
    /// Useful for data that should write its initial state.
    pub fn new_dirty(data: T) -> Self {
        OnChange {
            data,
            dirty: true.into(),
        }
    }

    /// Get the underlying value, ignoring dirty flag state.
    pub fn get(&self) -> &T {
        &self.data
    }

    /// Set the underlying value and dirty flag.
    pub fn set(&mut self, data: T) {
        self.data = data;
        self.dirty.store(true, Ordering::SeqCst);
    }

    // If the value has changed, set the underlying value and mark it as dirty
    pub fn set_checked(&mut self, data: T)
    where
        T: PartialEq,
    {
        if data != self.data {
            self.set(data)
        }
    }

    /// Retrieve Some(T) and reset the dirty flag if it is set, or None otherwise.
    pub fn take_change(&mut self) -> Option<&T> {
        if self.dirty.load(Ordering::SeqCst) {
            self.dirty.store(false, Ordering::SeqCst);
            Some(&self.data)
        } else {
            None
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }

    pub fn set_dirty(&self, dirty: bool) {
        self.dirty.store(dirty, Ordering::SeqCst)
    }
}
