use std::time::Instant;

use crate::Timing;

#[legion::system]
#[profiling::function]
pub fn timing_update(#[state] since: &Instant, #[resource] timing: &mut Timing) {
    timing.timestamp(since);
}
