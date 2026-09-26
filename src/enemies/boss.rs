//! Boss scripts. A boss walks in its own style, then runs moves from the
//! list for its rage level. Rage rises at 55% and 25% HP: a roar, a ring of
//! shots, a harder move list, and shorter rests. Every move has a windup the
//! renderer telegraphs.

use std::f32::consts::{PI, TAU};

use serde::Deserialize;

use super::ai::{Act, Ctx, keep_distance};
use super::{AiState, Enemy, EnemyDef};
use crate::engine::Vec2;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossKind {
    /// Tar slime queen: spit fans, broods, rings.
    Mire,
    /// Skeleton charger: telegraphed charges, ground slams.
    Colossus,
    /// Teleports and spirals.
    Wraith,
    /// Burrows under the player and erupts.
    Sandmaw,
    /// Snipes, bullet walls, and swarms.
    Eye,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Move {
    #[default]
    Idle,
    Fan {
        n: u32,
        volleys: u32,
    },
    Ring {
        n: u32,
        waves: u32,
    },
    Brood {
        n: u32,
    },
    Charge {
        times: u32,
    },
    Slam,
    Teleport,
    Spiral {
        arms: u32,
        secs: f32,
    },
    Erupt,
    Snipe {
        n: u32,
    },
    Wall {
        rows: u32,
    },
}

pub const SLAM_RADIUS: f32 = 44.0;
pub const ERUPT_RADIUS: f32 = 26.0;
/// Seconds before the end of a windup when the aim stops tracking.
pub const CHARGE_LOCK: f32 = 0.5;
pub const SNIPE_LOCK: f32 = 0.3;
pub const ERUPT_LOCK: f32 = 0.75;
const CHARGE_SPEED: f32 = 250.0;

fn moves(kind: BossKind, rage: u8) -> &'static [Move] {
    use BossKind as B;
    use Move as M;
    match (kind, rage.min(1)) {
        (B::Mire, 0) => &[
            M::Fan { n: 4, volleys: 2 },
            M::Brood { n: 3 },
            M::Fan { n: 4, volleys: 2 },
            M::Ring { n: 10, waves: 2 },
        ],
        (B::Mire, _) => &[
            M::Fan { n: 6, volleys: 2 },
            M::Ring { n: 14, waves: 2 },
            M::Brood { n: 4 },
            M::Fan { n: 6, volleys: 3 },
            M::Ring { n: 14, waves: 2 },
        ],
        (B::Colossus, 0) => {
            &[M::Charge { times: 1 }, M::Slam, M::Charge { times: 1 }, M::Fan { n: 3, volleys: 2 }]
        }
        (B::Colossus, _) => {
            &[M::Charge { times: 2 }, M::Slam, M::Ring { n: 16, waves: 2 }, M::Charge { times: 3 }]
        }
        (B::Wraith, 0) => {
            &[M::Teleport, M::Spiral { arms: 2, secs: 2.5 }, M::Teleport, M::Fan { n: 3, volleys: 3 }]
        }
        (B::Wraith, _) => &[
            M::Teleport,
            M::Spiral { arms: 3, secs: 3.0 },
            M::Teleport,
            M::Ring { n: 20, waves: 2 },
            M::Spiral { arms: 4, secs: 2.5 },
        ],
        (B::Sandmaw, 0) => &[M::Erupt, M::Fan { n: 3, volleys: 3 }, M::Erupt, M::Ring { n: 12, waves: 1 }],
        (B::Sandmaw, _) => {
            &[M::Erupt, M::Erupt, M::Fan { n: 5, volleys: 3 }, M::Erupt, M::Ring { n: 16, waves: 2 }]
        }
        (B::Eye, 0) => {
            &[M::Snipe { n: 5 }, M::Wall { rows: 2 }, M::Brood { n: 6 }, M::Spiral { arms: 4, secs: 2.0 }]
        }
        (B::Eye, _) => &[
            M::Wall { rows: 3 },
            M::Snipe { n: 7 },
            M::Spiral { arms: 4, secs: 3.0 },
            M::Brood { n: 8 },
            M::Ring { n: 24, waves: 3 },
        ],
    }
}

fn windup_time(m: Move) -> f32 {
    match m {
        Move::Idle => 0.0,
        Move::Fan { .. } | Move::Wall { .. } => 0.6,
        Move::Ring { .. } | Move::Brood { .. } => 0.7,
        Move::Spiral { .. } => 0.5,
        Move::Teleport | Move::Snipe { .. } => 0.9,
        Move::Charge { .. } => 1.1,
        Move::Slam => 1.0,
        Move::Erupt => 2.0,
    }
}

pub fn think(e: &mut Enemy, def: &EnemyDef, kind: BossKind, cx: &mut Ctx) -> Vec2 {
    let to = cx.target - e.pos;
    let (dist, dir) = (to.len(), to.norm());
    let frac = e.hp / e.max_hp.max(1.0);
    let rage = if frac < 0.25 {
        2
    } else if frac < 0.55 {
        1
    } else {
        0
    };
    if rage > e.rage {
        e.rage = rage;
        e.hidden = false;
        e.mv = Move::Idle;
        e.state = AiState::Recover;
        e.timer = 1.0;
        let offset = cx.rng.angle();
        cx.ring(e.pos, 16 + 6 * rage as u32, offset, 50.0, def.shot_id);
        cx.out.push(Act::Roar { pos: e.pos });
        return Vec2::ZERO;
    }

    e.timer -= cx.dt;
    match e.state {
        AiState::Move => {
            if e.timer <= 0.0 {
                let list = moves(kind, e.rage);
                let m = list[e.step as usize % list.len()];
                e.step += 1;
                start(e, m, dir, cx);
            }
            walk(e, kind, dir, dist, cx.target)
        }
        AiState::Windup => {
            let vel = match e.mv {
                Move::Charge { .. } => {
                    if e.timer > CHARGE_LOCK {
                        e.dir = dir;
                    }
                    Vec2::ZERO
                }
                Move::Snipe { .. } => {
                    if e.timer > SNIPE_LOCK {
                        e.dir = dir;
                    }
                    Vec2::ZERO
                }
                Move::Erupt if e.timer > ERUPT_LOCK => to.clamp_len(e.speed * 4.0 * cx.dt) / cx.dt,
                Move::Erupt | Move::Teleport | Move::Slam => Vec2::ZERO,
                _ => walk(e, kind, dir, dist, cx.target) * 0.3,
            };
            if e.timer <= 0.0 {
                begin(e, def, dir, cx);
            }
            vel
        }
        AiState::Act => act(e, def, dir, cx),
        AiState::Recover => {
            if e.timer <= 0.0 {
                e.state = AiState::Move;
                e.timer = [1.1, 0.85, 0.6][e.rage as usize];
            }
            walk(e, kind, dir, dist, cx.target) * 0.4
        }
    }
}

fn walk(e: &Enemy, kind: BossKind, dir: Vec2, dist: f32, target: Vec2) -> Vec2 {
    match kind {
        BossKind::Mire => keep_distance(e, dir, dist, 55.0),
        BossKind::Colossus | BossKind::Sandmaw => dir * e.speed,
        BossKind::Wraith | BossKind::Eye => {
            let r = if kind == BossKind::Wraith { 85.0 } else { 100.0 };
            let goal = target + (e.pos - target).norm().rotate(0.35) * r;
            let d = goal - e.pos;
            d.norm() * e.speed * (d.len() / 16.0).min(1.0)
        }
    }
}

fn shot_speed(e: &Enemy, base: f32) -> f32 {
    base * (1.0 + 0.12 * e.rage as f32)
}

fn start(e: &mut Enemy, m: Move, dir: Vec2, cx: &mut Ctx) {
    e.mv = m;
    e.state = AiState::Windup;
    e.timer = windup_time(m);
    e.dir = dir;
    match m {
        Move::Teleport => {
            e.aim = cx.near_target(80.0, 16.0);
            e.hidden = true;
        }
        Move::Erupt => e.hidden = true,
        Move::Charge { times } => e.count = times,
        _ => {}
    }
}

fn finish(e: &mut Enemy) {
    e.mv = Move::Idle;
    e.state = AiState::Recover;
    e.timer = 0.6;
}

/// End of a windup: fire one-shot moves, set up repeating ones.
fn begin(e: &mut Enemy, def: &EnemyDef, dir: Vec2, cx: &mut Ctx) {
    e.state = AiState::Act;
    e.clock = 0.0;
    let (pos, sprite) = (e.pos, def.shot_id);
    match e.mv {
        Move::Fan { volleys, .. } => e.count = volleys,
        Move::Ring { waves, .. } => e.count = waves,
        Move::Snipe { n } => e.count = n,
        Move::Wall { rows } => e.count = rows,
        Move::Charge { .. } => e.timer = 1.0,
        Move::Spiral { secs, .. } => {
            e.timer = secs;
            e.spin = dir.angle();
        }
        Move::Brood { n } => {
            cx.out.push(Act::Spawn { kind: def.minion_kind, pos, count: n, spread: 24.0 });
            finish(e);
        }
        Move::Slam => {
            cx.out.push(Act::Blast { pos, radius: SLAM_RADIUS, damage: 2, friendly: 0.0 });
            cx.ring(pos, 16, dir.angle(), shot_speed(e, 55.0), sprite);
            finish(e);
        }
        Move::Teleport => {
            e.pos = e.aim;
            e.hidden = false;
            cx.ring(e.aim, 8, dir.angle() + PI / 8.0, shot_speed(e, 50.0), sprite);
            finish(e);
        }
        Move::Erupt => {
            e.hidden = false;
            cx.out.push(Act::Blast { pos, radius: ERUPT_RADIUS, damage: 2, friendly: 0.0 });
            cx.out.push(Act::Impact { pos });
            let offset = cx.rng.angle();
            cx.ring(pos, 12, offset, shot_speed(e, 60.0), sprite);
            finish(e);
        }
        Move::Idle => finish(e),
    }
}

/// Repeating part of a move. Returns the velocity.
fn act(e: &mut Enemy, def: &EnemyDef, dir: Vec2, cx: &mut Ctx) -> Vec2 {
    let (pos, sprite, dt) = (e.pos, def.shot_id, cx.dt);
    e.clock -= dt;
    let fire = e.clock <= 0.0;
    match e.mv {
        Move::Fan { n, .. } => {
            if fire {
                cx.fan(pos, dir, n, 0.18 * (n - 1) as f32, shot_speed(e, 58.0), sprite);
                e.clock = 0.5;
                e.count = e.count.saturating_sub(1);
            }
        }
        Move::Ring { n, .. } => {
            if fire {
                cx.ring(pos, n, e.spin, shot_speed(e, 50.0), sprite);
                e.spin += PI / n as f32;
                e.clock = 0.45;
                e.count = e.count.saturating_sub(1);
            }
        }
        Move::Snipe { .. } => {
            if fire {
                cx.shot(pos, e.dir, shot_speed(e, 165.0), sprite);
                e.clock = 0.07;
                e.count = e.count.saturating_sub(1);
            }
        }
        Move::Wall { .. } => {
            if fire {
                let side = dir.perp();
                let gap = cx.rng.range(-3.5, 3.5).round() as i32;
                for k in -11..=11 {
                    if (k - gap).abs() > 1 {
                        let at = pos + side * (k as f32 * 9.0);
                        cx.out.push(Act::Shot { pos: at, vel: dir * shot_speed(e, 48.0), sprite });
                    }
                }
                e.clock = 1.0;
                e.count = e.count.saturating_sub(1);
            }
        }
        Move::Spiral { arms, .. } => {
            if fire {
                let turn = if e.rage > 0 && e.step.is_multiple_of(2) { -1.0 } else { 1.0 };
                for k in 0..arms {
                    let a = e.spin + k as f32 / arms as f32 * TAU;
                    cx.shot(pos, Vec2::from_angle(a), shot_speed(e, 58.0), sprite);
                }
                e.spin += 0.22 * turn;
                e.clock = 0.09;
            }
            if e.timer <= 0.0 {
                finish(e);
            }
            return Vec2::ZERO;
        }
        Move::Charge { .. } => {
            let r = def.radius + 2.0;
            let next = pos + e.dir * CHARGE_SPEED * dt;
            let a = cx.arena;
            let wall =
                next.x < a.x + r || next.y < a.y + r || next.x > a.x + a.w - r || next.y > a.y + a.h - r;
            if !(wall || e.timer <= 0.0) {
                return e.dir * CHARGE_SPEED;
            }
            cx.out.push(Act::Impact { pos });
            cx.ring(pos, 10, e.dir.angle(), shot_speed(e, 60.0), sprite);
            e.count = e.count.saturating_sub(1);
            if e.count > 0 {
                e.state = AiState::Windup;
                e.timer = CHARGE_LOCK + 0.2;
            } else {
                finish(e);
            }
            return Vec2::ZERO;
        }
        _ => finish(e),
    }
    if e.state == AiState::Act && e.count == 0 {
        finish(e);
    }
    Vec2::ZERO
}
