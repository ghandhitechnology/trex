//! Regular enemy behaviors. Each one is a small state machine over
//! `AiState`; windups are the telegraph the renderer shows.

use std::f32::consts::TAU;

use super::{AiState, Behavior, Enemy, EnemyDef, Pattern};
use crate::engine::{Rect, Rng, Vec2};
use crate::render::sprite::SpriteId;

/// Something an enemy wants done that needs the whole world.
pub enum Act {
    Shot {
        pos: Vec2,
        vel: Vec2,
        sprite: SpriteId,
    },
    Spawn {
        kind: usize,
        pos: Vec2,
        count: u32,
        spread: f32,
    },
    /// Hurts the player within `radius`; deals `friendly` damage to enemies.
    Blast {
        pos: Vec2,
        radius: f32,
        damage: i32,
        friendly: f32,
    },
    /// Boss rage: shake and rings.
    Roar {
        pos: Vec2,
    },
    /// Heavy landing: shake and dust.
    Impact {
        pos: Vec2,
    },
    /// Remove this enemy without loot.
    Vanish,
}

pub struct Ctx<'a> {
    pub target: Vec2,
    pub dt: f32,
    pub arena: Rect,
    pub rng: &'a mut Rng,
    pub out: &'a mut Vec<Act>,
}

impl Ctx<'_> {
    pub fn shot(&mut self, pos: Vec2, dir: Vec2, speed: f32, sprite: SpriteId) {
        self.out.push(Act::Shot { pos: pos + dir * 4.0, vel: dir * speed, sprite });
    }

    /// `n` shots fanned over `angle` radians around `dir`.
    pub fn fan(&mut self, pos: Vec2, dir: Vec2, n: u32, angle: f32, speed: f32, sprite: SpriteId) {
        let step = if n > 1 { angle / (n - 1) as f32 } else { 0.0 };
        for k in 0..n {
            let off = (k as f32 - (n - 1) as f32 / 2.0) * step;
            self.shot(pos, dir.rotate(off), speed, sprite);
        }
    }

    /// `n` shots evenly around, starting at angle `offset`.
    pub fn ring(&mut self, pos: Vec2, n: u32, offset: f32, speed: f32, sprite: SpriteId) {
        for k in 0..n {
            let a = offset + k as f32 / n.max(1) as f32 * TAU;
            self.shot(pos, Vec2::from_angle(a), speed, sprite);
        }
    }

    /// A point `dist` from the target in a random direction, inside the arena.
    pub fn near_target(&mut self, dist: f32, margin: f32) -> Vec2 {
        let a = self.rng.angle();
        self.arena.clamp(self.target + Vec2::from_angle(a) * dist, margin)
    }
}

/// Behavior-specific starting state.
pub fn init(e: &mut Enemy, def: &EnemyDef) {
    match def.behavior {
        Behavior::Burrow { under, .. } => {
            e.hidden = true;
            e.timer = under * (0.5 + e.phase * 0.5);
        }
        Behavior::Blink { cooldown, .. } | Behavior::Summon { cooldown, .. } => {
            e.timer = cooldown * (0.3 + e.phase * 0.5)
        }
        Behavior::Orbit { dive, .. } => {
            e.timer = dive * (0.5 + e.phase);
            e.spin = e.phase * TAU;
        }
        Behavior::Shoot { cooldown, .. } => e.timer = cooldown * (0.4 + e.phase * 0.6),
        Behavior::Boss(_) => {
            e.state = AiState::Recover;
            e.timer = 1.5;
        }
        _ => {}
    }
}

/// Decide velocity (and queue actions) for one enemy.
pub fn think(e: &mut Enemy, def: &EnemyDef, cx: &mut Ctx) -> Vec2 {
    let to = cx.target - e.pos;
    let dist = to.len();
    let dir = to.norm();
    let dt = cx.dt;
    match def.behavior {
        Behavior::Chase | Behavior::Boss(_) => dir * e.speed,
        Behavior::Weave { amp, freq } => {
            let sway = (e.anim * freq + e.phase * 6.0).sin() * amp;
            (dir + dir.perp() * sway).norm() * e.speed
        }
        Behavior::Charge { range, windup, speed, time, cooldown } => {
            e.timer -= dt;
            match e.state {
                AiState::Move => {
                    if dist < range && e.timer <= 0.0 {
                        e.state = AiState::Windup;
                        e.timer = windup;
                        e.dir = dir;
                    }
                    dir * e.speed
                }
                AiState::Windup => {
                    e.dir = e.dir.lerp(dir, 0.15).norm();
                    if e.timer <= 0.0 {
                        e.state = AiState::Act;
                        e.timer = time;
                    }
                    Vec2::ZERO
                }
                AiState::Act => {
                    if e.timer <= 0.0 {
                        e.state = AiState::Recover;
                        e.timer = cooldown;
                    }
                    e.dir * speed
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
        Behavior::Shoot { range, cooldown, speed, pattern, windup } => {
            e.timer -= dt;
            match e.state {
                AiState::Move => {
                    if dist < range && e.timer <= 0.0 {
                        e.state = AiState::Windup;
                        e.timer = windup;
                    }
                    keep_distance(e, dir, dist, range * 0.7)
                }
                AiState::Windup => {
                    if e.timer <= 0.0 {
                        let (pos, sprite) = (e.pos, def.shot_id);
                        match pattern {
                            Pattern::Single => cx.shot(pos, dir, speed, sprite),
                            Pattern::Spread { count, angle } => {
                                cx.fan(pos, dir, count, angle.to_radians(), speed, sprite)
                            }
                            Pattern::Ring { count } => {
                                let offset = dir.angle() + e.phase * TAU;
                                cx.ring(pos, count, offset, speed, sprite)
                            }
                            Pattern::Burst { count, .. } => {
                                e.state = AiState::Act;
                                e.count = count;
                                e.timer = 0.0;
                                return Vec2::ZERO;
                            }
                        }
                        e.state = AiState::Move;
                        e.timer = cooldown;
                    }
                    Vec2::ZERO
                }
                AiState::Act => {
                    if e.timer <= 0.0 {
                        cx.shot(e.pos, dir, speed, def.shot_id);
                        e.count = e.count.saturating_sub(1);
                        e.timer = if let Pattern::Burst { gap, .. } = pattern { gap } else { 0.1 };
                        if e.count == 0 {
                            e.state = AiState::Move;
                            e.timer = cooldown;
                        }
                    }
                    Vec2::ZERO
                }
                AiState::Recover => {
                    e.state = AiState::Move;
                    Vec2::ZERO
                }
            }
        }
        Behavior::Orbit { radius, spin, dive } => {
            e.timer -= dt;
            match e.state {
                AiState::Move => {
                    let turn = if e.phase > 0.5 { 1.0 } else { -1.0 };
                    e.spin += spin * turn * dt;
                    let goal = cx.target + Vec2::from_angle(e.spin) * radius;
                    if e.timer <= 0.0 && dist < radius * 1.4 {
                        e.state = AiState::Windup;
                        e.timer = 0.4;
                    }
                    (goal - e.pos).norm() * e.speed
                }
                AiState::Windup => {
                    e.dir = dir;
                    if e.timer <= 0.0 {
                        e.state = AiState::Act;
                        e.timer = (dist + radius) / (e.speed * 2.6);
                    }
                    -dir * e.speed * 0.3
                }
                AiState::Act => {
                    if e.timer <= 0.0 {
                        e.state = AiState::Move;
                        e.timer = dive;
                        e.spin = (e.pos - cx.target).angle();
                    }
                    e.dir * e.speed * 2.6
                }
                AiState::Recover => {
                    e.state = AiState::Move;
                    Vec2::ZERO
                }
            }
        }
        Behavior::Burrow { under, up, shots } => {
            e.timer -= dt;
            match e.state {
                AiState::Move => {
                    if e.timer <= 0.0 && dist < 60.0 {
                        e.state = AiState::Windup;
                        e.timer = 0.6;
                    }
                    dir * e.speed * 1.3
                }
                AiState::Windup => {
                    if e.timer <= 0.0 {
                        e.hidden = false;
                        e.state = AiState::Act;
                        e.timer = up;
                        let offset = cx.rng.angle();
                        cx.ring(e.pos, shots, offset, 55.0, def.shot_id);
                    }
                    Vec2::ZERO
                }
                AiState::Act => {
                    if e.timer <= 0.0 {
                        e.hidden = true;
                        e.state = AiState::Move;
                        e.timer = under;
                    }
                    dir * e.speed * 0.4
                }
                AiState::Recover => {
                    e.state = AiState::Move;
                    Vec2::ZERO
                }
            }
        }
        Behavior::Blink { range, cooldown, windup, shots } => {
            e.timer -= dt;
            match e.state {
                AiState::Move => {
                    if e.timer <= 0.0 && dist < range * 3.0 {
                        e.aim = cx.near_target(range, 10.0);
                        e.hidden = true;
                        e.state = AiState::Windup;
                        e.timer = windup;
                    }
                    let sway = (e.anim * 2.0 + e.phase * 6.0).sin() * 0.6;
                    (dir + dir.perp() * sway).norm() * e.speed
                }
                AiState::Windup => {
                    if e.timer <= 0.0 {
                        e.pos = e.aim;
                        e.hidden = false;
                        e.state = AiState::Recover;
                        e.timer = 0.3;
                        let aim = (cx.target - e.pos).norm();
                        cx.fan(e.pos, aim, shots, 0.5, 60.0, def.shot_id);
                    }
                    Vec2::ZERO
                }
                _ => {
                    if e.timer <= 0.0 {
                        e.state = AiState::Move;
                        e.timer = cooldown;
                    }
                    Vec2::ZERO
                }
            }
        }
        Behavior::Kamikaze { range, fuse, radius, damage } => {
            e.timer -= dt;
            match e.state {
                AiState::Move => {
                    if dist < range {
                        e.state = AiState::Windup;
                        e.timer = fuse;
                    }
                    dir * e.speed
                }
                _ => {
                    if e.timer <= 0.0 {
                        cx.out.push(Act::Vanish);
                        cx.out.push(Act::Blast { pos: e.pos, radius, damage, friendly: e.max_hp * 3.0 });
                    }
                    dir * e.speed * 0.35
                }
            }
        }
        Behavior::Slam { range, windup, radius, damage, cooldown } => {
            e.timer -= dt;
            match e.state {
                AiState::Move => {
                    if dist < range && e.timer <= 0.0 {
                        e.state = AiState::Windup;
                        e.timer = windup;
                    }
                    dir * e.speed
                }
                AiState::Windup => {
                    if e.timer <= 0.0 {
                        cx.out.push(Act::Blast { pos: e.pos, radius, damage, friendly: 0.0 });
                        e.state = AiState::Recover;
                        e.timer = 0.6;
                    }
                    Vec2::ZERO
                }
                _ => {
                    if e.timer <= 0.0 {
                        e.state = AiState::Move;
                        e.timer = cooldown;
                    }
                    Vec2::ZERO
                }
            }
        }
        Behavior::Summon { count, cooldown, range } => {
            e.timer -= dt;
            match e.state {
                AiState::Move => {
                    if e.timer <= 0.0 && dist < range * 1.8 {
                        e.state = AiState::Windup;
                        e.timer = 0.7;
                    }
                    keep_distance(e, dir, dist, range)
                }
                _ => {
                    if e.timer <= 0.0 {
                        cx.out.push(Act::Spawn { kind: def.minion_kind, pos: e.pos, count, spread: 14.0 });
                        e.state = AiState::Move;
                        e.timer = cooldown;
                    }
                    Vec2::ZERO
                }
            }
        }
    }
}

/// Approach to `keep`, back off when too close, strafe in between.
pub fn keep_distance(e: &Enemy, dir: Vec2, dist: f32, keep: f32) -> Vec2 {
    if dist > keep {
        dir * e.speed
    } else if dist < keep * 0.6 {
        -dir * e.speed * 0.6
    } else {
        dir.perp() * e.speed * 0.4 * if e.phase > 0.5 { 1.0 } else { -1.0 }
    }
}
