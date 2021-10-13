use std::time::Instant;

use legion::system;

use crate::resources::Timing;

#[system]
#[profiling::function]
pub fn timing_update(#[state] since: &Instant, #[resource] timing: &mut Timing) {
    timing.timestamp(since);
}
