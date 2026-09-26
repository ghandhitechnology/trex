//! Player movement, dash, auto-fire, and taking damage.

use std::f32::consts::TAU;

use super::Controls;
use super::world::{Shot, World};
use crate::content;
use crate::engine::{Vec2, damp};
use crate::items::effects::GameEvent;
use crate::items::{On, Stat};
use crate::meta::characters::Pattern;
use crate::render::palette;
use crate::render::sprite::SpriteId;

pub const RADIUS: f32 = 5.0;
const DASH_TIME: f32 = 0.16;
const DASH_SPEED: f32 = 3.4;
const HURT_INVULN: f32 = 1.0;

pub fn update(w: &mut World, c: &Controls, dt: f32) {
    let speed = w.stats.get(Stat::Speed);
    let input = Vec2::new(c.move_x, c.move_y).clamp_len(1.0);
    let p = &mut w.player;
    p.invuln = (p.invuln - dt).max(0.0);
    p.hurt = (p.hurt - dt).max(0.0);
    p.dash_cd = (p.dash_cd - dt).max(0.0);

    if c.dash && p.dash_cd <= 0.0 && p.dash_time <= 0.0 {
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
        p.invuln = p.invuln.max(DASH_TIME + 0.1);
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
        let rate = if input.len_sq() > 0.0 { 14.0 } else { 18.0 };
        p.vel = p.vel.lerp(input * speed, damp(rate, dt));
    }

    let p = &mut w.player;
    p.pos = w.arena.clamp(p.pos + p.vel * dt, RADIUS + 1.0);
    if p.vel.x.abs() > 4.0 {
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
            let aim = (w.enemies[t].pos - pos).norm();
            let spread = s.get(Stat::Spread).to_radians();
            for k in 0..shots {
                let off = (k as f32 - (shots - 1) as f32 / 2.0) * spread;
                spawn_shot(w, pos, aim.rotate(off), 1.0, 0, ch.weapon.shot_id);
            }
            w.player.facing = if aim.x < 0.0 { -1.0 } else { 1.0 };
            w.fx.spark(pos + aim * 6.0, palette::CREAM, 40.0);
        }
        Pattern::Radial => {
            let base = w.time * 0.9;
            for k in 0..shots {
                let a = base + k as f32 / shots as f32 * TAU;
                spawn_shot(w, pos, Vec2::from_angle(a), 1.0, 0, ch.weapon.shot_id);
            }
        }
    }
    w.player.fire_cd += 1.0 / s.get(Stat::FireRate);
}

/// Spawn a player projectile using the current projectile stats.
/// `ratio` scales the Damage stat.
pub fn spawn_shot(w: &mut World, pos: Vec2, dir: Vec2, ratio: f32, depth: u8, sprite: SpriteId) {
    let s = &w.stats;
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
    });
}

/// Damage the player unless invulnerable. `from` is the source position.
pub fn hurt(w: &mut World, dmg: i32, from: Vec2) {
    if w.player.invuln > 0.0 || w.player.hp <= 0 || dmg <= 0 {
        return;
    }
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
