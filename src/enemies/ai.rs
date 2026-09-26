use super::{AiState, Behavior, Enemy, EnemyDef};
use crate::engine::Vec2;

pub struct Intent {
    pub vel: Vec2,
    /// Unit direction to fire a hostile shot this tick.
    pub shoot: Option<Vec2>,
}

/// Decide velocity (and shots) for one enemy. Pure apart from the enemy's own state.
pub fn think(e: &mut Enemy, def: &EnemyDef, target: Vec2, dt: f32) -> Intent {
    let to = target - e.pos;
    let dist = to.len();
    let dir = to.norm();
    let mut shoot = None;
    let vel = match &def.behavior {
        Behavior::Chase => dir * e.speed,
        Behavior::Weave { amp, freq } => {
            let sway = (e.anim * freq + e.phase * 6.0).sin() * amp;
            (dir + dir.perp() * sway).norm() * e.speed
        }
        Behavior::Charge { range, windup, speed, time, cooldown } => {
            e.timer -= dt;
            match e.state {
                AiState::Move => {
                    if dist < *range && e.timer <= 0.0 {
                        e.state = AiState::Windup;
                        e.timer = *windup;
                        e.dir = dir;
                    }
                    dir * e.speed
                }
                AiState::Windup => {
                    e.dir = e.dir.lerp(dir, 0.15).norm();
                    if e.timer <= 0.0 {
                        e.state = AiState::Charge;
                        e.timer = *time;
                    }
                    Vec2::ZERO
                }
                AiState::Charge => {
                    if e.timer <= 0.0 {
                        e.state = AiState::Recover;
                        e.timer = *cooldown;
                    }
                    e.dir * *speed
                }
                AiState::Recover => {
                    if e.timer <= 0.0 {
                        e.state = AiState::Move;
                        e.timer = 0.0;
                    }
                    dir * e.speed * 0.35
                }
            }
        }
        Behavior::Shoot { range, cooldown, .. } => {
            e.timer -= dt;
            if dist < *range && e.timer <= 0.0 {
                e.timer = *cooldown;
                shoot = Some(dir);
            }
            let keep = range * 0.7;
            if dist > keep {
                dir * e.speed
            } else if dist < keep * 0.6 {
                -dir * e.speed * 0.6
            } else {
                dir.perp() * e.speed * 0.4 * if e.phase > 0.5 { 1.0 } else { -1.0 }
            }
        }
    };
    Intent { vel, shoot }
}
