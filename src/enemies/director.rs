//! Wave director: spends credits that grow over time on enemies from the
//! current phase's pool, plus scripted group events. Endless by design.

use serde::Deserialize;

use super::Enemy;
use crate::content;
use crate::engine::Vec2;
use crate::game::world::World;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct WavesDef {
    /// Credits per second at the start of a run.
    pub credits: f32,
    /// Credits per second added each minute.
    pub credits_per_min: f32,
    /// Enemy HP growth per minute, compounding (0.2 = +20% per minute).
    pub hp_growth: f32,
    /// Enemy speed multiplier added each minute (capped at +50%).
    #[serde(default)]
    pub speed_per_min: f32,
    /// Cap on live plus pending enemies.
    pub max_alive: usize,
    pub phases: Vec<Phase>,
    #[serde(default)]
    pub events: Vec<WaveEvent>,
}

/// From `at` seconds on (until the next phase), spawn from `pool`.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Phase {
    pub at: f32,
    /// (enemy id, weight)
    pub pool: Vec<(String, f32)>,
    #[serde(skip)]
    pub kinds: Vec<(usize, f32)>,
}

/// A scripted group spawn at `at` seconds, repeating every `every` seconds if set.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct WaveEvent {
    pub at: f32,
    #[serde(default)]
    pub every: f32,
    pub enemy: String,
    pub count: u32,
    #[serde(default)]
    pub shape: Shape,
    #[serde(skip)]
    pub kind: usize,
}

#[derive(Deserialize, Debug, Default, Clone, Copy)]
pub enum Shape {
    /// Evenly spaced circle around the player.
    #[default]
    Ring,
    /// One tight group off screen.
    Cluster,
}

/// An enemy about to appear. On-screen spawns get a visible warning first.
#[derive(Clone, Debug)]
pub struct Pending {
    pub kind: usize,
    pub pos: Vec2,
    pub timer: f32,
}

pub const WARN_TIME: f32 = 0.8;

#[derive(Default)]
pub struct Director {
    credits: f32,
    next_event: Vec<f32>,
    pub pending: Vec<Pending>,
}

impl Director {
    pub fn new() -> Self {
        let waves = &content::get().waves;
        Director {
            credits: waves.credits * 2.0,
            next_event: waves.events.iter().map(|e| e.at).collect(),
            pending: Vec::new(),
        }
    }
}

pub fn update(w: &mut World, dt: f32) {
    let waves = &content::get().waves;
    let enemies = &content::get().enemies;
    let minutes = w.time / 60.0;
    let mut d = std::mem::take(&mut w.director);

    d.credits += (waves.credits + waves.credits_per_min * minutes) * dt;
    let phase = waves.phases.iter().rev().find(|p| p.at <= w.time).or(waves.phases.first());
    if let Some(phase) = phase {
        let weights: Vec<f32> = phase.kinds.iter().map(|k| k.1).collect();
        while w.enemies.len() + d.pending.len() < waves.max_alive {
            let Some(pick) = w.rng.weighted(&weights) else { break };
            let kind = phase.kinds[pick].0;
            let cost = enemies[kind].cost;
            if d.credits < cost {
                break;
            }
            d.credits -= cost;
            let pos = spawn_point(w, 0.0);
            queue(w, &mut d, kind, pos);
        }
    }
    d.credits = d.credits.min(40.0);

    for (i, ev) in waves.events.iter().enumerate() {
        if w.time < d.next_event[i] {
            continue;
        }
        d.next_event[i] = if ev.every > 0.0 { d.next_event[i] + ev.every } else { f32::INFINITY };
        let count = ev.count + (minutes * ev.count as f32 * 0.15) as u32;
        match ev.shape {
            Shape::Ring => {
                let r = w.view.1 as f32 * 0.62;
                let base = w.rng.angle();
                for k in 0..count {
                    let a = base + k as f32 / count as f32 * std::f32::consts::TAU;
                    let pos = w.arena.clamp(w.player.pos + Vec2::from_angle(a) * r, 8.0);
                    queue(w, &mut d, ev.kind, pos);
                }
            }
            Shape::Cluster => {
                let center = spawn_point(w, 12.0);
                for _ in 0..count {
                    let off = Vec2::from_angle(w.rng.angle()) * w.rng.range(0.0, 14.0);
                    let pos = w.arena.clamp(center + off, 8.0);
                    queue(w, &mut d, ev.kind, pos);
                }
            }
        }
    }

    let hp_mul = (1.0 + waves.hp_growth).powf(minutes);
    let speed_mul = 1.0 + (waves.speed_per_min * minutes).min(0.5);
    let mut i = 0;
    while i < d.pending.len() {
        d.pending[i].timer -= dt;
        if d.pending[i].timer > 0.0 {
            i += 1;
            continue;
        }
        let p = d.pending.swap_remove(i);
        let uid = w.next_uid();
        let phase = w.rng.f32();
        w.enemies.push(Enemy::new(uid, p.kind, p.pos, hp_mul, speed_mul, phase));
    }
    w.director = d;
}

fn queue(w: &mut World, d: &mut Director, kind: usize, pos: Vec2) {
    let timer = if w.camera.sees(pos, 6.0) { WARN_TIME } else { 0.0 };
    d.pending.push(Pending { kind, pos, timer });
}

/// A point just outside the view around the player, kept inside the arena.
fn spawn_point(w: &mut World, extra: f32) -> Vec2 {
    let half = Vec2::new(w.view.0 as f32, w.view.1 as f32) / 2.0;
    let r = half.len() + 10.0 + extra + w.rng.range(0.0, 30.0);
    let mut best = w.player.pos;
    let mut best_d = -1.0;
    for _ in 0..4 {
        let p = w.arena.clamp(w.player.pos + Vec2::from_angle(w.rng.angle()) * r, 8.0);
        let d = p.dist_sq(w.player.pos);
        if d > best_d {
            best = p;
            best_d = d;
        }
        if !w.camera.sees(p, 8.0) {
            break;
        }
    }
    best
}
