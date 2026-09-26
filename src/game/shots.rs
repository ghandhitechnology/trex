//! Projectile movement, homing, and collisions.

use std::f32::consts::{PI, TAU};

use super::player;
use super::world::{Hit, World};
use crate::content;
use crate::items::Stat;
use crate::render::palette;

const HOMING_RANGE: f32 = 70.0;
const BOUNCE_RANGE: f32 = 110.0;

pub fn update(w: &mut World, dt: f32) {
    let enemies = &content::get().enemies;
    let mut near = Vec::new();
    for si in 0..w.shots.len() {
        let mut s = w.shots[si];
        s.age += dt;
        s.life -= dt;
        if s.life <= 0.0 {
            s.dead = true;
            w.fx.spark(s.pos, palette::AMBER, 20.0);
            w.shots[si] = s;
            continue;
        }

        if s.homing > 0.0
            && !s.hostile
            && let Some(t) = w.nearest_enemy(s.pos, HOMING_RANGE, |e| !s.has_hit(e.uid))
        {
            let diff = ((w.enemies[t].pos - s.pos).angle() - s.vel.angle() + PI).rem_euclid(TAU) - PI;
            s.vel = s.vel.rotate(diff.clamp(-s.homing * dt, s.homing * dt));
        }

        s.pos += s.vel * dt;
        if !w.arena.contains(s.pos) {
            s.dead = true;
            w.shots[si] = s;
            continue;
        }

        if s.hostile {
            if s.pos.dist_sq(w.player.pos) < (s.radius + player::RADIUS).powi(2) && w.player.invuln <= 0.0 {
                s.dead = true;
                let from = s.pos - s.vel;
                player::hurt(w, s.hostile_damage, from);
            }
            w.shots[si] = s;
            continue;
        }

        near.clear();
        w.grid.query(s.pos, s.radius + 12.0, |i| near.push(i));
        for &i in &near {
            let e = &w.enemies[i];
            if !e.hittable() || s.has_hit(e.uid) {
                continue;
            }
            let r = s.radius + enemies[e.kind].radius;
            if e.pos.dist_sq(s.pos) >= r * r {
                continue;
            }
            let uid = e.uid;
            let crit = w.rng.chance(w.stats.get(Stat::Crit));
            let damage = if crit { s.damage * w.stats.get(Stat::CritDamage) } else { s.damage };
            let knock = s.vel.norm() * s.knock;
            w.damage_enemy(i, Hit { damage, knock, crit, depth: s.depth, procs: true });
            w.fx.burst(s.pos, &[palette::CREAM, palette::GOLD], 3, 50.0);
            s.mark(uid);
            if s.pierce > 0 {
                s.pierce -= 1;
            } else if s.bounce > 0 {
                s.bounce -= 1;
                let speed = s.vel.len();
                let next = w.nearest_enemy(s.pos, BOUNCE_RANGE, |e| !s.has_hit(e.uid));
                match next {
                    Some(t) => s.vel = (w.enemies[t].pos - s.pos).norm() * speed,
                    None => s.dead = true,
                }
                s.life = s.life.max(0.4);
            } else {
                s.dead = true;
            }
            if s.dead {
                break;
            }
        }
        w.shots[si] = s;
    }
    w.shots.retain(|s| !s.dead);
}
