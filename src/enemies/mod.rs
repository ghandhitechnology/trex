//! Enemy definitions, runtime state, and per-tick enemy simulation.

pub mod ai;
pub mod boss;
pub mod death;
pub mod director;
pub mod draw;
pub mod sprites;

use serde::Deserialize;

use crate::content;
use crate::engine::Vec2;
use crate::game::player;
use crate::game::world::{Gem, Hit, Shot, World};
use crate::render::palette::{self, Color};
use crate::render::sprite::{SpriteId, bank};
use ai::{Act, Ctx};
use boss::{BossKind, Move};
use death::Death;

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
    /// Shown on the boss bar.
    #[serde(default)]
    pub name: String,
    /// Projectile sprite for anything this enemy fires.
    #[serde(default)]
    pub shot: Option<String>,
    /// Enemy spawned by `Summon` and boss broods.
    #[serde(default)]
    pub minion: Option<String>,
    /// Enemies this one bursts into on death.
    #[serde(default)]
    pub split: Option<Split>,
    /// Shield HP that soaks damage and regrows after a short pause.
    #[serde(default)]
    pub shield: f32,
    #[serde(default)]
    pub death: Death,
    #[serde(skip)]
    pub sprite_id: SpriteId,
    #[serde(skip)]
    pub shot_id: SpriteId,
    #[serde(skip)]
    pub minion_kind: usize,
    #[serde(skip)]
    pub split_kind: usize,
}

impl EnemyDef {
    pub fn boss(&self) -> Option<BossKind> {
        match self.behavior {
            Behavior::Boss(k) => Some(k),
            _ => None,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Split {
    pub enemy: String,
    pub count: u32,
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
fn default_windup() -> f32 {
    0.45
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
    /// Keep distance, flash, then fire `pattern`.
    Shoot {
        range: f32,
        cooldown: f32,
        speed: f32,
        #[serde(default)]
        pattern: Pattern,
        #[serde(default = "default_windup")]
        windup: f32,
    },
    /// Circle the player, then dive through them.
    Orbit { radius: f32, spin: f32, dive: f32 },
    /// Travel underground, surface near the player with a ring of `shots`.
    Burrow { under: f32, up: f32, shots: u32 },
    /// Vanish and reappear near the player, then fire a fan of `shots`.
    Blink { range: f32, cooldown: f32, windup: f32, shots: u32 },
    /// Rush in, light a fuse, explode. Hurts enemies too.
    Kamikaze { range: f32, fuse: f32, radius: f32, damage: i32 },
    /// Wind up and pound the ground around itself.
    Slam { range: f32, windup: f32, radius: f32, damage: i32, cooldown: f32 },
    /// Keep distance and call in `count` minions.
    Summon { count: u32, cooldown: f32, range: f32 },
    /// Scripted multi-phase boss.
    Boss(BossKind),
}

#[derive(Deserialize, Debug, Default, Clone, Copy)]
pub enum Pattern {
    /// One shot at the player.
    #[default]
    Single,
    /// `count` shots fanned over `angle` degrees.
    Spread { count: u32, angle: f32 },
    /// `count` shots evenly around.
    Ring { count: u32 },
    /// `count` aimed shots, `gap` seconds apart.
    Burst { count: u32, gap: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AiState {
    #[default]
    Move,
    Windup,
    Act,
    Recover,
}

/// Elite modifiers. Elites have more HP, triple XP, and a colored aura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elite {
    Swift,
    Tough,
    Shielded,
    Volatile,
    Splitting,
}

impl Elite {
    pub const ALL: [Elite; 5] =
        [Elite::Swift, Elite::Tough, Elite::Shielded, Elite::Volatile, Elite::Splitting];

    pub fn color(self) -> Color {
        match self {
            Elite::Swift => palette::CYAN,
            Elite::Tough => palette::GOLD,
            Elite::Shielded => palette::ICE,
            Elite::Volatile => palette::EMBER,
            Elite::Splitting => palette::LIME,
        }
    }
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
    /// Underground or mid-teleport: no collisions, no damage taken.
    pub hidden: bool,
    pub shield: f32,
    pub shield_max: f32,
    /// Seconds until the shield starts to regrow.
    pub shield_cd: f32,
    pub elite: Option<Elite>,
    /// Target point for dives, teleports, eruptions, and aimed moves.
    pub aim: Vec2,
    /// Rotating angle for orbits and spirals.
    pub spin: f32,
    /// Shots or repeats left in the current action.
    pub count: u32,
    /// Secondary timer for repeats inside an action.
    pub clock: f32,
    /// Boss: current move, moves started, and rage level (0-2).
    pub mv: Move,
    pub step: u32,
    pub rage: u8,
}

impl Enemy {
    pub fn new(uid: u32, kind: usize, pos: Vec2, hp_mul: f32, speed_mul: f32, phase: f32) -> Self {
        let def = &content::get().enemies[kind];
        let mut e = Enemy {
            uid,
            kind,
            pos,
            hp: def.hp * hp_mul,
            max_hp: def.hp * hp_mul,
            speed: def.speed * speed_mul,
            phase,
            timer: phase,
            shield_max: def.shield * hp_mul,
            ..Default::default()
        };
        ai::init(&mut e, def);
        e.shield = e.shield_max;
        e
    }

    /// Promote to an elite with `hp` times the health.
    pub fn make_elite(&mut self, elite: Elite, hp: f32) {
        self.elite = Some(elite);
        self.max_hp *= hp;
        match elite {
            Elite::Swift => self.speed *= 1.5,
            Elite::Tough => self.max_hp *= 1.8,
            Elite::Shielded => self.shield_max += self.max_hp * 0.6,
            Elite::Volatile | Elite::Splitting => {}
        }
        self.hp = self.max_hp;
        self.shield = self.shield_max;
    }

    /// Can be hit, targeted, and touched.
    pub fn hittable(&self) -> bool {
        !self.dead && !self.hidden
    }
}

/// Resolve sprite and enemy references after loading.
pub fn resolve(defs: &mut [EnemyDef]) -> Result<(), String> {
    let ids: Vec<String> = defs.iter().map(|e| e.id.clone()).collect();
    let enemy = |owner: &str, id: &str| {
        ids.iter().position(|i| i == id).ok_or_else(|| format!("{owner}: unknown enemy `{id}`"))
    };
    let sprite =
        |owner: &str, name: &str| bank().id(name).ok_or_else(|| format!("{owner}: unknown sprite `{name}`"));
    for e in defs.iter_mut() {
        let owner = format!("enemy `{}`", e.id);
        e.sprite_id = sprite(&owner, &e.sprite)?;
        e.shot_id = sprite(&owner, e.shot.as_deref().unwrap_or("spit"))?;
        if let Some(m) = &e.minion {
            e.minion_kind = enemy(&owner, m)?;
        } else if matches!(e.behavior, Behavior::Summon { .. }) {
            return Err(format!("{owner}: Summon needs a `minion`"));
        }
        if let Some(s) = &e.split {
            e.split_kind = enemy(&owner, &s.enemy)?;
        }
    }
    Ok(())
}

const BURN_TICK: f32 = 0.25;
const SHIELD_DELAY: f32 = 2.5;
/// Seconds an enemy projectile lives.
const SHOT_LIFE: f32 = 3.0;

/// Statuses, AI, actions, separation, movement, and contact damage for every enemy.
pub fn update(w: &mut World, dt: f32) {
    let content = content::get();
    let target = w.player.pos;
    let mut burns: Vec<(usize, f32)> = Vec::new();
    let mut acts: Vec<(usize, Act)> = Vec::new();
    let mut out: Vec<Act> = Vec::new();

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
        if e.shield_max > 0.0 {
            e.shield_cd -= dt;
            if e.shield_cd <= 0.0 {
                e.shield = (e.shield + e.shield_max * 0.5 * dt).min(e.shield_max);
            }
        }

        let mut cx = Ctx { target, dt, arena: w.arena, rng: &mut w.rng, out: &mut out };
        let (vel, slow) = match def.behavior {
            Behavior::Boss(kind) => (boss::think(e, def, kind, &mut cx), e.slow * 0.4),
            _ => (ai::think(e, def, &mut cx), e.slow),
        };
        e.vel = vel * (1.0 - slow);
        acts.extend(out.drain(..).map(|a| (i, a)));
    }

    for (i, dmg) in burns {
        let pos = w.enemies[i].pos;
        w.damage_enemy(i, Hit { damage: dmg, procs: false, ..Hit::default() });
        if w.fx.enabled && w.fx.rng.chance(0.6) {
            w.fx.spark(pos + Vec2::new(0.0, -4.0), palette::AMBER, 20.0);
        }
    }

    for (i, act) in acts {
        apply(w, i, act);
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
        if !e.hittable() {
            continue;
        }
        let def = &content.enemies[e.kind];
        if e.pos.dist_sq(target) < (def.radius + player::RADIUS).powi(2) {
            let (dmg, from) = (def.damage, e.pos);
            player::hurt(w, dmg, from);
        }
    }
}

fn apply(w: &mut World, i: usize, act: Act) {
    match act {
        Act::Shot { pos, vel, sprite } => {
            let mut s = Shot::hostile(pos, vel, 1, sprite);
            s.life = SHOT_LIFE;
            w.shots.push(s);
        }
        Act::Spawn { kind, pos, count, spread } => director::summon(w, kind, pos, count, spread, 0.45, 1.0),
        Act::Blast { pos, radius, damage, friendly } => blast(w, pos, radius, damage, friendly),
        Act::Roar { pos } => {
            w.fx.shake(0.6);
            w.fx.ring(pos, 40.0, palette::RED);
            w.fx.ring(pos, 26.0, palette::GOLD);
        }
        Act::Impact { pos } => {
            w.fx.shake(0.35);
            w.fx.burst(pos, &[palette::FOG, palette::HAZE, palette::BONE], 16, 90.0);
        }
        Act::Vanish => {
            let e = &mut w.enemies[i];
            if !e.dead {
                e.dead = true;
                let (pos, kind) = (e.pos, e.kind);
                death::play(w, pos, kind, false);
            }
        }
    }
}

/// Explosion at `pos`: hurts the player inside `radius`, and enemies for `friendly` damage.
fn blast(w: &mut World, pos: Vec2, radius: f32, damage: i32, friendly: f32) {
    if w.player.pos.dist_sq(pos) < (radius + player::RADIUS).powi(2) {
        player::hurt(w, damage, pos);
    }
    if friendly > 0.0 {
        for j in w.enemies_in(pos, radius) {
            let knock = (w.enemies[j].pos - pos).norm() * 120.0;
            w.damage_enemy(j, Hit { damage: friendly, knock, crit: false, depth: 2, procs: false });
        }
    }
    w.fx.ring(pos, radius, palette::EMBER);
    w.fx.ring(pos, radius * 0.6, palette::GOLD);
    w.fx.burst(pos, &[palette::GOLD, palette::AMBER, palette::EMBER, palette::RED], 20, radius * 3.0);
    w.fx.shake(0.3);
}

/// Shield soaks damage first. Returns what gets through.
pub fn absorb(e: &mut Enemy, damage: f32) -> f32 {
    if e.shield <= 0.0 {
        return damage;
    }
    e.shield_cd = SHIELD_DELAY;
    let soaked = damage.min(e.shield);
    e.shield -= soaked;
    damage - soaked
}

/// Loot, on-death effects, and the death animation for enemy `i`, which just died.
pub fn died(w: &mut World, i: usize) {
    let content = content::get();
    let e = &w.enemies[i];
    let (pos, kind, elite, boss) = (e.pos, e.kind, e.elite, content.enemies[e.kind].boss().is_some());
    let def = &content.enemies[kind];

    let xp = def.xp * if elite.is_some() { 3 } else { 1 };
    let gems = if boss { 10 } else { 1 };
    for _ in 0..gems {
        let speed = if boss { w.rng.range(30.0, 110.0) } else { 30.0 };
        w.gems.push(Gem {
            pos,
            vel: Vec2::from_angle(w.rng.angle()) * speed,
            value: (xp / gems).max(1),
            pull: false,
            age: 0.0,
            dead: false,
        });
    }

    if boss {
        w.heal(4);
    }
    if let Some(s) = &def.split {
        director::summon(w, def.split_kind, pos, s.count, 6.0, 0.0, 1.0);
    }
    match elite {
        Some(Elite::Volatile) => {
            let sprite = bank().id("needle").unwrap_or(def.shot_id);
            let base = w.rng.angle();
            for k in 0..10 {
                let dir = Vec2::from_angle(base + k as f32 / 10.0 * std::f32::consts::TAU);
                w.shots.push(Shot::hostile(pos + dir * 4.0, dir * 55.0, 1, sprite));
            }
        }
        Some(Elite::Splitting) => director::summon(w, kind, pos, 2, 8.0, 0.0, 0.4),
        _ => {}
    }
    death::play(w, pos, kind, boss);
}

/// Soft push so crowds spread out instead of stacking on one pixel.
/// Heavier enemies give way less.
fn separate(w: &mut World, dt: f32) {
    let content = content::get();
    let n = w.enemies.len();
    let mut shove = vec![Vec2::ZERO; n];
    for (i, e) in w.enemies.iter().enumerate() {
        if !e.hittable() {
            continue;
        }
        let di = &content.enemies[e.kind];
        w.grid.query(e.pos, di.radius * 2.0, |j| {
            if j == i || j >= n {
                return;
            }
            let o = &w.enemies[j];
            if !o.hittable() {
                return;
            }
            let dj = &content.enemies[o.kind];
            let min = di.radius + dj.radius;
            let d = e.pos - o.pos;
            let dist = d.len();
            if dist < min {
                let dir = if dist > 1e-3 { d / dist } else { Vec2::from_angle(i as f32) };
                let give = dj.mass / (di.mass + dj.mass) * 2.0;
                shove[i] += dir * (min - dist) * give;
            }
        });
    }
    for (e, s) in w.enemies.iter_mut().zip(shove) {
        e.pos += s.clamp_len(60.0 * dt);
    }
}
