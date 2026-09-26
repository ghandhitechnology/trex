//! Item weapons: extra attacks that fire on their own and scale with the
//! player's stats. Also owns the runtime objects items create: mines,
//! meteors, beams, timed buffs, and the synergy banner.
//!
//! Stat rules shared by every weapon: damage is a ratio of Damage, attack
//! rate scales with FireRate, Shots adds to every count, radii scale with
//! Area, lifetimes with Duration, every hit can crit and fires Hit triggers.

use std::f32::consts::TAU;

use serde::Deserialize;

use super::effects::{self, GameEvent};
use super::synergy::Upgrade;
use super::{Action, On, Source, Stat, StatMod, apply_mods};
use crate::content;
use crate::engine::Vec2;
use crate::game::Controls;
use crate::game::player;
use crate::game::world::{Hit, Shot, World};
use crate::render::palette;
use crate::render::sprite::SpriteId;

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WeaponDef {
    pub kind: WeaponKind,
    /// Projectile, blade, mine, or meteor sprite. Beams, zaps, and auras take
    /// their colors from it.
    pub sprite: String,
    /// Ratio of the Damage stat per hit.
    pub damage: f32,
    /// Attacks per second, scaled by FireRate. Orbit: hits per second on one enemy.
    #[serde(default = "one")]
    pub rate: f32,
    /// Projectiles, blades, zaps, beams, mines, or meteors per attack.
    #[serde(default = "one_u")]
    pub count: u32,
    /// Actions run at every enemy this weapon hits.
    #[serde(default)]
    pub on_hit: Vec<Action>,
    /// Action run where a shot ends, a mine or meteor lands, or a beam or zap ends.
    #[serde(default)]
    pub on_end: Option<Action>,
    #[serde(skip)]
    pub sprite_id: SpriteId,
}

fn one() -> f32 {
    1.0
}

fn one_u() -> u32 {
    1
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum WeaponKind {
    /// Shots at the nearest enemy in a fan of `arc` degrees. 360 fires all around.
    Gun {
        #[serde(default)]
        arc: f32,
        /// ShotSpeed multiplier.
        #[serde(default = "one")]
        speed: f32,
        /// Range multiplier.
        #[serde(default = "one")]
        range: f32,
        #[serde(default)]
        pierce: u32,
        #[serde(default)]
        bounce: u32,
        #[serde(default)]
        motion: Motion,
    },
    /// Blades circling the player at `radius`, turning `spin` rad/s.
    Orbit { radius: f32, spin: f32 },
    /// Lightning at the nearest enemy that jumps `jumps` times.
    Zap { jumps: u32, range: f32 },
    /// An instant line that hits everything along it.
    Beam { length: f32, width: f32 },
    /// Mines dropped at the player's feet. They blow up on touch or after `secs`.
    Mine { radius: f32, secs: f32 },
    /// Hurts and slows everything near the player, `rate` times a second.
    Aura {
        radius: f32,
        #[serde(default)]
        slow: f32,
    },
    /// Rocks that fall on random nearby enemies and explode.
    Meteor { radius: f32 },
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Motion {
    #[default]
    Straight,
    /// Turns toward enemies.
    Homing,
    /// Flies out, turns around, and comes back through everything.
    Boomerang,
}

/// Extra data on a projectile, for item weapons and effects.
#[derive(Clone, Copy, Debug, Default)]
pub struct ShotTag {
    pub source: Source,
    /// Index into `Gear::weapons` for shots from item weapons.
    pub weapon: Option<u8>,
    pub motion: Motion,
    /// Boomerang: age at which it heads back.
    pub turn: f32,
}

/// One held weapon with its stacks and synergy upgrades applied.
#[derive(Clone, Debug)]
pub struct Armed {
    pub item: usize,
    pub kind: WeaponKind,
    pub sprite: SpriteId,
    pub damage: f32,
    pub rate: f32,
    pub count: u32,
    /// Radius multiplier from stacks and upgrades (Area applies on top).
    pub area: f32,
    pub pierce: u32,
    pub bounce: u32,
    pub on_hit: Vec<Action>,
    pub on_end: Option<Action>,
    pub cd: f32,
    /// Orbit angle, radians.
    pub spin: f32,
    /// Orbit: enemies hit recently, with seconds until they can be hit again.
    touched: Vec<(u32, f32)>,
}

/// Each stack past the first adds +10% damage, and stacks 2 and 4 add +1
/// count. Auras grow 15% in radius per stack instead of count.
const STACK_DAMAGE: f32 = 0.1;
const STACK_AURA_AREA: f32 = 0.15;

impl Armed {
    fn new(item: usize, def: &WeaponDef, stacks: u32) -> Self {
        let extra = stacks.saturating_sub(1);
        let aura = matches!(def.kind, WeaponKind::Aura { .. });
        Armed {
            item,
            kind: def.kind,
            sprite: def.sprite_id,
            damage: def.damage * (1.0 + STACK_DAMAGE * extra as f32),
            rate: def.rate,
            count: def.count + if aura { 0 } else { extra.div_ceil(2) },
            area: 1.0 + if aura { STACK_AURA_AREA * extra as f32 } else { 0.0 },
            pierce: 0,
            bounce: 0,
            on_hit: def.on_hit.clone(),
            on_end: def.on_end,
            cd: 0.3,
            spin: 0.0,
            touched: Vec::new(),
        }
    }

    fn upgrade(&mut self, u: &Upgrade) {
        if let Some(s) = u.sprite_id {
            self.sprite = s;
        }
        self.count += u.count;
        self.damage *= 1.0 + u.damage;
        self.area += u.area;
        self.rate *= 1.0 + u.rate;
        self.pierce += u.pierce;
        self.bounce += u.bounce;
        self.on_hit.extend(u.on_hit.iter().copied());
        if u.on_end.is_some() {
            self.on_end = u.on_end;
        }
    }
}

pub struct Mine {
    pub pos: Vec2,
    pub arm: f32,
    pub life: f32,
    pub radius: f32,
    pub damage: f32,
    pub weapon: Option<usize>,
    pub depth: u8,
    pub sprite: SpriteId,
}

pub struct Meteor {
    pub pos: Vec2,
    pub fall: f32,
    pub radius: f32,
    pub damage: f32,
    pub weapon: Option<usize>,
    pub depth: u8,
    pub sprite: SpriteId,
}

/// A beam's afterglow; the damage is already dealt.
pub struct Beam {
    pub dir: Vec2,
    pub len: f32,
    pub width: f32,
    pub life: f32,
    pub sprite: SpriteId,
}

pub struct Buff {
    pub m: StatMod,
    pub time: f32,
}

pub const BANNER_TIME: f32 = 2.4;
pub const FALL_TIME: f32 = 0.55;
pub const BEAM_TIME: f32 = 0.22;
const MAX_MINES: usize = 40;
const MINE_TOUCH: f32 = 7.0;
const BLADE_RADIUS: f32 = 4.5;
const METEOR_REACH: f32 = 150.0;

/// Item runtime for one run.
#[derive(Default)]
pub struct Gear {
    pub weapons: Vec<Armed>,
    pub mines: Vec<Mine>,
    pub meteors: Vec<Meteor>,
    pub beams: Vec<Beam>,
    pub buffs: Vec<Buff>,
    /// Seconds of shield left (drawn as a bubble).
    pub shield: f32,
    /// Active synergies, to spot new ones.
    pub synergies: Vec<usize>,
    /// Newly formed synergy and seconds left on its banner.
    pub banner: Option<(usize, f32)>,
    /// Aura pulse clock for drawing.
    pub pulse: f32,
    /// Character FireRate before items; weapon rates scale by FireRate / this.
    base_rate: f32,
}

impl Gear {
    fn haste(&self, stats: &super::Stats) -> f32 {
        (stats.get(Stat::FireRate) / self.base_rate.max(0.1)).max(0.1)
    }
}

/// Rebuild weapons and reapply buffs after the build or buffs changed.
/// Called at the end of `World::refresh_build`.
pub fn refresh(w: &mut World) {
    let content = content::get();
    if !w.gear.buffs.is_empty() {
        let mods: Vec<StatMod> = w.gear.buffs.iter().map(|b| b.m.clone()).collect();
        w.stats = apply_mods(&w.stats, &mods);
    }
    let ch = &content.characters[w.character];
    w.gear.base_rate = ch.base.get(&Stat::FireRate).copied().unwrap_or(Stat::FireRate.default_value());

    let synergies: Vec<usize> = w.build.synergies(content).collect();
    let old = std::mem::take(&mut w.gear.weapons);
    for &(item, stacks) in &w.build.items {
        let Some(def) = &content.items[item].weapon else { continue };
        let mut a = Armed::new(item, def, stacks);
        for &k in &synergies {
            for u in content.synergies[k].upgrades.iter().filter(|u| u.item == item) {
                a.upgrade(u);
            }
        }
        if let Some(o) = old.iter().find(|o| o.item == item) {
            a.cd = o.cd;
            a.spin = o.spin;
        }
        w.gear.weapons.push(a);
    }

    if let Some(&new) = synergies.iter().find(|k| !w.gear.synergies.contains(k)) {
        w.gear.banner = Some((new, BANNER_TIME));
        let pos = w.player.pos;
        w.fx.ring(pos, 46.0, palette::GOLD);
        w.fx.burst(pos, &[palette::GOLD, palette::CREAM, palette::AMBER], 30, 150.0);
        w.fx.shake(0.3);
    }
    w.gear.synergies = synergies;
}

/// Fire weapons and advance mines, meteors, beams, buffs. Runs every tick
/// after the player's own weapon.
pub fn update(w: &mut World, c: &Controls, dt: f32) {
    if c.dash {
        w.events.push_back(GameEvent::at(On::Active, w.player.pos, 0));
    }

    for b in &mut w.gear.buffs {
        b.time -= dt;
    }
    let buffs = w.gear.buffs.len();
    w.gear.buffs.retain(|b| b.time > 0.0);
    if w.gear.buffs.len() != buffs {
        w.refresh_build();
    }
    w.gear.shield = (w.gear.shield - dt).max(0.0);
    w.gear.pulse += dt;
    if let Some((_, t)) = &mut w.gear.banner {
        *t -= dt;
        if *t <= 0.0 {
            w.gear.banner = None;
        }
    }

    let haste = w.gear.haste(&w.stats);
    for k in 0..w.gear.weapons.len() {
        let a = &mut w.gear.weapons[k];
        for t in &mut a.touched {
            t.1 -= dt * haste;
        }
        a.touched.retain(|t| t.1 > 0.0);
        if let WeaponKind::Orbit { spin, .. } = a.kind {
            a.spin = (a.spin + spin * dt).rem_euclid(TAU);
            orbit(w, k);
            continue;
        }
        a.cd -= dt * haste;
        if a.cd > 0.0 {
            continue;
        }
        let fired = match a.kind {
            WeaponKind::Gun { arc, speed, range, pierce, bounce, motion } => {
                gun(w, k, arc, speed, range, pierce, bounce, motion)
            }
            WeaponKind::Zap { jumps, range } => zap(w, k, jumps, range),
            WeaponKind::Beam { length, width } => beam(w, k, length, width),
            WeaponKind::Mine { radius, secs } => {
                drop_mines(w, k, radius, secs);
                true
            }
            WeaponKind::Aura { radius, slow } => {
                aura(w, k, radius, slow);
                true
            }
            WeaponKind::Meteor { radius } => call_meteors(w, k, radius),
            WeaponKind::Orbit { .. } => true,
        };
        let a = &mut w.gear.weapons[k];
        a.cd = if fired { a.cd + 1.0 / a.rate.max(0.05) } else { 0.0 };
    }

    update_mines(w, dt);
    update_meteors(w, dt);
    for b in &mut w.gear.beams {
        b.life -= dt;
    }
    w.gear.beams.retain(|b| b.life > 0.0);
}

/// Count for one attack: the weapon's count plus the Shots stat bonus.
fn count(w: &World, k: usize) -> u32 {
    w.gear.weapons[k].count + w.stats.count(Stat::Shots).saturating_sub(1)
}

/// Radius after the weapon's own multiplier and Area.
fn scaled(w: &World, k: usize, r: f32) -> f32 {
    r * w.gear.weapons[k].area * w.stats.get(Stat::Area)
}

/// A hit from an item weapon (`weapon` set) or an item effect. Weapon hits
/// roll crits and run the weapon's on-hit actions.
pub fn hit(
    w: &mut World,
    i: usize,
    damage: f32,
    knock: Vec2,
    source: Source,
    depth: u8,
    weapon: Option<usize>,
) {
    if w.enemies[i].dead {
        return;
    }
    let crit = weapon.is_some() && w.rng.chance(w.stats.get(Stat::Crit));
    let damage = if crit { damage * w.stats.get(Stat::CritDamage) } else { damage };
    let pos = w.enemies[i].pos;
    w.damage_enemy(i, Hit { damage, knock, crit, depth, procs: true, source });
    if let Some(k) = weapon {
        on_hit(w, k, i, pos, source, depth);
    }
}

/// Run weapon `k`'s on-hit actions at enemy `i`.
pub fn on_hit(w: &mut World, k: usize, i: usize, pos: Vec2, source: Source, depth: u8) {
    for j in 0..w.gear.weapons.get(k).map_or(0, |a| a.on_hit.len()) {
        let action = w.gear.weapons[k].on_hit[j];
        let ev = GameEvent { on: On::Hit, pos, target: Some(i), depth, source };
        effects::run(w, &action, &ev);
    }
}

fn on_end(w: &mut World, k: usize, pos: Vec2, source: Source, depth: u8) {
    if let Some(action) = w.gear.weapons.get(k).and_then(|a| a.on_end) {
        let ev = GameEvent { on: On::Hit, pos, target: None, depth, source };
        effects::run(w, &action, &ev);
    }
}

#[allow(clippy::too_many_arguments)]
fn gun(
    w: &mut World,
    k: usize,
    arc: f32,
    speed: f32,
    range: f32,
    pierce: u32,
    bounce: u32,
    motion: Motion,
) -> bool {
    let pos = w.player.pos;
    let n = count(w, k);
    let reach = w.stats.get(Stat::Range) * range;
    let dirs: Vec<Vec2> = if arc >= 360.0 {
        let base = w.time * 1.7;
        (0..n).map(|j| Vec2::from_angle(base + j as f32 / n as f32 * TAU)).collect()
    } else {
        let Some(t) = w.nearest_enemy(pos, reach, |_| true) else { return false };
        let aim = (w.enemies[t].pos - pos).norm();
        let step = if n > 1 { arc.to_radians() / (n - 1) as f32 } else { 0.0 };
        (0..n).map(|j| aim.rotate((j as f32 - (n - 1) as f32 / 2.0) * step)).collect()
    };
    let a = &w.gear.weapons[k];
    let (ratio, sprite, extra_pierce, extra_bounce) = (a.damage, a.sprite, a.pierce, a.bounce);
    for dir in dirs {
        player::spawn_shot(w, pos, dir, ratio, 0, sprite);
        let s = w.shots.last_mut().expect("shot just spawned");
        tag_gun(s, k, speed, range, pierce + extra_pierce, bounce + extra_bounce, motion);
    }
    true
}

fn tag_gun(s: &mut Shot, k: usize, speed: f32, range: f32, pierce: u32, bounce: u32, motion: Motion) {
    s.vel *= speed;
    s.life *= range / speed.max(0.05);
    s.pierce += pierce;
    s.bounce += bounce;
    let mut turn = 0.0;
    match motion {
        Motion::Straight => {}
        Motion::Homing => s.homing = s.homing.max(4.5),
        Motion::Boomerang => {
            turn = s.life / 1.15 * 0.8;
            s.life = turn * 3.0 + 0.5;
            s.pierce = u32::MAX / 2;
        }
    }
    s.tag = ShotTag { source: Source::Gun, weapon: Some(k as u8), motion, turn };
}

/// Per-tick motion for tagged shots: boomerangs head home after `turn`.
pub fn steer(w: &World, s: &mut Shot, dt: f32) {
    if s.tag.motion != Motion::Boomerang || s.age < s.tag.turn {
        return;
    }
    if s.age - dt < s.tag.turn {
        s.nhits = 0;
    }
    let to = w.player.pos - s.pos;
    let speed = s.vel.len().max(60.0);
    s.vel = s.vel.lerp(to.norm() * speed * 1.08, crate::engine::damp(9.0, dt));
    if to.len_sq() < 64.0 {
        s.dead = true;
    }
}

/// End actions for item shots that died this tick. Called before dead shots
/// are removed.
pub fn shots_ended(w: &mut World) {
    let ended: Vec<(usize, Vec2, u8)> = w
        .shots
        .iter()
        .filter(|s| s.dead && !s.hostile)
        .filter_map(|s| s.tag.weapon.map(|k| (k as usize, s.pos, s.depth)))
        .filter(|(k, _, _)| w.gear.weapons.get(*k).is_some_and(|a| a.on_end.is_some()))
        .collect();
    for (k, pos, depth) in ended {
        on_end(w, k, pos, Source::Gun, depth);
    }
}

/// World positions of an orbit weapon's blades.
pub fn blades(w: &World, k: usize) -> Vec<Vec2> {
    let WeaponKind::Orbit { radius, .. } = w.gear.weapons[k].kind else { return Vec::new() };
    let n = count(w, k);
    let r = scaled(w, k, radius);
    let spin = w.gear.weapons[k].spin;
    (0..n).map(|b| w.player.pos + Vec2::from_angle(spin + b as f32 / n as f32 * TAU) * r).collect()
}

fn orbit(w: &mut World, k: usize) {
    let size = BLADE_RADIUS * w.stats.get(Stat::ShotSize);
    let hit_gap = 1.0 / w.gear.weapons[k].rate.max(0.05);
    let damage = w.gear.weapons[k].damage * w.stats.get(Stat::Damage);
    let knock = w.stats.get(Stat::Knockback);
    for pos in blades(w, k) {
        for i in w.enemies_in(pos, size) {
            let uid = w.enemies[i].uid;
            if w.gear.weapons[k].touched.iter().any(|t| t.0 == uid) {
                continue;
            }
            w.gear.weapons[k].touched.push((uid, hit_gap));
            let push = (w.enemies[i].pos - w.player.pos).norm() * knock;
            hit(w, i, damage, push, Source::Orbit, 0, Some(k));
            w.fx.burst(pos, &[palette::BONE, palette::FOG], 2, 40.0);
        }
    }
}

fn zap(w: &mut World, k: usize, jumps: u32, range: f32) -> bool {
    let reach = scaled(w, k, range);
    let damage = w.gear.weapons[k].damage * w.stats.get(Stat::Damage);
    let mut struck: Vec<u32> = Vec::new();
    let mut fired = false;
    for _ in 0..count(w, k) {
        let mut from = w.player.pos;
        let mut hops = 0;
        while hops <= jumps {
            let Some(i) = w.nearest_enemy(from, reach, |e| !struck.contains(&e.uid)) else { break };
            let to = w.enemies[i].pos;
            struck.push(w.enemies[i].uid);
            w.fx.bolt(from, to);
            hit(w, i, damage, Vec2::ZERO, Source::Zap, 0, Some(k));
            from = to;
            hops += 1;
            fired = true;
        }
        if hops > 0 {
            on_end(w, k, from, Source::Zap, 0);
        }
    }
    fired
}

fn beam(w: &mut World, k: usize, length: f32, width: f32) -> bool {
    let pos = w.player.pos;
    let len = length * w.gear.weapons[k].area;
    let half = scaled(w, k, width) / 2.0;
    let damage = w.gear.weapons[k].damage * w.stats.get(Stat::Damage);
    let knock = w.stats.get(Stat::Knockback) * 0.5;
    let sprite = w.gear.weapons[k].sprite;
    let enemies = &content::get().enemies;
    let mut aimed: Vec<u32> = Vec::new();
    for _ in 0..count(w, k) {
        let Some(t) = w.nearest_enemy(pos, len, |e| !aimed.contains(&e.uid)) else { break };
        aimed.push(w.enemies[t].uid);
        let dir = (w.enemies[t].pos - pos).norm();
        let targets: Vec<usize> = (0..w.enemies.len())
            .filter(|&i| {
                let e = &w.enemies[i];
                let d = e.pos - pos;
                let along = d.x * dir.x + d.y * dir.y;
                let across = (d.x * dir.y - d.y * dir.x).abs();
                !e.dead && along > -4.0 && along < len && across < half + enemies[e.kind].radius
            })
            .collect();
        for i in targets {
            hit(w, i, damage, dir * knock, Source::Beam, 0, Some(k));
        }
        w.gear.beams.push(Beam { dir, len, width: half * 2.0, life: BEAM_TIME, sprite });
        w.fx.burst(pos + dir * 6.0, &[palette::CREAM, palette::BONE], 4, 60.0);
        on_end(w, k, pos + dir * len, Source::Beam, 0);
    }
    !aimed.is_empty()
}

fn drop_mines(w: &mut World, k: usize, radius: f32, secs: f32) {
    let a = &w.gear.weapons[k];
    let (ratio, sprite) = (a.damage, a.sprite);
    let n = count(w, k);
    let r = scaled(w, k, radius);
    let pos = w.player.pos;
    for j in 0..n {
        let off = if n > 1 { Vec2::from_angle(w.rng.angle()) * (4.0 + j as f32 * 3.0) } else { Vec2::ZERO };
        let life = secs * w.stats.get(Stat::Duration);
        let damage = ratio * w.stats.get(Stat::Damage);
        place_mine(
            w,
            Mine { pos: pos + off, arm: 0.5, life, radius: r, damage, weapon: Some(k), depth: 0, sprite },
        );
    }
}

pub fn place_mine(w: &mut World, m: Mine) {
    if w.gear.mines.len() >= MAX_MINES {
        w.gear.mines.remove(0);
    }
    w.gear.mines.push(m);
}

fn update_mines(w: &mut World, dt: f32) {
    let enemies = &content::get().enemies;
    let mut j = 0;
    while j < w.gear.mines.len() {
        let m = &mut w.gear.mines[j];
        m.arm -= dt;
        m.life -= dt;
        let (pos, armed, expired) = (m.pos, m.arm <= 0.0, m.life <= 0.0);
        let touched = armed
            && w.nearest_enemy(pos, MINE_TOUCH + 10.0, |e| {
                e.pos.dist_sq(pos) < (MINE_TOUCH + enemies[e.kind].radius).powi(2)
            })
            .is_some();
        if touched || expired {
            let m = w.gear.mines.swap_remove(j);
            blast(w, m.pos, m.radius, m.damage, Source::Mine, m.depth, m.weapon);
            if let Some(k) = m.weapon {
                on_end(w, k, m.pos, Source::Mine, m.depth);
            }
        } else {
            j += 1;
        }
    }
}

fn aura(w: &mut World, k: usize, radius: f32, slow: f32) {
    let r = scaled(w, k, radius);
    let damage = w.gear.weapons[k].damage * w.stats.get(Stat::Damage);
    let pos = w.player.pos;
    let secs = w.stats.get(Stat::Duration);
    for i in w.enemies_in(pos, r) {
        if slow > 0.0 {
            let e = &mut w.enemies[i];
            e.slow = e.slow.max(slow.min(0.85));
            e.slow_time = e.slow_time.max(secs);
        }
        if damage > 0.0 {
            hit(w, i, damage, Vec2::ZERO, Source::Aura, 0, Some(k));
        }
    }
}

fn call_meteors(w: &mut World, k: usize, radius: f32) -> bool {
    let a = &w.gear.weapons[k];
    let (ratio, sprite) = (a.damage, a.sprite);
    let r = scaled(w, k, radius);
    let n = count(w, k);
    let damage = ratio * w.stats.get(Stat::Damage);
    rain(w, n, r, damage, Some(k), 0, sprite)
}

/// Drop `n` meteors on random enemies near the player. False if none are near.
pub fn rain(
    w: &mut World,
    n: u32,
    radius: f32,
    damage: f32,
    weapon: Option<usize>,
    depth: u8,
    sprite: SpriteId,
) -> bool {
    let pos = w.player.pos;
    let mut near: Vec<usize> = (0..w.enemies.len())
        .filter(|&i| !w.enemies[i].dead && w.enemies[i].pos.dist_sq(pos) < METEOR_REACH * METEOR_REACH)
        .collect();
    if near.is_empty() {
        return false;
    }
    for j in 0..n {
        let target = if near.is_empty() {
            pos + Vec2::from_angle(w.rng.angle()) * w.rng.range(20.0, 80.0)
        } else {
            let pick = w.rng.below(near.len());
            w.enemies[near.swap_remove(pick)].pos
        };
        let fall = FALL_TIME + j as f32 * 0.07;
        w.gear.meteors.push(Meteor { pos: target, fall, radius, damage, weapon, depth, sprite });
    }
    true
}

fn update_meteors(w: &mut World, dt: f32) {
    let mut j = 0;
    while j < w.gear.meteors.len() {
        w.gear.meteors[j].fall -= dt;
        if w.gear.meteors[j].fall > 0.0 {
            j += 1;
            continue;
        }
        let m = w.gear.meteors.swap_remove(j);
        blast(w, m.pos, m.radius, m.damage, Source::Meteor, m.depth, m.weapon);
        w.fx.debris(m.pos, &[palette::CLAY, palette::UMBER, palette::SAND], 8);
        if let Some(k) = m.weapon {
            on_end(w, k, m.pos, Source::Meteor, m.depth);
        }
    }
}

/// Damage everything in a radius, with explosion effects.
pub fn blast(
    w: &mut World,
    pos: Vec2,
    radius: f32,
    damage: f32,
    source: Source,
    depth: u8,
    weapon: Option<usize>,
) {
    for i in w.enemies_in(pos, radius) {
        let knock = (w.enemies[i].pos - pos).norm() * 90.0;
        hit(w, i, damage, knock, source, depth, weapon);
    }
    w.fx.ring(pos, radius, palette::EMBER);
    w.fx.burst(pos, &[palette::GOLD, palette::AMBER, palette::EMBER, palette::RED], 18, radius * 3.0);
    w.fx.shake(0.12);
}

/// Add a timed stat buff and recompute stats.
pub fn buff(w: &mut World, m: StatMod, secs: f32) {
    w.gear.buffs.push(Buff { m, time: secs });
    w.refresh_build();
}
