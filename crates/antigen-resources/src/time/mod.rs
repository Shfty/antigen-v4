use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Timing {
    fifo: [Duration; 2],
    delta: Duration,
}

impl Timing {
    pub fn timestamp(&mut self, instant: &Instant) {
        self.fifo[0] = self.fifo[1];
        self.fifo[1] = instant.elapsed();
        self.delta = self.fifo[1] - self.fifo[0];
    }

    pub fn total_time(&self) -> Duration {
        self.fifo[0]
    }

    pub fn delta_time(&self) -> Duration {
        self.delta
    }
}

legion_debugger::register_resource!(Timing);
