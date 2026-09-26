//! Death animations. Visual only: uses the effects RNG, never the game RNG.

use serde::Deserialize;

use crate::content;
use crate::engine::Vec2;
use crate::game::world::World;
use crate::render::fx::{Fx, Particle};
use crate::render::palette::{self, Color};
use crate::render::sprite::bank;

#[derive(Deserialize, Debug, Default, Clone, Copy)]
pub enum Death {
    /// Chunks of the body arc and fall.
    #[default]
    Pop,
    /// Goo sprays low and wide.
    Splat,
    /// Embers drift upward.
    Ash,
    /// Fast shards and a cold ring.
    Shatter,
    /// Slow rising smoke puffs.
    Smoke,
    /// Sparks and little lightning forks.
    Zap,
    /// Fireball ring.
    Boom,
    /// Afterimage that fades upward.
    Fade,
    /// A cloud of drifting spores.
    Spores,
}

const MAX_PARTICLES: usize = 1500;

pub fn play(w: &mut World, pos: Vec2, kind: usize, boss: bool) {
    if !w.fx.enabled {
        return;
    }
    let def = &content::get().enemies[kind];
    let sprite = bank().get(def.sprite_id);
    let colors: Vec<Color> = sprite.colors.iter().take(4).map(|&c| palette::color(c)).collect();
    let fx = &mut w.fx;
    if boss {
        fx.shake(1.0);
        fx.flash = 0.08;
        for (k, c) in [palette::CREAM, palette::GOLD, palette::EMBER, palette::RED].into_iter().enumerate() {
            fx.ring(pos, 24.0 + k as f32 * 16.0, c);
        }
        fx.debris(pos, &colors, 60);
        fx.burst(pos, &[palette::BONE, palette::CREAM, palette::GOLD], 50, 180.0);
        drift(fx, pos, &colors, 30, -40.0, 1.4);
        return;
    }
    match def.death {
        Death::Pop => {
            fx.debris(pos, &colors, 10);
            fx.burst(pos, &[palette::BONE, palette::CREAM], 5, 60.0);
        }
        Death::Splat => {
            for _ in 0..16 {
                let a = fx.rng.range(-3.1, 0.0);
                let v = fx.rng.range(20.0, 70.0);
                let c = colors[fx.rng.below(colors.len())];
                let life = fx.rng.range(0.4, 0.8);
                push(fx, pos, Vec2::new(a.cos() * v * 1.6, a.sin() * v * 0.6), c, 2, life, 3.0, 150.0);
            }
            fx.ring(pos, 10.0, colors[0]);
        }
        Death::Ash => {
            drift(fx, pos, &[palette::EMBER, palette::AMBER, palette::GOLD, palette::NIGHT], 14, -35.0, 0.9);
            fx.burst(pos, &colors, 6, 40.0);
        }
        Death::Shatter => {
            fx.burst(pos, &[palette::ICE, palette::BONE, palette::CYAN], 14, 130.0);
            fx.debris(pos, &colors, 8);
            fx.ring(pos, 12.0, palette::ICE);
        }
        Death::Smoke => {
            drift(fx, pos, &[palette::HAZE, palette::MAUVE, palette::FOG], 12, -18.0, 1.2);
            fx.debris(pos, &colors, 6);
        }
        Death::Zap => {
            for _ in 0..3 {
                let to = pos + Vec2::from_angle(fx.rng.angle()) * fx.rng.range(8.0, 16.0);
                fx.bolt(pos, to);
            }
            fx.burst(pos, &[palette::ICE, palette::CYAN, palette::SKY], 10, 90.0);
        }
        Death::Boom => {
            fx.ring(pos, 16.0, palette::EMBER);
            fx.burst(pos, &[palette::GOLD, palette::AMBER, palette::EMBER, palette::RED], 18, 110.0);
            drift(fx, pos, &[palette::MAUVE, palette::SLATE], 6, -20.0, 0.9);
            fx.shake(0.12);
        }
        Death::Fade => {
            let frame = sprite.frames.len().saturating_sub(1);
            fx.ghost(pos, def.sprite_id, frame, false);
            drift(fx, pos, &colors, 10, -30.0, 0.8);
        }
        Death::Spores => {
            drift(fx, pos, &[palette::PINK, palette::BLUSH, palette::GRAPE, palette::SPROUT], 18, -8.0, 1.3);
            fx.ring(pos, 9.0, palette::PINK);
        }
    }
}

/// Particles that float up (negative `lift`) or sink, spreading slowly.
fn drift(fx: &mut Fx, pos: Vec2, colors: &[Color], n: usize, lift: f32, life: f32) {
    for _ in 0..n {
        let c = colors[fx.rng.below(colors.len())];
        let off = Vec2::new(fx.rng.range(-5.0, 5.0), fx.rng.range(-4.0, 3.0));
        let vel = Vec2::new(fx.rng.range(-14.0, 14.0), lift * fx.rng.range(0.5, 1.2));
        let life = life * fx.rng.range(0.6, 1.0);
        let size = if fx.rng.chance(0.35) { 2 } else { 1 };
        push(fx, pos + off, vel, c, size, life, 1.5, 0.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn push(fx: &mut Fx, pos: Vec2, vel: Vec2, color: Color, size: u8, life: f32, drag: f32, grav: f32) {
    if fx.particles.len() < MAX_PARTICLES {
        fx.particles.push(Particle { pos, vel, life, max: life, color, size, drag, grav });
    }
}
