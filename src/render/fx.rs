//! Visual-only effects: particles, damage numbers, rings, lightning, afterimages.
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
    pub size: u8,
    pub drag: f32,
    pub grav: f32,
}

pub struct Text {
    pub pos: Vec2,
    pub life: f32,
    pub text: String,
    pub color: Color,
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
    /// Full-screen flash, seconds left.
    pub flash: f32,
}

const MAX_PARTICLES: usize = 1500;
const MAX_TEXTS: usize = 48;

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
        }
    }

    fn push(&mut self, p: Particle) {
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
                size,
                drag: 5.0,
                grav: 0.0,
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
            size: 1,
            drag: 6.0,
            grav: 0.0,
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
                size,
                drag: 2.0,
                grav: 260.0,
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
        self.texts.push(Text { pos: pos + jitter, life: 0.6, text, color });
    }

    pub fn number(&mut self, pos: Vec2, value: f32, color: Color) {
        let v = value.round().max(1.0) as i64;
        self.text(pos, v.to_string(), color);
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
        }
    }

    pub fn ghost(&mut self, pos: Vec2, sprite: SpriteId, frame: usize, flip: bool) {
        if self.enabled {
            self.ghosts.push(Ghost { pos, sprite, frame, flip, life: 0.22 });
        }
    }

    pub fn shake(&mut self, amount: f32) {
        self.kick += amount;
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
            t.pos.y -= 22.0 * dt * (t.life / 0.6);
            t.life -= dt;
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
    }

    /// Afterimages, drawn under creatures.
    pub fn draw_under(&self, cv: &mut Canvas, cam: &Camera) {
        for g in &self.ghosts {
            let (x, y) = cam.to_screen(g.pos);
            let s = bank().get(g.sprite);
            let f = &s.frames[g.frame % s.frames.len()];
            let a = (g.life / 0.22 * 150.0) as u8;
            cv.blit_centered(f, x, y, Blit { flip_x: g.flip, flash: Some(palette::CYAN), alpha: a });
        }
    }

    /// Particles, rings, lightning and damage numbers, drawn over everything in the world.
    pub fn draw_over(&self, cv: &mut Canvas, cam: &Camera) {
        for r in &self.rings {
            let t = 1.0 - r.life / r.max;
            let (x, y) = cam.to_screen(r.pos);
            let rad = (r.radius * (0.35 + 0.65 * t)) as i32;
            if t < 0.25 {
                cv.fill_circle(x, y, rad / 2, palette::CREAM);
            }
            cv.circle(x, y, rad, r.color);
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
                cv.line(prev.0, prev.1, next.0, next.1, palette::ICE);
                prev = next;
            }
        }
        for p in &self.particles {
            let (x, y) = cam.to_screen(p.pos);
            let c = if p.life < p.max * 0.3 { p.color.mix(palette::INK, 90) } else { p.color };
            if p.size > 1 {
                cv.fill_rect(x, y, 2, 2, c);
            } else {
                cv.put(x, y, c);
            }
        }
        for t in &self.texts {
            let (x, y) = cam.to_screen(t.pos);
            let w = font::width(&t.text);
            font::draw_outlined(cv, x - w / 2, y - 3, &t.text, t.color, palette::INK);
        }
    }
}
