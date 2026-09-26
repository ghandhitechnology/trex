//! XP gems: pop out, get pulled in by the magnet radius, level the player up.

use super::world::{World, xp_for_level};
use crate::engine::{Vec2, damp};
use crate::items::Stat;
use crate::render::palette;

const MAX_GEMS: usize = 400;
const COLLECT_RADIUS: f32 = 6.0;

pub fn update(w: &mut World, dt: f32) {
    let pickup = w.stats.get(Stat::Pickup);
    let xp_mul = w.stats.get(Stat::XpGain);
    let target = w.player.pos;
    let mut gained = 0.0;
    for g in &mut w.gems {
        g.age += dt;
        let to = target - g.pos;
        let d = to.len();
        if !g.pull && d < pickup {
            g.pull = true;
            g.vel = -to.norm() * 40.0;
        }
        if g.pull {
            let speed = 90.0 + g.age.min(3.0) * 40.0;
            g.vel = g.vel.lerp(to.norm() * speed.max(d * 4.0), damp(7.0, dt));
        } else {
            g.vel *= (1.0 - 6.0 * dt).max(0.0);
        }
        g.pos += g.vel * dt;
        if (target - g.pos).len() < COLLECT_RADIUS {
            g.dead = true;
            gained += g.value as f32 * xp_mul;
        }
    }
    let collected = w.gems.iter().filter(|g| g.dead).count();
    w.gems.retain(|g| !g.dead);
    for _ in 0..collected.min(3) {
        w.fx.spark(target + Vec2::new(0.0, -2.0), palette::CYAN, 45.0);
    }
    if collected > 0 {
        w.fx.xp_pulse();
    }

    // Keep the count bounded by folding the oldest gem into the next one.
    while w.gems.len() > MAX_GEMS {
        let old = w.gems.remove(0);
        w.gems[0].value += old.value;
    }

    let p = &mut w.player;
    p.xp += gained;
    while p.xp >= p.xp_next {
        p.xp -= p.xp_next;
        p.level += 1;
        p.xp_next = xp_for_level(p.level) as f32;
        w.pending_levels += 1;
    }
}
