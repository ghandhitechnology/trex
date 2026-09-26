//! Visual-only effects: particles, damage numbers, rings, lightning, afterimages,
//! screen shake requests and hit stop.
//! Uses its own RNG so gameplay stays identical with or without visuals.

use super::camera::Camera;
use super::canvas::{Blit, Canvas};
use super::font;
use super::palette::{self, Color};
use super::sprite::{SpriteId, bank};
use crate::engine::{Rng, Vec2};

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max: f32,
    pub color: Color,
    /// Color it cools toward as it dies.
    pub fade: Color,
    pub size: u8,
    pub drag: f32,
    pub grav: f32,
    /// Drawn as a short line along its velocity.
    pub streak: bool,
}

pub struct Text {
    pub pos: Vec2,
    pub life: f32,
    pub text: String,
    pub color: Color,
    /// Running total for merged damage numbers.
    pub value: f32,
    pub crit: bool,
    /// Seconds since the last pop (spawn or merge).
    pub pop: f32,
}

pub struct Ring {
    pub pos: Vec2,
    pub radius: f32,
    pub life: f32,
    pub max: f32,
    pub color: Color,
}

pub struct Bolt {
    pub a: Vec2,
    pub b: Vec2,
    pub life: f32,
    pub seed: u64,
}

pub struct Ghost {
    pub pos: Vec2,
    pub sprite: SpriteId,
    pub frame: usize,
    pub flip: bool,
    pub life: f32,
}

pub struct Fx {
    /// When false (headless sim) nothing is spawned.
    pub enabled: bool,
    pub rng: Rng,
    pub particles: Vec<Particle>,
    pub texts: Vec<Text>,
    pub rings: Vec<Ring>,
    pub bolts: Vec<Bolt>,
    pub ghosts: Vec<Ghost>,
    /// Screen shake requested this tick, consumed by the camera.
    pub kick: f32,
    /// Red hurt flash, seconds left.
    pub flash: f32,
    /// Bright white flash for big kills, seconds left.
    pub glare: f32,
    /// Hit stop: seconds the world stays frozen. Only set when enabled.
    pub hitstop: f32,
}

const MAX_PARTICLES: usize = 1500;
const MAX_TEXTS: usize = 40;
const TEXT_LIFE: f32 = 0.7;
const GHOST_LIFE: f32 = 0.22;
/// Damage numbers landing this close to a fresh one merge into it.
const MERGE_DIST: f32 = 10.0;
const MERGE_AGE: f32 = 0.18;

impl Fx {
    pub fn new(seed: u64, enabled: bool) -> Self {
        Fx {
            enabled,
            rng: Rng::new(seed ^ 0x5eed_f00d),
            particles: Vec::new(),
            texts: Vec::new(),
            rings: Vec::new(),
            bolts: Vec::new(),
            ghosts: Vec::new(),
            kick: 0.0,
            flash: 0.0,
            glare: 0.0,
            hitstop: 0.0,
        }
    }

    pub fn push(&mut self, p: Particle) {
        if self.enabled && self.particles.len() < MAX_PARTICLES {
            self.particles.push(p);
        }
    }

    /// Radial burst of `n` particles picked from `colors`.
    pub fn burst(&mut self, pos: Vec2, colors: &[Color], n: usize, speed: f32) {
        if !self.enabled || colors.is_empty() {
            return;
        }
        for _ in 0..n {
            let a = self.rng.angle();
            let v = self.rng.range(0.3, 1.0) * speed;
            let life = self.rng.range(0.25, 0.6);
            let color = colors[self.rng.below(colors.len())];
            let size = if self.rng.chance(0.3) { 2 } else { 1 };
            self.push(Particle {
                pos,
                vel: Vec2::from_angle(a) * v,
                life,
                max: life,
                color,
                fade: palette::MAUVE,
                size,
                drag: 5.0,
                grav: 0.0,
                streak: v > 70.0,
            });
        }
    }

    /// A single short-lived spark.
    pub fn spark(&mut self, pos: Vec2, color: Color, speed: f32) {
        if !self.enabled {
            return;
        }
        let a = self.rng.angle();
        let life = self.rng.range(0.12, 0.3);
        let v = speed * self.rng.range(0.5, 1.0);
        self.push(Particle {
            pos,
            vel: Vec2::from_angle(a) * v,
            life,
            max: life,
            color,
            fade: color.mix(palette::INK, 110),
            size: 1,
            drag: 6.0,
            grav: 0.0,
            streak: false,
        });
    }

    /// Debris that arcs and falls (death bursts).
    pub fn debris(&mut self, pos: Vec2, colors: &[Color], n: usize) {
        if !self.enabled || colors.is_empty() {
            return;
        }
        for _ in 0..n {
            let a = self.rng.range(-2.8, -0.35);
            let v = self.rng.range(30.0, 90.0);
            let life = self.rng.range(0.35, 0.7);
            let color = colors[self.rng.below(colors.len())];
            let size = if self.rng.chance(0.4) { 2 } else { 1 };
            self.push(Particle {
                pos,
                vel: Vec2::from_angle(a) * v,
                life,
                max: life,
                color,
                fade: color.mix(palette::INK, 120),
                size,
                drag: 2.0,
                grav: 260.0,
                streak: false,
            });
        }
    }

    /// The flash every enemy death shares: a quick bright ring, and for heavy
    /// enemies a bigger ring, a shake and a short hit stop.
    pub fn pop(&mut self, pos: Vec2, heavy: bool) {
        if !self.enabled {
            return;
        }
        self.rings.push(Ring {
            pos,
            radius: if heavy { 20.0 } else { 7.0 },
            life: 0.16,
            max: 0.16,
            color: palette::BONE,
        });
        if heavy {
            self.shake(0.3);
            self.freeze(0.05);
        }
    }

    /// Celebration when a level-up item is taken.
    pub fn level_up(&mut self, pos: Vec2) {
        if !self.enabled {
            return;
        }
        self.rings.push(Ring { pos, radius: 34.0, life: 0.4, max: 0.4, color: palette::GOLD });
        self.burst(pos, &[palette::GOLD, palette::CREAM, palette::AMBER], 22, 120.0);
        for k in 0..10 {
            let x = pos.x + (k as f32 - 4.5) * 3.0 + self.rng.range(-1.0, 1.0);
            let life = self.rng.range(0.5, 0.9);
            let rise = self.rng.range(40.0, 80.0);
            self.push(Particle {
                pos: Vec2::new(x, pos.y + 4.0),
                vel: Vec2::new(0.0, -rise),
                life,
                max: life,
                color: if k % 2 == 0 { palette::CREAM } else { palette::GOLD },
                fade: palette::AMBER,
                size: 1,
                drag: 1.5,
                grav: 0.0,
                streak: true,
            });
        }
    }

    pub fn text(&mut self, pos: Vec2, text: String, color: Color) {
        if !self.enabled {
            return;
        }
        if self.texts.len() >= MAX_TEXTS {
            self.texts.remove(0);
        }
        let jitter = Vec2::new(self.rng.range(-3.0, 3.0), self.rng.range(-2.0, 0.0));
        self.texts.push(Text {
            pos: pos + jitter,
            life: TEXT_LIFE,
            text,
            color,
            value: 0.0,
            crit: false,
            pop: 0.0,
        });
    }

    /// Damage number. Hits landing on a fresh number nearby add to it.
    pub fn number(&mut self, pos: Vec2, value: f32, color: Color) {
        self.damage(pos, value, color == palette::GOLD);
    }

    pub fn damage(&mut self, pos: Vec2, value: f32, crit: bool) {
        if !self.enabled {
            return;
        }
        let near = self.texts.iter_mut().rev().find(|t| {
            t.value > 0.0
                && t.crit == crit
                && TEXT_LIFE - t.life < MERGE_AGE
                && t.pos.dist_sq(pos) < MERGE_DIST.powi(2)
        });
        if let Some(t) = near {
            t.value += value;
            t.text = (t.value.round().max(1.0) as i64).to_string();
            t.life = TEXT_LIFE;
            t.pop = 0.0;
            return;
        }
        let color = if crit { palette::GOLD } else { palette::BONE };
        self.text(pos, (value.round().max(1.0) as i64).to_string(), color);
        if let Some(t) = self.texts.last_mut() {
            t.value = value;
            t.crit = crit;
        }
    }

    pub fn ring(&mut self, pos: Vec2, radius: f32, color: Color) {
        if self.enabled {
            self.rings.push(Ring { pos, radius, life: 0.28, max: 0.28, color });
        }
    }

    pub fn bolt(&mut self, a: Vec2, b: Vec2) {
        if self.enabled {
            let seed = self.rng.next_u64();
            self.bolts.push(Bolt { a, b, life: 0.16, seed });
            self.spark(b, palette::ICE, 50.0);
        }
    }

    pub fn ghost(&mut self, pos: Vec2, sprite: SpriteId, frame: usize, flip: bool) {
        if self.enabled {
            self.ghosts.push(Ghost { pos, sprite, frame, flip, life: GHOST_LIFE });
        }
    }

    pub fn shake(&mut self, amount: f32) {
        self.kick += amount;
    }

    /// Freeze the world for a moment to sell a heavy hit.
    pub fn freeze(&mut self, secs: f32) {
        if self.enabled {
            self.hitstop = self.hitstop.max(secs);
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.vel *= (1.0 - p.drag * dt).max(0.0);
            p.vel.y += p.grav * dt;
            p.pos += p.vel * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
        for t in &mut self.texts {
            t.pos.y -= 20.0 * dt * (t.life / TEXT_LIFE);
            t.life -= dt;
            t.pop += dt;
        }
        self.texts.retain(|t| t.life > 0.0);
        for r in &mut self.rings {
            r.life -= dt;
        }
        self.rings.retain(|r| r.life > 0.0);
        for b in &mut self.bolts {
            b.life -= dt;
        }
        self.bolts.retain(|b| b.life > 0.0);
        for g in &mut self.ghosts {
            g.life -= dt;
        }
        self.ghosts.retain(|g| g.life > 0.0);
        self.flash = (self.flash - dt).max(0.0);
        self.glare = (self.glare - dt).max(0.0);
    }

    /// Afterimages, drawn under creatures.
    pub fn draw_under(&self, cv: &mut Canvas, cam: &Camera) {
        for g in &self.ghosts {
            let (x, y) = cam.to_screen(g.pos);
            let s = bank().get(g.sprite);
            let f = &s.frames[g.frame % s.frames.len()];
            let k = g.life / GHOST_LIFE;
            let c = if k > 0.5 { palette::CYAN } else { palette::SKY };
            cv.blit_centered(f, x, y, Blit { flip_x: g.flip, flash: Some(c), alpha: (k * 150.0) as u8 });
        }
    }

    /// Particles, rings, lightning and damage numbers, drawn over everything in the world.
    pub fn draw_over(&self, cv: &mut Canvas, cam: &Camera) {
        for r in &self.rings {
            let t = 1.0 - r.life / r.max;
            let (x, y) = cam.to_screen(r.pos);
            let ease = 1.0 - (1.0 - t) * (1.0 - t);
            let rad = (r.radius * (0.3 + 0.7 * ease)) as i32;
            if t < 0.2 {
                cv.fill_circle(x, y, rad * 2 / 3, palette::CREAM);
            } else if t < 0.4 {
                cv.blend_ellipse(x, y, rad, rad, r.color, 70);
            }
            cv.circle(x, y, rad, if t < 0.6 { r.color } else { r.color.mix(palette::INK, 100) });
            if t < 0.5 {
                cv.circle(x, y, rad - 1, palette::CREAM);
            }
        }
        for b in &self.bolts {
            let mut rng = Rng::new(b.seed);
            let (ax, ay) = cam.to_screen(b.a);
            let (bx, by) = cam.to_screen(b.b);
            let steps = 5;
            let mut prev = (ax, ay);
            let core = if b.life > 0.08 { palette::BONE } else { palette::ICE };
            for i in 1..=steps {
                let t = i as f32 / steps as f32;
                let mut x = ax as f32 + (bx - ax) as f32 * t;
                let mut y = ay as f32 + (by - ay) as f32 * t;
                if i < steps {
                    x += rng.range(-4.0, 4.0);
                    y += rng.range(-4.0, 4.0);
                }
                let next = (x as i32, y as i32);
                cv.line(prev.0, prev.1 + 1, next.0, next.1 + 1, palette::BLUE);
                cv.line(prev.0 + 1, prev.1, next.0 + 1, next.1, palette::CYAN);
                cv.line(prev.0, prev.1, next.0, next.1, core);
                prev = next;
            }
        }
        for p in &self.particles {
            let (x, y) = cam.to_screen(p.pos);
            let k = p.life / p.max;
            let c = if k > 0.55 {
                p.color
            } else if k > 0.25 {
                p.color.mix(p.fade, 128)
            } else {
                p.fade
            };
            if p.streak {
                let tail = p.vel * -0.025;
                cv.line(x, y, x + tail.x as i32, y + tail.y as i32, c);
            } else if p.size > 1 && k > 0.35 {
                cv.fill_rect(x, y, 2, 2, c);
            } else {
                cv.put(x, y, c);
            }
        }
        for t in &self.texts {
            let (x, y) = cam.to_screen(t.pos);
            // Pop up on spawn and on every merge, then cool down before vanishing.
            let lift = if t.pop < 0.05 {
                2
            } else if t.pop < 0.1 {
                1
            } else {
                0
            };
            let (fill, edge) = match (t.crit, t.life / TEXT_LIFE) {
                (true, k) if k > 0.3 => {
                    (if t.pop < 0.06 { palette::CREAM } else { t.color }, palette::MAROON)
                }
                (false, k) if k > 0.3 => (if t.pop < 0.06 { palette::CREAM } else { t.color }, palette::INK),
                (_, _) => (t.color.mix(palette::MAUVE, 150), palette::INK),
            };
            let w = font::width(&t.text);
            let y = y - 3 - lift;
            if t.crit {
                font::draw_big(cv, x - w / 2, y, &t.text, fill, edge, 1);
            } else {
                font::draw_outlined(cv, x - w / 2, y, &t.text, fill, edge);
            }
        }
    }
}
