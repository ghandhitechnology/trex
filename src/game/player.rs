//! Player movement, dash, auto-fire, and taking damage.

use std::f32::consts::TAU;

use super::Controls;
use super::world::{Shot, World};
use crate::content;
use crate::engine::{Vec2, damp};
use crate::items::effects::GameEvent;
use crate::items::{On, Owner, Stat};
use crate::meta::characters::Pattern;
use crate::render::palette;
use crate::render::sprite::SpriteId;

pub const RADIUS: f32 = 5.0;
pub const DASH_TIME: f32 = 0.14;
/// Dash velocity as a multiple of move speed: about 38 px at base speed.
const DASH_SPEED: f32 = 4.0;
/// Invulnerable this long after a dash ends.
const DASH_GRACE: f32 = 0.12;
const HURT_INVULN: f32 = 1.0;
/// A dash pressed this long before the cooldown ends still goes off.
const DASH_BUFFER: f32 = 0.15;
/// Seconds the attack pose holds after each shot.
pub const ATTACK_POSE: f32 = 0.2;
/// Longest lead, in seconds of enemy travel, when aiming ahead of a target.
const MAX_LEAD: f32 = 0.5;
/// Velocity smoothing rates toward the held direction and toward a stop.
const ACCEL: f32 = 20.0;
const DECEL: f32 = 26.0;

pub fn update(w: &mut World, c: &Controls, dt: f32) {
    let speed = w.stats.get(Stat::Speed);
    let input = Vec2::new(c.move_x, c.move_y).clamp_len(1.0);
    let p = &mut w.player;
    p.invuln = (p.invuln - dt).max(0.0);
    p.hurt = (p.hurt - dt).max(0.0);
    p.dash_cd = (p.dash_cd - dt).max(0.0);
    p.attack = (p.attack - dt).max(0.0);
    p.dash_buffer = if c.dash { DASH_BUFFER } else { (p.dash_buffer - dt).max(0.0) };

    if p.dash_buffer > 0.0 && p.dash_cd <= 0.0 && p.dash_time <= 0.0 {
        p.dash_buffer = 0.0;
        let dir = if input.len_sq() > 0.01 {
            input.norm()
        } else if p.vel.len_sq() > 1.0 {
            p.vel.norm()
        } else {
            Vec2::new(p.facing, 0.0)
        };
        p.dash_dir = dir;
        p.dash_time = DASH_TIME;
        p.dash_cd = w.stats.get(Stat::DashCooldown);
        p.invuln = p.invuln.max(DASH_TIME + DASH_GRACE);
        let pos = p.pos;
        w.events.push_back(GameEvent::at(On::Dash, pos, 0));
        w.fx.burst(pos, &[palette::FOG, palette::HAZE], 6, 50.0);
    }

    let p = &mut w.player;
    if p.dash_time > 0.0 {
        p.dash_time -= dt;
        p.vel = p.dash_dir * speed * DASH_SPEED;
        let (pos, flip) = (p.pos, p.facing < 0.0);
        let sprite = content::get().characters[w.character].sprite_id;
        let frame = (w.player.anim * 8.0) as usize;
        w.fx.ghost(pos, sprite, frame, flip);
    } else {
        let rate = if input.len_sq() > 0.0 { ACCEL } else { DECEL };
        p.vel = p.vel.lerp(input * speed, damp(rate, dt));
    }

    let p = &mut w.player;
    p.pos = w.arena.clamp(p.pos + p.vel * dt, RADIUS + 1.0);
    // Mid-attack the hero keeps facing its target; otherwise it faces where it runs.
    if p.attack > 0.0 && p.aim.x.abs() > 0.01 {
        p.facing = p.aim.x.signum();
    } else if p.vel.x.abs() > 4.0 {
        p.facing = p.vel.x.signum();
    }
    p.anim += dt * (p.vel.len() / speed.max(1.0)).min(2.0);

    let regen = w.stats.get(Stat::Regen);
    if regen > 0.0 {
        w.player.regen += regen / 60.0 * dt;
        if w.player.regen >= 1.0 {
            w.player.regen -= 1.0;
            w.heal(1);
        }
    }
}

/// Auto-fire the character's weapon.
pub fn fire(w: &mut World, dt: f32) {
    let ch = &content::get().characters[w.character];
    let s = w.stats;
    w.player.fire_cd -= dt;
    if w.player.fire_cd > 0.0 {
        return;
    }
    let shots = s.count(Stat::Shots).max(1);
    let pos = w.player.pos;
    match ch.weapon.pattern {
        Pattern::Aimed => {
            let Some(t) = w.nearest_enemy(pos, s.get(Stat::Range), |_| true) else {
                w.player.fire_cd = 0.0;
                return;
            };
            let aim = lead(w, t, pos, s.get(Stat::ShotSpeed));
            let facing = if aim.x < 0.0 { -1.0 } else { 1.0 };
            let from = muzzle(w, pos, facing);
            let spread = s.get(Stat::Spread).to_radians();
            for k in 0..shots {
                let off = (k as f32 - (shots - 1) as f32 / 2.0) * spread;
                spawn_shot(w, Owner::Hero, from, aim.rotate(off), 1.0, 0, ch.weapon.shot_id);
            }
            let p = &mut w.player;
            (p.facing, p.aim, p.attack) = (facing, aim, ATTACK_POSE);
            w.fx.spark(from + aim * 2.0, palette::CREAM, 40.0);
        }
        Pattern::Radial => {
            let base = w.time * 0.9;
            for k in 0..shots {
                let a = base + k as f32 / shots as f32 * TAU;
                spawn_shot(w, Owner::Hero, pos, Vec2::from_angle(a), 1.0, 0, ch.weapon.shot_id);
            }
            w.player.attack = ATTACK_POSE;
        }
    }
    w.player.fire_cd += 1.0 / s.get(Stat::FireRate);
}

/// Direction from `from` to where enemy `t` will be when a shot at `speed`
/// arrives, so shots meet moving targets instead of trailing them.
pub fn lead(w: &World, t: usize, from: Vec2, speed: f32) -> Vec2 {
    let e = &w.enemies[t];
    let time = (e.pos.dist_sq(from).sqrt() / speed.max(1.0)).min(MAX_LEAD);
    let aim = e.pos + e.vel * time - from;
    if aim.len_sq() > 1.0 { aim.norm() } else { (e.pos - from).norm() }
}

/// Where the hero's shots leave its sprite when facing `facing`.
pub fn muzzle(w: &World, pos: Vec2, facing: f32) -> Vec2 {
    let (x, y) = content::get().characters[w.character].muzzle;
    pos + Vec2::new(x * facing, y)
}

/// Spawn a player projectile using `owner`'s projectile stats.
/// `ratio` scales the Damage stat.
pub fn spawn_shot(
    w: &mut World,
    owner: Owner,
    pos: Vec2,
    dir: Vec2,
    ratio: f32,
    depth: u8,
    sprite: SpriteId,
) {
    let s = w.stats_of(owner);
    let speed = s.get(Stat::ShotSpeed);
    w.shots.push(Shot {
        pos,
        vel: dir * speed,
        damage: s.get(Stat::Damage) * ratio,
        radius: 2.5 * s.get(Stat::ShotSize),
        pierce: s.count(Stat::Pierce),
        bounce: s.count(Stat::Bounce),
        life: s.get(Stat::Range) / speed.max(1.0) * 1.15,
        homing: s.get(Stat::Homing),
        knock: s.get(Stat::Knockback),
        depth,
        sprite,
        hostile: false,
        hostile_damage: 0,
        hits: [0; 8],
        nhits: 0,
        age: 0.0,
        dead: false,
        tag: Default::default(),
    });
}

/// Damage the player unless invulnerable. `from` is the source position.
pub fn hurt(w: &mut World, dmg: i32, from: Vec2) {
    if w.player.invuln > 0.0 || w.player.hp <= 0 || dmg <= 0 {
        return;
    }
    let dmg = dmg + content::get().waves.overtime_damage(w.director.clock(w));
    let pos = w.player.pos;
    let dodge = w.stats.get(Stat::Dodge);
    if dodge > 0.0 && w.rng.chance(dodge) {
        w.player.invuln = 0.4;
        w.fx.text(pos + Vec2::new(0.0, -10.0), "DODGE".into(), palette::CYAN);
        return;
    }
    let p = &mut w.player;
    p.hp -= dmg;
    p.invuln = HURT_INVULN;
    p.hurt = 0.25;
    p.vel += (pos - from).norm() * 120.0;
    w.fx.shake(0.55);
    w.fx.flash = 0.08;
    w.fx.freeze(0.07);
    w.fx.burst(pos, &[palette::RED, palette::BLOOD, palette::BONE], 14, 90.0);
    w.events.push_back(GameEvent::at(On::Hurt, pos, 0));
}
