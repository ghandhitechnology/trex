//! Runs item and passive triggers. Events queue up during a tick and are
//! processed together; actions can cause new events one generation deeper,
//! capped by `MAX_DEPTH` so chain reactions stay bounded.

use super::{Action, On, Stat, Trigger};
use crate::engine::Vec2;
use crate::game::player;
use crate::game::world::{Hit, World};
use crate::render::palette;
use crate::render::sprite::bank;

/// Events from depth 0 (weapon) and depth 1 (first proc) can fire triggers.
pub const MAX_DEPTH: u8 = 1;
const EVENT_BUDGET: usize = 3000;

#[derive(Clone, Copy, Debug)]
pub struct GameEvent {
    pub on: On,
    pub pos: Vec2,
    /// Enemy index for Hit/Crit/Kill.
    pub target: Option<usize>,
    pub depth: u8,
}

impl GameEvent {
    pub fn at(on: On, pos: Vec2, depth: u8) -> Self {
        GameEvent { on, pos, target: None, depth }
    }
}

#[derive(Clone, Debug)]
pub struct ActiveTrigger {
    pub def: Trigger,
    /// Seconds until it can fire again (for `Timer`, until it fires).
    pub cd: f32,
}

impl Trigger {
    pub fn initial_cd(&self) -> f32 {
        if self.on == On::Timer { self.cooldown } else { 0.0 }
    }
}

/// Tick cooldowns, fire timers, then drain the event queue.
pub fn tick(w: &mut World, dt: f32) {
    for i in 0..w.triggers.len() {
        let t = &mut w.triggers[i];
        t.cd -= dt;
        if t.def.on == On::Timer && t.cd <= 0.0 {
            t.cd = t.def.cooldown.max(0.1);
            let (chance, action) = (t.def.chance, t.def.action.clone());
            if chance >= 1.0 || w.rng.chance(chance) {
                let ev = GameEvent::at(On::Timer, w.player.pos, 0);
                run(w, &action, &ev);
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
            let (chance, cooldown, action) = (t.def.chance, t.def.cooldown, t.def.action.clone());
            if chance < 1.0 && !w.rng.chance(chance) {
                continue;
            }
            w.triggers[i].cd = cooldown;
            run(w, &action, &ev);
        }
    }
}

fn run(w: &mut World, action: &Action, ev: &GameEvent) {
    let damage = w.stats.get(Stat::Damage);
    let area = w.stats.get(Stat::Area);
    let duration = w.stats.get(Stat::Duration);
    let depth = ev.depth + 1;
    match *action {
        Action::Burn { dps, secs } => {
            if let Some(e) = ev.target.and_then(|i| w.enemies.get_mut(i)).filter(|e| !e.dead) {
                e.burn_dps = (e.burn_dps + dps * damage).min(damage * 8.0);
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
            let r = radius * area;
            for i in w.enemies_in(ev.pos, r) {
                let knock = (w.enemies[i].pos - ev.pos).norm() * 90.0;
                w.damage_enemy(i, Hit { damage: damage * ratio, knock, crit: false, depth, procs: true });
            }
            w.fx.ring(ev.pos, r, palette::EMBER);
            w.fx.burst(ev.pos, &[palette::GOLD, palette::AMBER, palette::EMBER, palette::RED], 18, r * 3.0);
            w.fx.shake(0.12);
        }
        Action::Nova { count, damage: ratio } => {
            let spark = bank().id("spark").expect("sprite `spark`");
            let base = w.rng.angle();
            for k in 0..count {
                let a = base + k as f32 / count.max(1) as f32 * std::f32::consts::TAU;
                player::spawn_shot(w, ev.pos, Vec2::from_angle(a), ratio, depth, spark);
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
                w.damage_enemy(
                    i,
                    Hit { damage: damage * ratio, knock: Vec2::ZERO, crit: false, depth, procs: true },
                );
                from = to;
            }
        }
        Action::Volley { count, damage: ratio } => {
            let pos = w.player.pos;
            let shot = crate::content::get().characters[w.character].weapon.shot_id;
            let range = w.stats.get(Stat::Range);
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
                player::spawn_shot(w, pos, dir, ratio, depth, shot);
            }
        }
        Action::Shockwave { radius, force } => {
            let r = radius * area;
            for i in w.enemies_in(ev.pos, r) {
                let e = &mut w.enemies[i];
                e.push += (e.pos - ev.pos).norm() * force;
            }
            w.fx.ring(ev.pos, r, palette::ICE);
        }
        Action::Heal { amount } => w.heal(amount),
    }
}
