//! One run's simulation state and the fixed-step update order.

use std::collections::VecDeque;

use super::{Controls, pickups, player, shots};
use crate::content;
use crate::enemies::director::{self, Director};
use crate::enemies::{self, Enemy};
use crate::engine::{DT, Grid, Rect, Rng, Vec2};
use crate::items::effects::{self, ActiveTrigger, GameEvent};
use crate::items::weapons::{self, Gear, ShotTag};
use crate::items::{Build, On, Owner, Source, Stat, Stats};
use crate::render::camera::Camera;
use crate::render::fx::Fx;
use crate::render::palette;
use crate::render::sprite::SpriteId;

/// Arena size in world pixels. The view is smaller and follows the player.
pub const ARENA: Rect = Rect::new(0.0, 0.0, 512.0, 320.0);
/// Level-up burst: shove radius (the gold ring's size), force, and grace seconds.
const LEVEL_SHOVE_RADIUS: f32 = 34.0;
const LEVEL_SHOVE: f32 = 150.0;
const LEVEL_GRACE: f32 = 0.5;

#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub pos: Vec2,
    pub vel: Vec2,
    pub damage: f32,
    pub radius: f32,
    pub pierce: u32,
    pub bounce: u32,
    pub life: f32,
    pub homing: f32,
    pub knock: f32,
    /// Trigger generation: 0 for weapon shots, +1 per proc.
    pub depth: u8,
    pub sprite: SpriteId,
    pub hostile: bool,
    pub hostile_damage: i32,
    pub hits: [u32; 8],
    pub nhits: usize,
    pub age: f32,
    pub dead: bool,
    /// Item weapon and effect data.
    pub tag: ShotTag,
}

impl Shot {
    pub fn hostile(pos: Vec2, vel: Vec2, damage: i32, sprite: SpriteId) -> Self {
        Shot {
            pos,
            vel,
            damage: 0.0,
            radius: 2.5,
            pierce: 0,
            bounce: 0,
            life: 4.0,
            homing: 0.0,
            knock: 0.0,
            depth: 0,
            sprite,
            hostile: true,
            hostile_damage: damage,
            hits: [0; 8],
            nhits: 0,
            age: 0.0,
            dead: false,
            tag: ShotTag::default(),
        }
    }

    pub fn has_hit(&self, uid: u32) -> bool {
        self.hits[..self.nhits.min(8)].contains(&uid)
    }

    pub fn mark(&mut self, uid: u32) {
        self.hits[self.nhits % 8] = uid;
        self.nhits += 1;
    }
}

#[derive(Clone, Debug)]
pub struct Gem {
    pub pos: Vec2,
    pub vel: Vec2,
    pub value: u32,
    pub pull: bool,
    pub age: f32,
    pub dead: bool,
}

/// One instance of damage to an enemy.
#[derive(Clone, Copy, Debug, Default)]
pub struct Hit {
    pub damage: f32,
    pub knock: Vec2,
    pub crit: bool,
    pub depth: u8,
    /// Whether this hit fires `Hit` triggers (false for burn ticks).
    pub procs: bool,
    pub source: Source,
}

#[derive(Clone, Debug, Default)]
pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub facing: f32,
    pub hp: i32,
    pub invuln: f32,
    pub hurt: f32,
    pub dash_time: f32,
    pub dash_cd: f32,
    pub dash_dir: Vec2,
    /// Seconds a dash press waits for the cooldown to end.
    pub dash_buffer: f32,
    pub fire_cd: f32,
    /// Seconds left of the attack pose; the hero faces `aim` meanwhile.
    pub attack: f32,
    /// Direction of the last aimed shot.
    pub aim: Vec2,
    pub regen: f32,
    pub xp: f32,
    pub xp_next: f32,
    pub level: u32,
    pub anim: f32,
}

pub struct World {
    pub time: f32,
    pub rng: Rng,
    pub arena: Rect,
    pub view: (i32, i32),
    pub camera: Camera,
    pub character: usize,
    pub player: Player,
    pub build: Build,
    pub stats: Stats,
    /// Stats for item weapons and effects; see `Build::item_stats`.
    pub item_stats: Stats,
    pub triggers: Vec<ActiveTrigger>,
    pub enemies: Vec<Enemy>,
    pub shots: Vec<Shot>,
    pub gems: Vec<Gem>,
    pub events: VecDeque<GameEvent>,
    pub director: Director,
    pub grid: Grid,
    pub fx: Fx,
    pub kills: u32,
    /// Level-ups earned but not yet picked.
    pub pending_levels: u32,
    /// Item weapons and the objects items create.
    pub gear: Gear,
    uid: u32,
}

impl World {
    pub fn new(seed: u64, character: usize, view: (i32, i32), visuals: bool) -> Self {
        let center = ARENA.center();
        let mut w = World {
            time: 0.0,
            rng: Rng::new(seed),
            arena: ARENA,
            view,
            camera: Camera::new(center, view),
            character,
            player: Player {
                pos: center,
                facing: 1.0,
                level: 1,
                xp_next: xp_for_level(1) as f32,
                ..Player::default()
            },
            build: Build::default(),
            stats: Stats::defaults(),
            item_stats: Stats::defaults(),
            triggers: Vec::new(),
            enemies: Vec::new(),
            shots: Vec::new(),
            gems: Vec::new(),
            events: VecDeque::new(),
            director: Director::new(),
            grid: Grid::new(ARENA, 24.0),
            fx: Fx::new(seed, visuals),
            kills: 0,
            pending_levels: 0,
            gear: Gear::default(),
            uid: 0,
        };
        w.refresh_build();
        w.player.hp = w.max_hp();
        w
    }

    pub fn next_uid(&mut self) -> u32 {
        self.uid += 1;
        self.uid
    }

    pub fn stats_of(&self, owner: Owner) -> &Stats {
        match owner {
            Owner::Hero => &self.stats,
            Owner::Item => &self.item_stats,
        }
    }

    pub fn max_hp(&self) -> i32 {
        self.stats.count(Stat::MaxHp) as i32
    }

    pub fn set_view(&mut self, view: (i32, i32)) {
        self.view = view;
        self.camera.view = view;
    }

    /// Recompute stats and triggers after the build changed. Keeps trigger cooldowns
    /// for triggers that still exist and heals by any MaxHp gained.
    pub fn refresh_build(&mut self) {
        let content = content::get();
        let ch = &content.characters[self.character];
        let old_max = self.max_hp();
        self.stats = self.build.stats(ch, content);
        self.item_stats = self.build.item_stats(ch, content);
        let old = std::mem::take(&mut self.triggers);
        self.triggers = self
            .build
            .triggers(ch, content)
            .into_iter()
            .enumerate()
            .map(|(i, def)| {
                let cd = old.get(i).filter(|o| o.def == def).map_or(def.initial_cd(), |o| o.cd);
                ActiveTrigger { def, cd }
            })
            .collect();
        let gained = self.max_hp() - old_max;
        if gained > 0 && self.player.hp > 0 {
            self.player.hp += gained;
        }
        self.player.hp = self.player.hp.min(self.max_hp());
        weapons::refresh(self);
    }

    /// Take a level-up item. The burst shoves nearby enemies back and the
    /// hero gets a moment of grace, so a pick never lands straight into a hit.
    pub fn add_item(&mut self, item: usize) {
        self.build.add(item);
        self.refresh_build();
        let pos = self.player.pos;
        self.fx.level_up(pos);
        // Dead enemies were removed after the last grid build; the shove and
        // LevelUp triggers look enemies up by grid index.
        self.rebuild_grid();
        let enemies = &content::get().enemies;
        for i in self.enemies_in(pos, LEVEL_SHOVE_RADIUS) {
            let e = &mut self.enemies[i];
            e.push += (e.pos - pos).norm() * LEVEL_SHOVE / enemies[e.kind].mass.max(1.0);
        }
        self.player.invuln = self.player.invuln.max(LEVEL_GRACE);
        self.events.push_back(GameEvent::at(On::LevelUp, self.player.pos, 0));
        effects::process(self);
    }

    pub fn heal(&mut self, amount: i32) {
        let before = self.player.hp;
        self.player.hp = (self.player.hp + amount).min(self.max_hp());
        let healed = self.player.hp - before;
        if healed > 0 {
            let pos = self.player.pos + Vec2::new(0.0, -10.0);
            self.fx.text(pos, format!("+{healed}"), palette::LIME);
        }
    }

    pub fn rebuild_grid(&mut self) {
        self.grid.rebuild(self.enemies.iter().map(|e| e.pos));
    }

    /// Nearest live enemy within `range` that passes `ok`.
    pub fn nearest_enemy(&self, pos: Vec2, range: f32, ok: impl Fn(&Enemy) -> bool) -> Option<usize> {
        let mut best = None;
        let mut best_d = range * range;
        self.grid.query(pos, range, |i| {
            let Some(e) = self.enemies.get(i) else { return };
            let d = e.pos.dist_sq(pos);
            if e.hittable() && d < best_d && ok(e) {
                best_d = d;
                best = Some(i);
            }
        });
        best
    }

    /// Indices of live enemies whose bodies overlap the circle.
    pub fn enemies_in(&self, pos: Vec2, radius: f32) -> Vec<usize> {
        let enemies = &content::get().enemies;
        let mut out = Vec::new();
        self.grid.query(pos, radius + 12.0, |i| {
            let Some(e) = self.enemies.get(i) else { return };
            let r = radius + enemies[e.kind].radius;
            if e.hittable() && e.pos.dist_sq(pos) < r * r {
                out.push(i);
            }
        });
        out
    }

    /// Apply damage to an enemy. Emits Hit/Crit/Kill events and drops loot.
    pub fn damage_enemy(&mut self, i: usize, hit: Hit) -> bool {
        let def = &content::get().enemies[self.enemies[i].kind];
        let e = &mut self.enemies[i];
        if !e.hittable() {
            return false;
        }
        e.hp -= enemies::absorb(e, hit.damage);
        e.hit_flash();
        e.push += hit.knock / def.mass;
        let pos = e.pos;
        let killed = e.hp <= 0.0;
        let color = if hit.crit { palette::GOLD } else { palette::BONE };
        if hit.procs || hit.damage >= 1.0 {
            self.fx.number(pos + Vec2::new(0.0, -8.0), hit.damage, color);
        }
        let (depth, source) = (hit.depth, hit.source);
        if hit.procs {
            self.events.push_back(GameEvent { on: On::Hit, pos, target: Some(i), depth, source });
            if hit.crit {
                self.events.push_back(GameEvent { on: On::Crit, pos, target: Some(i), depth, source });
            }
        }
        if killed {
            self.enemies[i].dead = true;
            self.kills += 1;
            self.events.push_back(GameEvent { on: On::Kill, pos, target: Some(i), depth, source });
            enemies::died(self, i);
        }
        killed
    }

    /// Advance one fixed tick.
    pub fn step(&mut self, c: &Controls) {
        self.time += DT;
        player::update(self, c, DT);
        self.rebuild_grid();
        player::fire(self, DT);
        weapons::update(self, c, DT);
        director::update(self, DT);
        enemies::update(self, DT);
        self.rebuild_grid();
        shots::update(self, DT);
        effects::tick(self, DT);
        self.enemies.retain(|e| !e.dead);
        pickups::update(self, DT);
        self.fx.update(DT);
        let kick = std::mem::take(&mut self.fx.kick);
        self.camera.kick(kick);
        self.camera.update(self.player.pos, self.arena, DT, &mut self.fx.rng);
    }

    /// Visual-only tick used while the death animation plays.
    pub fn step_fx(&mut self) {
        self.fx.update(DT);
        let kick = std::mem::take(&mut self.fx.kick);
        self.camera.kick(kick);
        self.camera.update(self.player.pos, self.arena, DT, &mut self.fx.rng);
    }
}

/// XP needed to go from `level` to `level + 1`.
pub fn xp_for_level(level: u32) -> u32 {
    3 + level * 3 + level * level / 3
}
