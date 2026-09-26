//! Runs item and passive triggers. Events queue up during a tick and are
//! processed together; actions can cause new events one generation deeper,
//! capped by `MAX_DEPTH` so chain reactions stay bounded.

use super::weapons::{self, Mine};
use super::{Action, On, Owner, Source, Stat, StatMod};
use crate::engine::Vec2;
use crate::game::player;
use crate::game::world::World;
use crate::render::palette;
use crate::render::sprite::bank;

/// Events from depth 0 (weapon) and depth 1 (first proc) can fire triggers.
pub const MAX_DEPTH: u8 = 1;
const EVENT_BUDGET: usize = 3000;
/// Most burn damage per second on one enemy, as a ratio of Damage.
const BURN_CAP: f32 = 2.5;

#[derive(Clone, Copy, Debug)]
pub struct GameEvent {
    pub on: On,
    pub pos: Vec2,
    /// Enemy index for Hit/Crit/Kill.
    pub target: Option<usize>,
    pub depth: u8,
    /// What dealt the hit, for Hit/Crit/Kill.
    pub source: Source,
}

impl GameEvent {
    pub fn at(on: On, pos: Vec2, depth: u8) -> Self {
        GameEvent { on, pos, target: None, depth, source: Source::Main }
    }
}

#[derive(Clone, Debug)]
pub struct ActiveTrigger {
    pub def: super::Trigger,
    /// Seconds until it can fire again (for `Timer`, until it fires).
    pub cd: f32,
}

impl super::Trigger {
    pub fn initial_cd(&self) -> f32 {
        if self.on == On::Timer { self.cooldown } else { 0.0 }
    }

    /// Item triggers use item stats; the hero's own passive uses hero stats.
    pub fn owned_by(&self) -> Owner {
        if self.owner.is_some() { Owner::Item } else { Owner::Hero }
    }
}

/// Tick cooldowns, fire timers, then drain the event queue.
pub fn tick(w: &mut World, dt: f32) {
    for i in 0..w.triggers.len() {
        let t = &mut w.triggers[i];
        t.cd -= dt;
        if t.def.on == On::Timer && t.cd <= 0.0 {
            t.cd = t.def.cooldown.max(0.1);
            let (chance, action, owner) = (t.def.chance, t.def.action, t.def.owned_by());
            if chance >= 1.0 || w.rng.chance(chance) {
                let ev = GameEvent::at(On::Timer, w.player.pos, 0);
                run(w, &action, &ev, owner);
            }
        }
    }
    process(w);
}

pub fn process(w: &mut World) {
    let mut budget = EVENT_BUDGET;
    while let Some(ev) = w.events.pop_front() {
        if budget == 0 {
            w.events.clear();
            break;
        }
        budget -= 1;
        if ev.depth > MAX_DEPTH {
            continue;
        }
        for i in 0..w.triggers.len() {
            let t = &w.triggers[i];
            if t.def.on != ev.on || t.def.on == On::Timer || t.cd > 0.0 {
                continue;
            }
            if t.def.from.is_some_and(|f| f != ev.source) {
                continue;
            }
            let (chance, cooldown, action, owner) =
                (t.def.chance, t.def.cooldown, t.def.action, t.def.owned_by());
            if chance < 1.0 && !w.rng.chance(chance) {
                continue;
            }
            w.triggers[i].cd = cooldown;
            run(w, &action, &ev, owner);
        }
    }
}

/// Carry out one action for an event with `owner`'s stats. Damage it deals
/// is one generation deeper.
pub fn run(w: &mut World, action: &Action, ev: &GameEvent, owner: Owner) {
    let stats = *w.stats_of(owner);
    let damage = stats.get(Stat::Damage);
    let area = stats.get(Stat::Area);
    let duration = stats.get(Stat::Duration);
    let depth = ev.depth + 1;
    match *action {
        Action::Burn { dps, secs } => {
            if let Some(e) = ev.target.and_then(|i| w.enemies.get_mut(i)).filter(|e| !e.dead) {
                e.burn_dps = (e.burn_dps + dps * damage).min(damage * BURN_CAP);
                e.burn_time = e.burn_time.max(secs * duration);
            }
        }
        Action::Slow { amount, secs } => {
            if let Some(e) = ev.target.and_then(|i| w.enemies.get_mut(i)).filter(|e| !e.dead) {
                e.slow = e.slow.max(amount.clamp(0.0, 0.85));
                e.slow_time = e.slow_time.max(secs * duration);
            }
        }
        Action::Explode { radius, damage: ratio } => {
            weapons::blast(w, ev.pos, radius * area, damage * ratio, Source::Explode, depth, None);
            // The hero's own blasts (headbutts, hurt blasts) flash at its head.
            if ev.target.is_none() && ev.pos.dist_sq(w.player.pos) < 16.0 * 16.0 {
                w.fx.impact();
            }
        }
        Action::Nova { count, damage: ratio } => {
            let spark = bank().id("spark").expect("sprite `spark`");
            let tint = bank().get(spark).tint();
            let base = w.rng.angle();
            for k in 0..count {
                let dir = Vec2::from_angle(base + k as f32 / count.max(1) as f32 * std::f32::consts::TAU);
                player::spawn_shot(w, owner, ev.pos, dir, ratio, depth, spark);
                tag_last(w, Source::Nova);
                w.fx.streak(ev.pos, dir, tint);
            }
        }
        Action::Chain { jumps, range, damage: ratio } => {
            let mut from = ev.pos;
            let mut hit: Vec<u32> = ev.target.map(|i| w.enemies[i].uid).into_iter().collect();
            for _ in 0..jumps {
                let Some(i) = w.nearest_enemy(from, range * area, |e| !hit.contains(&e.uid)) else { break };
                let to = w.enemies[i].pos;
                hit.push(w.enemies[i].uid);
                w.fx.bolt(from, to);
                weapons::hit(w, i, damage * ratio, Vec2::ZERO, Source::Chain, depth, None);
                from = to;
            }
        }
        Action::Volley { count, damage: ratio } => {
            let pos = w.player.pos;
            let shot = crate::content::get().characters[w.character].weapon.shot_id;
            let range = stats.get(Stat::Range);
            let mut taken: Vec<u32> = Vec::new();
            for _ in 0..count {
                let target = w.nearest_enemy(pos, range, |e| !taken.contains(&e.uid));
                let dir = match target {
                    Some(i) => {
                        taken.push(w.enemies[i].uid);
                        (w.enemies[i].pos - pos).norm()
                    }
                    None => Vec2::from_angle(w.rng.angle()),
                };
                player::spawn_shot(w, owner, pos, dir, ratio, depth, shot);
                tag_last(w, Source::Volley);
            }
        }
        Action::Shockwave { radius, force } => {
            let r = radius * area;
            let enemies = &crate::content::get().enemies;
            for i in w.enemies_in(ev.pos, r) {
                let e = &mut w.enemies[i];
                // Heavy enemies and bosses resist shoves and pulls.
                e.push += (e.pos - ev.pos).norm() * force / enemies[e.kind].mass.max(1.0);
            }
            let color = if force < 0.0 { palette::GRAPE } else { palette::ICE };
            w.fx.ring(ev.pos, r, color);
            if force > 0.0 {
                w.fx.dust_ring(ev.pos + Vec2::new(0.0, 6.0), r);
            }
        }
        Action::Heal { amount } => w.heal(amount),
        Action::Strike { count, radius, damage: ratio } => {
            let rock = bank().id("meteor").expect("sprite `meteor`");
            weapons::rain(w, count, radius * area, damage * ratio, None, depth, rock);
        }
        Action::Chill { radius, amount, secs } => {
            let r = radius * area;
            for i in w.enemies_in(ev.pos, r) {
                let e = &mut w.enemies[i];
                e.slow = e.slow.max(amount.clamp(0.0, 0.85));
                e.slow_time = e.slow_time.max(secs * duration);
                e.hit_flash();
            }
            w.fx.ring(ev.pos, r, palette::CYAN);
            w.fx.burst(ev.pos, &[palette::ICE, palette::CYAN, palette::BONE], 30, r * 2.5);
        }
        Action::Shield { secs } => {
            let t = secs * duration;
            w.player.invuln = w.player.invuln.max(t);
            w.gear.shield = w.gear.shield.max(t);
            w.fx.ring(w.player.pos, 16.0, palette::SKY);
        }
        Action::Buff { stat, add, mul, secs } => {
            weapons::buff(w, StatMod { stat, add, mul }, secs * duration);
            let pos = w.player.pos;
            w.fx.burst(pos, &[palette::GOLD, palette::CREAM], 16, 90.0);
        }
        Action::Mines { count, radius, damage: ratio } => {
            let sprite = bank().id("mine").expect("sprite `mine`");
            for k in 0..count {
                let a = k as f32 / count.max(1) as f32 * std::f32::consts::TAU;
                let pos = w.arena.clamp(ev.pos + Vec2::from_angle(a) * 20.0, 4.0);
                let life = 8.0 * duration;
                let mine = Mine {
                    pos,
                    arm: 0.3,
                    life,
                    radius: radius * area,
                    damage: damage * ratio,
                    weapon: None,
                    depth,
                    sprite,
                };
                weapons::place_mine(w, mine);
            }
        }
    }
}

fn tag_last(w: &mut World, source: Source) {
    if let Some(s) = w.shots.last_mut() {
        s.tag.source = source;
    }
}
