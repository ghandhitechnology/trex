//! A simple autopilot for headless runs: kite away from crowds, grab gems,
//! stay off the walls, dash when cornered, and take level-up items the way a
//! sensible player would.

use crate::content;
use crate::engine::{Rng, Vec2};
use crate::game::world::World;
use crate::game::{Controls, Game, Scene};
use crate::items::Kind;

pub struct Bot {
    rng: Rng,
    wander: Vec2,
    wander_t: f32,
}

impl Bot {
    pub fn new(seed: u64) -> Self {
        Bot { rng: Rng::new(seed ^ 0xb07), wander: Vec2::new(1.0, 0.0), wander_t: 0.0 }
    }

    pub fn controls(&mut self, g: &Game) -> Controls {
        let mut c = Controls::default();
        match &g.scene {
            Scene::Title | Scene::Hub(_) => c.confirm = true,
            Scene::LevelUp(o) if o.age > 0.5 => {
                c.pick = g.world.as_ref().map(|w| 1 + self.pick(w, &o.items) as u8);
            }
            Scene::Playing => self.steer(g, &mut c),
            _ => {}
        }
        c
    }

    /// A synergy's last piece first, then weapons until the build has three,
    /// then stacks of held items, with a little noise so runs differ.
    fn pick(&mut self, w: &World, items: &[usize]) -> usize {
        let content = content::get();
        let weapons = w.build.held(content, Kind::Weapon);
        let mut best = (0, f32::MIN);
        for (k, &i) in items.iter().enumerate() {
            let mut score = self.rng.range(0.0, 1.0);
            if w.build.completes(content, i).is_some() {
                score += 4.0;
            }
            if content.items[i].kind() == Kind::Weapon && weapons < 3 {
                score += 2.0;
            }
            if w.build.stacks(i) > 0 {
                score += 1.0;
            }
            if score > best.1 {
                best = (k, score);
            }
        }
        best.0
    }

    fn steer(&mut self, g: &Game, c: &mut Controls) {
        let Some(w) = &g.world else { return };
        // Frozen ticks are skipped by the game; stay in step with the sim.
        if w.fx.hitstop > 0.0 {
            return;
        }
        let enemies = &content::get().enemies;
        let p = w.player.pos;

        self.wander_t -= crate::engine::DT;
        if self.wander_t <= 0.0 {
            self.wander_t = self.rng.range(0.8, 2.0);
            self.wander = Vec2::from_angle(self.rng.angle());
        }

        let mut force = self.wander * 0.25;
        let mut close = 0;
        for e in &w.enemies {
            let d = p - e.pos;
            let dist = d.len();
            let reach = 56.0 + enemies[e.kind].radius;
            if dist < reach {
                force += d.norm() * ((reach - dist) / reach).powi(2) * 3.0;
                if dist < 16.0 {
                    close += 1;
                }
            }
        }
        // Sidestep shots on course to hit within half a second.
        for s in w.shots.iter().filter(|s| s.hostile) {
            let d = p - s.pos;
            let along = d.x * s.vel.x + d.y * s.vel.y;
            let speed_sq = s.vel.len_sq().max(1.0);
            let t = along / speed_sq;
            if (0.0..0.5).contains(&t) {
                let miss = d - s.vel * t;
                if miss.len() < 10.0 {
                    let side = if miss.len_sq() > 0.01 { miss.norm() } else { s.vel.norm().perp() };
                    force += side * 2.5 * (1.0 - t * 1.5);
                }
            }
        }
        if let Some(gem) = w.gems.iter().min_by(|a, b| a.pos.dist_sq(p).total_cmp(&b.pos.dist_sq(p))) {
            let d = gem.pos - p;
            if d.len() < 100.0 {
                force += d.norm() * 0.6;
            }
        }
        let a = w.arena;
        let edge = 36.0;
        force.x += ((a.x + edge - p.x).max(0.0) - (p.x - (a.x + a.w - edge)).max(0.0)) / edge * 2.0;
        force.y += ((a.y + edge - p.y).max(0.0) - (p.y - (a.y + a.h - edge)).max(0.0)) / edge * 2.0;

        let dir = force.norm();
        c.move_x = dir.x;
        c.move_y = dir.y;
        c.dash = close >= 2 && w.player.dash_cd <= 0.0;
    }
}
