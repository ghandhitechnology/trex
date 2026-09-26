//! An autopilot that plays like a decent player, for headless runs and
//! balance sims. It reads what a player can see: enemies and their motion,
//! shots, and telegraphed windups (charge lanes, slam and fuse circles). A few
//! times per second of reaction it scores a ring of directions by how much
//! danger the next half second holds along each, then moves the safest way
//! that also keeps off walls, away from crowds, and near gems. It dashes out
//! of hits it can't walk away from and takes level-up items sensibly.

use crate::content;
use crate::enemies::boss::{self, Move};
use crate::enemies::{AiState, Behavior};
use crate::engine::{Rng, Vec2};
use crate::game::player::RADIUS;
use crate::game::world::World;
use crate::game::{Controls, Game, Scene};
use crate::items::{Kind, On, Stat};

/// Ticks between decisions, a stand-in for reaction time.
const THINK_EVERY: u32 = 3;
/// Seconds a shot must be on screen before the bot reacts to it.
const SHOT_REACT: f32 = 0.12;
/// How far ahead each direction is checked, seconds.
const SAMPLES: [f32; 4] = [0.1, 0.2, 0.35, 0.5];
const DIRS: usize = 16;
/// Only threats this close matter for the next half second.
const NEAR: f32 = 110.0;

/// Something that hurts on touch and moves in a straight line.
struct Mover {
    pos: Vec2,
    vel: Vec2,
    radius: f32,
}

/// A circle that blows up in `fuse` seconds.
struct Zone {
    pos: Vec2,
    radius: f32,
    fuse: f32,
}

/// A telegraphed dash from `from` to `to` starting in `fuse` seconds.
struct Lane {
    from: Vec2,
    to: Vec2,
    radius: f32,
    fuse: f32,
}

#[derive(Default)]
struct Threats {
    movers: Vec<Mover>,
    zones: Vec<Zone>,
    lanes: Vec<Lane>,
}

pub struct Bot {
    /// Always take this item when offered (balance sims).
    pub force: Option<usize>,
    rng: Rng,
    tick: u32,
    dir: Vec2,
    dash: bool,
}

impl Bot {
    pub fn new(seed: u64) -> Self {
        Bot { force: None, rng: Rng::new(seed ^ 0xb07), tick: 0, dir: Vec2::ZERO, dash: false }
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

    /// Weapons until the build has three, a synergy's last piece, hearts when
    /// hurt, then stacks of held items, with some noise so runs differ.
    fn pick(&mut self, w: &World, items: &[usize]) -> usize {
        if let Some(k) = items.iter().position(|&i| Some(i) == self.force) {
            return k;
        }
        let content = content::get();
        let weapons = w.build.held(content, Kind::Weapon);
        let hurt = w.player.hp * 2 <= w.max_hp();
        let mut best = (0, f32::MIN);
        for (k, &i) in items.iter().enumerate() {
            let it = &content.items[i];
            let mut score = self.rng.range(0.0, 1.5);
            if w.build.completes(content, i).is_some() {
                score += 3.0;
            }
            match it.kind() {
                Kind::Weapon if weapons < 3 && w.build.stacks(i) == 0 => score += 2.5,
                Kind::Weapon => score += 0.8,
                Kind::Active if w.build.held(content, Kind::Active) == 0 => score += 0.8,
                _ => {}
            }
            if w.build.stacks(i) > 0 {
                score += 0.8;
            }
            if hurt && it.stats.iter().any(|m| m.stat == Stat::MaxHp && m.add > 0.0) {
                score += 1.5;
            }
            if it.stats.iter().any(|m| m.stat == Stat::MaxHp && m.add < 0.0) && w.max_hp() <= 6 {
                score -= 2.0;
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
        if self.tick.is_multiple_of(THINK_EVERY) {
            self.think(w);
        }
        self.tick += 1;
        c.move_x = self.dir.x;
        c.move_y = self.dir.y;
        c.dash = std::mem::take(&mut self.dash) && w.player.dash_cd <= 0.0;
    }

    fn think(&mut self, w: &World) {
        let p = w.player.pos;
        let speed = w.stats.get(Stat::Speed);
        let t = threats(w);
        let crowd: Vec<Vec2> = w
            .enemies
            .iter()
            .filter(|e| e.hittable() && e.pos.dist_sq(p) < NEAR * NEAR)
            .map(|e| e.pos)
            .collect();

        let mut best = (Vec2::ZERO, f32::MAX, 0.0);
        for k in 0..=DIRS {
            let dir = if k == DIRS {
                Vec2::ZERO
            } else {
                Vec2::from_angle(k as f32 / DIRS as f32 * std::f32::consts::TAU)
            };
            let (danger, soon) = self.danger(w, &t, p, dir * speed);
            let end = w.arena.clamp(p + dir * speed * 0.5, RADIUS + 1.0);
            let cost =
                danger + place_cost(w, &crowd, end) - dir.x * self.dir.x * 0.15 - dir.y * self.dir.y * 0.15;
            if cost < best.1 {
                best = (dir, cost, soon);
            }
        }
        self.dir = best.0;

        // Dash out of a hit that walking can't avoid, or through a crowd.
        let within = |r: f32| crowd.iter().filter(|e| e.dist_sq(p) < r * r).count();
        // Space also fires actives, so use it on big packs too.
        let actives = w.triggers.iter().any(|a| a.def.on == On::Active && a.cd <= 0.0);
        self.dash = best.2 > 2.0 || within(20.0) >= 3 || (actives && within(60.0) >= 8);
        if self.dash && self.dir == Vec2::ZERO {
            self.dir = Vec2::from_angle(self.rng.angle());
        }
    }

    /// Danger along a straight path at `vel` for the next half second, and
    /// the part of it in the first 0.2 seconds.
    fn danger(&self, w: &World, t: &Threats, p: Vec2, vel: Vec2) -> (f32, f32) {
        let (mut total, mut soon) = (0.0, 0.0);
        for (k, &dt) in SAMPLES.iter().enumerate() {
            let q = w.arena.clamp(p + vel * dt, RADIUS + 1.0);
            let weight = 1.0 - k as f32 * 0.15;
            let mut d = 0.0;
            for m in &t.movers {
                let gap = (m.pos + m.vel * dt).dist_sq(q).sqrt() - m.radius - RADIUS;
                if gap < 0.0 {
                    d += 10.0;
                } else if gap < 10.0 {
                    d += (1.0 - gap / 10.0).powi(2) * 2.0;
                }
            }
            for z in &t.zones {
                if dt + 0.15 >= z.fuse && z.pos.dist_sq(q) < (z.radius + RADIUS + 4.0).powi(2) {
                    d += 12.0;
                }
            }
            for l in &t.lanes {
                if dt + 0.2 >= l.fuse && seg_dist(q, l.from, l.to) < l.radius + RADIUS + 4.0 {
                    d += 8.0;
                }
            }
            total += d * weight;
            if dt <= 0.2 {
                soon += d;
            }
        }
        (total, soon)
    }
}

/// Everything that can hurt the player soon, as seen on screen.
fn threats(w: &World) -> Threats {
    let content = content::get();
    let p = w.player.pos;
    let mut t = Threats::default();
    for e in &w.enemies {
        if e.dead || e.pos.dist_sq(p) > NEAR * NEAR {
            continue;
        }
        let def = &content.enemies[e.kind];
        if !e.hidden {
            t.movers.push(Mover { pos: e.pos, vel: e.vel + e.push, radius: def.radius });
        }
        let lane =
            |len: f32, fuse: f32| Lane { from: e.pos, to: e.pos + e.dir * len, radius: def.radius, fuse };
        match (&def.behavior, e.state) {
            (Behavior::Charge { speed, time, .. }, AiState::Windup) => {
                t.lanes.push(lane(speed * time, e.timer))
            }
            (Behavior::Orbit { .. }, AiState::Windup) => t.lanes.push(lane(e.speed * 2.6 * 1.2, e.timer)),
            (Behavior::Slam { radius, .. }, AiState::Windup) => {
                t.zones.push(Zone { pos: e.pos, radius: *radius, fuse: e.timer })
            }
            (Behavior::Kamikaze { radius, .. }, s) if s != AiState::Move => {
                t.zones.push(Zone { pos: e.pos, radius: *radius, fuse: e.timer })
            }
            (Behavior::Burrow { .. }, AiState::Windup) => {
                t.zones.push(Zone { pos: e.pos, radius: def.radius + 4.0, fuse: e.timer })
            }
            (Behavior::Boss(_), AiState::Windup) => match e.mv {
                Move::Charge { .. } => t.lanes.push(lane(boss::CHARGE_SPEED, e.timer)),
                Move::Snipe { .. } => t.lanes.push(Lane { radius: 3.0, ..lane(240.0, e.timer) }),
                Move::Slam => t.zones.push(Zone { pos: e.pos, radius: boss::SLAM_RADIUS, fuse: e.timer }),
                Move::Erupt if e.timer < boss::ERUPT_LOCK + 0.3 => {
                    t.zones.push(Zone { pos: e.pos, radius: boss::ERUPT_RADIUS, fuse: e.timer })
                }
                _ => {}
            },
            _ => {}
        }
    }
    for s in w.shots.iter().filter(|s| s.hostile && s.age >= SHOT_REACT) {
        if s.pos.dist_sq(p) < NEAR * NEAR {
            t.movers.push(Mover { pos: s.pos, vel: s.vel, radius: s.radius });
        }
    }
    t
}

/// Where the bot would rather be: off the walls, away from crowds, near gems.
fn place_cost(w: &World, crowd: &[Vec2], q: Vec2) -> f32 {
    let a = w.arena;
    let edge = 56.0;
    let wall = |d: f32| ((edge - d).max(0.0) / edge).powi(2);
    let mut cost = (wall(q.x - a.x) + wall(a.x + a.w - q.x) + wall(q.y - a.y) + wall(a.y + a.h - q.y)) * 3.0;
    for e in crowd {
        let d2 = e.dist_sq(q);
        if d2 < 70.0 * 70.0 {
            cost += (1.0 - d2.sqrt() / 70.0).powi(2) * 0.35;
        }
    }
    let p = w.player.pos;
    if let Some(gem) = w.gems.iter().min_by(|a, b| a.pos.dist_sq(p).total_cmp(&b.pos.dist_sq(p))) {
        let now = gem.pos.dist_sq(p).sqrt();
        if now < 120.0 {
            cost -= (now - gem.pos.dist_sq(q).sqrt()).clamp(-40.0, 40.0) / 40.0 * 0.6;
        }
    }
    cost
}

/// Distance from `q` to the segment `a`-`b`.
fn seg_dist(q: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((q - a).x * ab.x + (q - a).y * ab.y) / ab.len_sq().max(1e-6);
    q.dist_sq(a + ab * t.clamp(0.0, 1.0)).sqrt()
}
