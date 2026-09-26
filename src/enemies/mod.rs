//! Enemy definitions, runtime state, and per-tick enemy simulation.

pub mod ai;
pub mod director;
pub mod sprites;

use serde::Deserialize;

use crate::content;
use crate::engine::Vec2;
use crate::game::player;
use crate::game::world::{Hit, Shot, World};
use crate::render::palette;
use crate::render::sprite::SpriteId;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EnemyDef {
    pub id: String,
    pub sprite: String,
    pub hp: f32,
    /// Move speed, px/s.
    pub speed: f32,
    /// Contact damage in half hearts.
    #[serde(default = "one_i")]
    pub damage: i32,
    /// Collision radius, px.
    #[serde(default = "default_radius")]
    pub radius: f32,
    /// XP dropped on death.
    #[serde(default = "one_u")]
    pub xp: u32,
    /// Knockback resistance. 2.0 takes half the push.
    #[serde(default = "one_f")]
    pub mass: f32,
    /// Director credits spent to spawn one.
    #[serde(default = "one_f")]
    pub cost: f32,
    #[serde(default)]
    pub behavior: Behavior,
    #[serde(skip)]
    pub sprite_id: SpriteId,
    #[serde(skip)]
    pub shot_id: SpriteId,
}

fn one_i() -> i32 {
    1
}
fn one_u() -> u32 {
    1
}
fn one_f() -> f32 {
    1.0
}
fn default_radius() -> f32 {
    6.0
}

#[derive(Deserialize, Debug, Default, Clone)]
pub enum Behavior {
    /// Walk straight at the player.
    #[default]
    Chase,
    /// Chase while swaying side to side.
    Weave { amp: f32, freq: f32 },
    /// Close in, stop to wind up, then dash in a straight line.
    Charge { range: f32, windup: f32, speed: f32, time: f32, cooldown: f32 },
    /// Keep distance and fire slow shots at the player.
    Shoot { range: f32, cooldown: f32, speed: f32, sprite: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AiState {
    #[default]
    Move,
    Windup,
    Charge,
    Recover,
}

#[derive(Clone, Debug, Default)]
pub struct Enemy {
    pub uid: u32,
    pub kind: usize,
    pub pos: Vec2,
    pub vel: Vec2,
    /// Knockback velocity, decays over time.
    pub push: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub flash: f32,
    pub anim: f32,
    pub burn_dps: f32,
    pub burn_time: f32,
    pub burn_tick: f32,
    pub slow: f32,
    pub slow_time: f32,
    pub state: AiState,
    pub timer: f32,
    pub dir: Vec2,
    /// Per-enemy phase offset for sway and animation.
    pub phase: f32,
    pub dead: bool,
}

impl Enemy {
    pub fn new(uid: u32, kind: usize, pos: Vec2, hp_mul: f32, speed_mul: f32, phase: f32) -> Self {
        let def = &content::get().enemies[kind];
        Enemy {
            uid,
            kind,
            pos,
            hp: def.hp * hp_mul,
            max_hp: def.hp * hp_mul,
            speed: def.speed * speed_mul,
            phase,
            timer: phase,
            ..Default::default()
        }
    }
}

const BURN_TICK: f32 = 0.25;
const PLAYER_RADIUS: f32 = 5.0;

/// Statuses, AI, separation, movement, and contact damage for every enemy.
pub fn update(w: &mut World, dt: f32) {
    let content = content::get();
    let target = w.player.pos;
    let mut burns: Vec<(usize, f32)> = Vec::new();

    for i in 0..w.enemies.len() {
        let e = &mut w.enemies[i];
        if e.dead {
            continue;
        }
        let def = &content.enemies[e.kind];
        e.flash = (e.flash - dt).max(0.0);
        e.anim += dt;
        if e.slow_time > 0.0 {
            e.slow_time -= dt;
        } else {
            e.slow = 0.0;
        }
        if e.burn_time > 0.0 {
            e.burn_time -= dt;
            e.burn_tick -= dt;
            if e.burn_tick <= 0.0 {
                e.burn_tick += BURN_TICK;
                burns.push((i, e.burn_dps * BURN_TICK));
            }
        } else {
            e.burn_dps = 0.0;
        }

        let intent = ai::think(e, def, target, dt);
        e.vel = intent.vel * (1.0 - e.slow);
        if let (Some(dir), Behavior::Shoot { speed, .. }) = (intent.shoot, &def.behavior) {
            let pos = e.pos;
            w.shots.push(Shot::hostile(pos, dir * *speed, def.damage, def.shot_id));
        }
    }

    for (i, dmg) in burns {
        let pos = w.enemies[i].pos;
        w.damage_enemy(i, Hit { damage: dmg, procs: false, ..Hit::default() });
        if w.fx.enabled && w.fx.rng.chance(0.6) {
            w.fx.spark(pos + Vec2::new(0.0, -4.0), palette::AMBER, 20.0);
        }
    }

    separate(w, dt);

    let arena = w.arena;
    for e in w.enemies.iter_mut().filter(|e| !e.dead) {
        let def = &content.enemies[e.kind];
        e.pos += (e.vel + e.push) * dt;
        e.push *= (1.0 - 9.0 * dt).max(0.0);
        e.pos = arena.clamp(e.pos, def.radius);
    }

    for i in 0..w.enemies.len() {
        let e = &w.enemies[i];
        if e.dead {
            continue;
        }
        let def = &content.enemies[e.kind];
        if e.pos.dist_sq(target) < (def.radius + PLAYER_RADIUS).powi(2) {
            let (dmg, from) = (def.damage, e.pos);
            player::hurt(w, dmg, from);
        }
    }
}

/// Soft push so crowds spread out instead of stacking on one pixel.
fn separate(w: &mut World, dt: f32) {
    let content = content::get();
    let n = w.enemies.len();
    let mut shove = vec![Vec2::ZERO; n];
    for (i, e) in w.enemies.iter().enumerate() {
        if e.dead {
            continue;
        }
        let ri = content.enemies[e.kind].radius;
        w.grid.query(e.pos, ri * 2.0, |j| {
            if j == i || j >= n {
                return;
            }
            let o = &w.enemies[j];
            if o.dead {
                return;
            }
            let min = ri + content.enemies[o.kind].radius;
            let d = e.pos - o.pos;
            let dist = d.len();
            if dist < min {
                let dir = if dist > 1e-3 { d / dist } else { Vec2::from_angle(i as f32) };
                shove[i] += dir * (min - dist);
            }
        });
    }
    for (e, s) in w.enemies.iter_mut().zip(shove) {
        e.pos += s.clamp_len(60.0 * dt);
    }
}
