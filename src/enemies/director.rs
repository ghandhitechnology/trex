//! Wave director. Endless by design: credits, enemy HP, and elite odds grow
//! with time; themed stages rotate the spawn pool and the ground palette;
//! bosses arrive on a fixed schedule and cycle forever.

use serde::Deserialize;

use super::{Elite, Enemy, EnemyDef};
use crate::content;
use crate::engine::Vec2;
use crate::game::world::World;
use crate::render::arena::Biome;
use crate::render::palette::{self, Color};

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
    /// Seconds per stage.
    pub stage_length: f32,
    /// After the last stage, loop back to this one.
    #[serde(default)]
    pub loop_from: usize,
    pub stages: Vec<Stage>,
    #[serde(default)]
    pub events: Vec<WaveEvent>,
    pub elites: Elites,
    pub bosses: Bosses,
    #[serde(skip)]
    pub boss_kinds: Vec<usize>,
}

/// A themed stretch of the run with its own spawn pool and ground colors.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub name: String,
    /// Arena look: floor, props, walls, and ambience.
    #[serde(default)]
    pub biome: Biome,
    /// Three palette characters (dark, mid, light) the floor is recolored to.
    #[serde(default)]
    pub ground: Option<String>,
    /// (enemy id, weight)
    pub pool: Vec<(String, f32)>,
    #[serde(skip)]
    pub kinds: Vec<(usize, f32)>,
    #[serde(skip)]
    pub ramp: Option<[Color; 3]>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Elites {
    /// Seconds before elites can appear.
    pub from: f32,
    /// Chance per spawn once they can, plus `per_min` each minute after, up to `max`.
    pub chance: f32,
    pub per_min: f32,
    pub max: f32,
    /// HP multiplier.
    pub hp: f32,
    /// Credit cost multiplier.
    pub cost: f32,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Bosses {
    /// Seconds until the first boss, then one every `every` seconds.
    pub first: f32,
    pub every: f32,
    /// Boss enemy ids in order; loops.
    pub order: Vec<String>,
    /// Spawn credit rate while a boss is alive.
    pub calm: f32,
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

impl WaveEvent {
    /// First firing time at or after `t`.
    fn first_after(&self, t: f32) -> f32 {
        if t <= self.at {
            self.at
        } else if self.every > 0.0 {
            self.at + ((t - self.at) / self.every).ceil() * self.every
        } else {
            f32::INFINITY
        }
    }
}

impl WavesDef {
    /// Resolve enemy ids and ground colors after loading.
    pub fn resolve(&mut self, enemies: &[EnemyDef]) -> Result<(), String> {
        let enemy = |id: &str| {
            enemies.iter().position(|e| e.id == id).ok_or_else(|| format!("waves: unknown enemy `{id}`"))
        };
        if self.stages.is_empty() {
            return Err("waves: no stages".into());
        }
        if self.loop_from >= self.stages.len() {
            return Err("waves: loop_from is past the last stage".into());
        }
        for s in &mut self.stages {
            s.kinds = s.pool.iter().map(|(id, w)| Ok((enemy(id)?, *w))).collect::<Result<_, String>>()?;
            s.ramp = match &s.ground {
                None => None,
                Some(g) => {
                    let c: Vec<Color> = g.chars().filter_map(palette::index_of).map(palette::color).collect();
                    if c.len() != 3 || g.chars().count() != 3 {
                        return Err(format!("waves: stage `{}` ground needs 3 palette colors", s.name));
                    }
                    Some([c[0], c[1], c[2]])
                }
            };
        }
        for ev in &mut self.events {
            ev.kind = enemy(&ev.enemy)?;
        }
        self.boss_kinds = self.bosses.order.iter().map(|id| enemy(id)).collect::<Result<_, _>>()?;
        for &k in &self.boss_kinds {
            if enemies[k].boss().is_none() {
                return Err(format!("waves: `{}` is not a boss", enemies[k].id));
            }
        }
        Ok(())
    }

    /// Stage index at `t` seconds, looping after the last one.
    pub fn stage_at(&self, t: f32) -> usize {
        let i = (t.max(0.0) / self.stage_length) as usize;
        let n = self.stages.len();
        if i < n { i } else { self.loop_from + (i - n) % (n - self.loop_from) }
    }

    pub fn hp_mul(&self, t: f32) -> f32 {
        (1.0 + self.hp_growth).powf(t / 60.0)
    }

    pub fn speed_mul(&self, t: f32) -> f32 {
        1.0 + (self.speed_per_min * t / 60.0).min(0.5)
    }
}

/// An enemy about to appear. On-screen spawns get a visible warning first.
#[derive(Clone, Debug)]
pub struct Pending {
    pub kind: usize,
    pub pos: Vec2,
    pub timer: f32,
    pub elite: Option<Elite>,
    /// Multiplies the HP it spawns with (splits spawn weaker).
    pub hp_scale: f32,
}

pub const WARN_TIME: f32 = 0.8;
const BOSS_WARN: f32 = 2.5;
const BANNER_TIME: f32 = 2.6;

/// Big centered text: a new stage or an incoming boss.
#[derive(Clone, Debug)]
pub struct Banner {
    pub text: String,
    pub color: Color,
    pub time: f32,
}

impl Banner {
    pub const TIME: f32 = BANNER_TIME;
}

#[derive(Default)]
pub struct Director {
    credits: f32,
    next_event: Vec<f32>,
    pub pending: Vec<Pending>,
    /// Seconds added to the run clock (`TREX_WARP`, for testing late game).
    warp: f32,
    next_boss: f32,
    bosses_sent: usize,
    pub stage: usize,
    pub prev_stage: usize,
    /// Seconds since the stage changed.
    pub stage_age: f32,
    pub banner: Option<Banner>,
}

impl Director {
    pub fn new() -> Self {
        let waves = &content::get().waves;
        let warp =
            std::env::var("TREX_WARP").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0).max(0.0);
        let mut next_boss = waves.bosses.first;
        let mut bosses_sent = 0;
        while next_boss < warp {
            next_boss += waves.bosses.every;
            bosses_sent += 1;
        }
        let stage = waves.stage_at(warp);
        Director {
            credits: waves.credits * 2.0,
            next_event: waves.events.iter().map(|e| e.first_after(warp)).collect(),
            pending: Vec::new(),
            warp,
            next_boss,
            bosses_sent,
            stage,
            prev_stage: stage,
            stage_age: 0.0,
            banner: Some(Banner {
                text: waves.stages[stage].name.clone(),
                color: palette::CREAM,
                time: BANNER_TIME,
            }),
        }
    }

    /// The director's clock: run time plus warp.
    pub fn clock(&self, w: &World) -> f32 {
        w.time + self.warp
    }
}

pub fn update(w: &mut World, dt: f32) {
    let content = content::get();
    let (waves, enemies) = (&content.waves, &content.enemies);
    let mut d = std::mem::take(&mut w.director);
    let t = d.clock(w);
    let minutes = t / 60.0;

    // Stage and banner.
    d.stage_age += dt;
    let stage = waves.stage_at(t);
    if stage != d.stage {
        d.prev_stage = d.stage;
        d.stage = stage;
        d.stage_age = 0.0;
        d.banner =
            Some(Banner { text: waves.stages[stage].name.clone(), color: palette::CREAM, time: BANNER_TIME });
    }
    if let Some(b) = &mut d.banner {
        b.time -= dt;
        if b.time <= 0.0 {
            d.banner = None;
        }
    }

    // Bosses.
    if t >= d.next_boss && !waves.boss_kinds.is_empty() {
        let kind = waves.boss_kinds[d.bosses_sent % waves.boss_kinds.len()];
        d.bosses_sent += 1;
        d.next_boss += waves.bosses.every;
        d.banner = Some(Banner { text: enemies[kind].name.clone(), color: palette::RED, time: BANNER_TIME });
        let pos = spawn_point(w, 20.0);
        d.pending.push(Pending { kind, pos, timer: BOSS_WARN, elite: None, hp_scale: 1.0 });
    }
    let boss_up = w.enemies.iter().any(|e| !e.dead && enemies[e.kind].boss().is_some());

    // Regular spawns bought with credits.
    let rate =
        (waves.credits + waves.credits_per_min * minutes) * if boss_up { waves.bosses.calm } else { 1.0 };
    d.credits += rate * dt;
    let el = &waves.elites;
    let elite_chance =
        if t < el.from { 0.0 } else { (el.chance + el.per_min * (t - el.from) / 60.0).min(el.max) };
    let pool = &waves.stages[stage].kinds;
    let weights: Vec<f32> = pool.iter().map(|k| k.1).collect();
    while w.enemies.len() + d.pending.len() < waves.max_alive {
        let Some(pick) = w.rng.weighted(&weights) else { break };
        let kind = pool[pick].0;
        let def = &enemies[kind];
        let elite = (def.cost >= 1.0 && w.rng.chance(elite_chance))
            .then(|| Elite::ALL[w.rng.below(Elite::ALL.len())]);
        let cost = def.cost * if elite.is_some() { el.cost } else { 1.0 };
        if d.credits < cost {
            break;
        }
        d.credits -= cost;
        let pos = spawn_point(w, 0.0);
        queue(w, &mut d, kind, pos, elite);
    }
    d.credits = d.credits.min(40.0);

    // Scripted groups.
    for (i, ev) in waves.events.iter().enumerate() {
        if t < d.next_event[i] {
            continue;
        }
        d.next_event[i] = if ev.every > 0.0 { d.next_event[i] + ev.every } else { f32::INFINITY };
        let count = ev.count + (minutes * ev.count as f32 * 0.15) as u32;
        let room = waves.max_alive.saturating_sub(w.enemies.len() + d.pending.len()) as u32;
        let count = count.min(room);
        match ev.shape {
            Shape::Ring => {
                let r = w.view.1 as f32 * 0.62;
                let base = w.rng.angle();
                for k in 0..count {
                    let a = base + k as f32 / count as f32 * std::f32::consts::TAU;
                    let pos = w.arena.clamp(w.player.pos + Vec2::from_angle(a) * r, 8.0);
                    queue(w, &mut d, ev.kind, pos, None);
                }
            }
            Shape::Cluster => {
                let center = spawn_point(w, 12.0);
                for _ in 0..count {
                    let off = Vec2::from_angle(w.rng.angle()) * w.rng.range(0.0, 14.0);
                    let pos = w.arena.clamp(center + off, 8.0);
                    queue(w, &mut d, ev.kind, pos, None);
                }
            }
        }
    }

    // Spawn whatever finished its warning.
    let (hp_mul, speed_mul) = (waves.hp_mul(t), waves.speed_mul(t));
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
        let mut e = Enemy::new(uid, p.kind, p.pos, hp_mul * p.hp_scale, speed_mul, phase);
        if let Some(el) = p.elite {
            e.make_elite(el, waves.elites.hp);
        }
        w.enemies.push(e);
    }
    w.director = d;
}

/// Enemies called in by other enemies (summons, splits). Respects the cap.
pub fn summon(w: &mut World, kind: usize, pos: Vec2, count: u32, spread: f32, warn: f32, hp_scale: f32) {
    let waves = &content::get().waves;
    let room = waves.max_alive.saturating_sub(w.enemies.len() + w.director.pending.len());
    let base = w.rng.angle();
    for k in 0..(count as usize).min(room) {
        let a = base + k as f32 / count as f32 * std::f32::consts::TAU;
        let at = w.arena.clamp(pos + Vec2::from_angle(a) * spread, 8.0);
        w.director.pending.push(Pending { kind, pos: at, timer: warn, elite: None, hp_scale });
    }
}

fn queue(w: &mut World, d: &mut Director, kind: usize, pos: Vec2, elite: Option<Elite>) {
    let timer = if w.camera.sees(pos, 6.0) { WARN_TIME } else { 0.0 };
    d.pending.push(Pending { kind, pos, timer, elite, hp_scale: 1.0 });
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

#[cfg(test)]
mod tests {
    #[test]
    fn stages_loop_forever() {
        let waves = &crate::content::get().waves;
        let (n, len) = (waves.stages.len(), waves.stage_length);
        assert_eq!(waves.stage_at(0.0), 0);
        assert_eq!(waves.stage_at(len * (n - 1) as f32 + 1.0), n - 1);
        assert_eq!(waves.stage_at(len * n as f32 + 1.0), waves.loop_from);
        for k in 0..500 {
            let s = waves.stage_at(k as f32 * len * 0.7);
            assert!(s < n && (k as f32 * 0.7 < n as f32 || s >= waves.loop_from));
        }
    }
}
