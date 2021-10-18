use crate::{Position, Velocity};
use antigen_resources::Timing;

use legion::system;

#[system(par_for_each)]
#[profiling::function]
pub fn game_update_positions(pos: &mut Position, vel: &Velocity, #[resource] timing: &Timing) {
    let delta = timing.delta_time().as_secs_f32();
    pos.x += vel.dx * delta;
    pos.y += vel.dy * delta;
}

