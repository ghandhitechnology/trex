//! Game flow: scenes, runs, and the glue between input, world, save, and UI.

pub mod pickups;
pub mod player;
pub mod shots;
pub mod world;

use std::path::PathBuf;

use crate::content;
use crate::engine::DT;
use crate::items;
use crate::meta::hub::{Hub, HubAction};
use crate::meta::save::{self, Save};
use crate::meta::{self, Reward, RunResult, Toasts};
use crate::render::canvas::Canvas;
use crate::render::sprite::bank;
use crate::render::{arena, palette, scene};
use crate::ui;
use world::World;

/// Input for one tick, from the keyboard or the bot. Buttons are edges
/// (pressed since the last tick); movement is the held direction.
#[derive(Clone, Copy, Debug, Default)]
pub struct Controls {
    pub move_x: f32,
    pub move_y: f32,
    /// Space: dash in play, confirm in menus.
    pub dash: bool,
    pub confirm: bool,
    /// P or Esc.
    pub pause: bool,
    /// Q.
    pub quit: bool,
    /// H: bank the run and go home from pause.
    pub home: bool,
    pub yes: bool,
    pub no: bool,
    /// Direction edges for menus (WASD or arrows).
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    /// Number keys 1-9 in menus.
    pub pick: Option<u8>,
}

pub struct Offer {
    pub items: Vec<usize>,
    pub cursor: usize,
    pub age: f32,
}

pub struct Summary {
    pub time: f32,
    pub kills: u32,
    pub level: u32,
    pub reward: Reward,
    pub age: f32,
}

pub enum Scene {
    Title,
    /// Hero select, shop, and feats.
    Hub(Hub),
    Playing,
    Paused,
    LevelUp(Offer),
    Dying(f32),
    Dead(Summary),
}

#[derive(PartialEq, Eq, Debug)]
pub enum Signal {
    None,
    Quit,
}

pub const DEATH_TIME: f32 = 1.4;
const MENU_GUARD: f32 = 0.35;

pub struct Game {
    pub scene: Scene,
    pub world: Option<World>,
    pub save: Save,
    save_path: Option<PathBuf>,
    pub character: usize,
    pub quit_prompt: bool,
    /// UI animation clock, seconds.
    pub clock: f32,
    pub view: (i32, i32),
    pub floors: Option<arena::Floors>,
    /// Feat banners shown on menus.
    pub toasts: Toasts,
    seed: u64,
    runs: u64,
    visuals: bool,
    recorded: bool,
    /// A Space press that landed during hit stop, replayed on the next step.
    held_dash: bool,
}

impl Game {
    /// `save_path: None` keeps the save in memory only (headless modes).
    pub fn new(
        mut save: Save,
        save_path: Option<PathBuf>,
        seed: u64,
        view: (i32, i32),
        visuals: bool,
    ) -> Self {
        let content = content::get();
        let character = content
            .character(&save.character)
            .filter(|&i| meta::hero_unlocked(&save, content, i))
            .unwrap_or(0);
        // Grants feats that older saves or newly added feats already qualify for.
        let mut toasts = Toasts::default();
        toasts.push(&meta::check_feats(&mut save, content));
        let game = Game {
            scene: Scene::Title,
            world: None,
            save,
            save_path,
            character,
            quit_prompt: false,
            clock: 0.0,
            view,
            floors: visuals.then(|| arena::Floors::new(world::ARENA, seed)),
            toasts,
            seed,
            runs: 0,
            visuals,
            recorded: false,
            held_dash: false,
        };
        if game.toasts.current().is_some() {
            game.store_save();
        }
        game
    }

    fn store_save(&self) {
        if let Some(path) = &self.save_path {
            let _ = save::store(path, &self.save);
        }
    }

    pub fn start_run(&mut self) {
        let seed = self.seed.wrapping_add(self.runs.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        self.runs += 1;
        let mut w = World::new(seed, self.character, self.view, self.visuals);
        w.build.bonus = meta::run_mods(&self.save, content::get());
        w.refresh_build();
        w.player.hp = w.max_hp();
        self.world = Some(w);
        if self.visuals {
            self.floors = Some(arena::Floors::new(world::ARENA, seed));
        }
        self.recorded = false;
        self.held_dash = false;
        self.scene = Scene::Playing;
    }

    pub fn set_view(&mut self, view: (i32, i32)) {
        self.view = view;
        if let Some(w) = &mut self.world {
            w.set_view(view);
        }
    }

    pub fn focus_lost(&mut self) {
        if matches!(self.scene, Scene::Playing) {
            self.scene = Scene::Paused;
        }
    }

    /// True while the world is simulating (used for render pacing).
    pub fn is_live(&self) -> bool {
        matches!(self.scene, Scene::Playing | Scene::Dying(_))
    }

    pub fn update(&mut self, c: &Controls) -> Signal {
        self.clock += DT;
        if matches!(self.scene, Scene::Title | Scene::Hub(_) | Scene::Dead(_)) {
            self.toasts.update(DT);
        }
        if self.quit_prompt {
            if c.yes || c.quit {
                self.finish_run();
                return Signal::Quit;
            }
            if c.no || c.pause {
                self.quit_prompt = false;
            }
            return Signal::None;
        }
        if c.quit {
            self.quit_prompt = true;
            return Signal::None;
        }
        match &mut self.scene {
            Scene::Title => {
                if c.confirm {
                    self.scene = Scene::Hub(Hub::new(self.character));
                }
            }
            Scene::Hub(hub) => match hub.update(c, &mut self.save, meta::screens::shop_cols(self.view.0)) {
                HubAction::None => {}
                HubAction::Play(i) => {
                    self.character = i;
                    self.start_run();
                }
                HubAction::Back => {
                    if meta::hero_unlocked(&self.save, content::get(), hub.hero) {
                        self.character = hub.hero;
                    }
                    self.scene = Scene::Title;
                }
                HubAction::Bought(feats) => {
                    self.toasts.push(&feats);
                    self.store_save();
                }
            },
            Scene::Playing => {
                if c.pause {
                    self.scene = Scene::Paused;
                    return Signal::None;
                }
                let w = self.world.as_mut().expect("playing without a world");
                if w.fx.hitstop > 0.0 {
                    w.fx.hitstop -= DT;
                    self.held_dash |= c.dash;
                    return Signal::None;
                }
                let c = Controls { dash: c.dash || std::mem::take(&mut self.held_dash), ..*c };
                w.step(&c);
                if w.player.hp <= 0 {
                    let pos = w.player.pos;
                    let hero = bank().get(content::get().characters[w.character].sprite_id);
                    let mut colors: Vec<_> = hero.colors.iter().take(2).map(|&c| palette::color(c)).collect();
                    colors.extend([palette::BONE, palette::RED]);
                    w.fx.burst(pos, &colors, 40, 140.0);
                    w.fx.shake(0.9);
                    self.scene = Scene::Dying(DEATH_TIME);
                } else if w.pending_levels > 0 {
                    self.open_offer();
                }
            }
            Scene::Paused => {
                if c.pause || c.confirm {
                    self.scene = Scene::Playing;
                } else if c.home {
                    self.finish_run();
                    self.world = None;
                    self.scene = Scene::Hub(Hub::new(self.character));
                }
            }
            Scene::LevelUp(offer) => {
                offer.age += DT;
                let n = offer.items.len();
                if c.left {
                    offer.cursor = (offer.cursor + n - 1) % n;
                }
                if c.right {
                    offer.cursor = (offer.cursor + 1) % n;
                }
                let choice = match c.pick {
                    Some(k) if (1..=n as u8).contains(&k) => Some(k as usize - 1),
                    _ if c.confirm && offer.age >= MENU_GUARD => Some(offer.cursor),
                    _ => None,
                };
                if let Some(i) = choice {
                    let item = offer.items[i];
                    let w = self.world.as_mut().expect("level up without a world");
                    w.pending_levels -= 1;
                    w.add_item(item);
                    if w.pending_levels > 0 {
                        self.open_offer();
                    } else {
                        self.scene = Scene::Playing;
                    }
                }
            }
            Scene::Dying(t) => {
                *t -= DT;
                let done = *t <= 0.0;
                if let Some(w) = &mut self.world {
                    w.step_fx();
                }
                if done {
                    let summary = self.finish_run();
                    self.scene = Scene::Dead(summary);
                }
            }
            Scene::Dead(s) => {
                s.age += DT;
                if s.age >= MENU_GUARD * 2.0 {
                    if c.confirm {
                        self.start_run();
                    } else if c.pause {
                        self.scene = Scene::Hub(Hub::new(self.character));
                        self.world = None;
                    }
                }
            }
        }
        Signal::None
    }

    fn open_offer(&mut self) {
        let content = content::get();
        let save = &self.save;
        let w = self.world.as_mut().expect("offer without a world");
        loop {
            let items = items::roll_offer(
                content,
                &w.build,
                |it| meta::is_unlocked(save, &it.id, it.unlock),
                &mut w.rng,
                3,
            );
            if !items.is_empty() {
                self.scene = Scene::LevelUp(Offer { items, cursor: 0, age: 0.0 });
                return;
            }
            // Every item is maxed: heal instead of offering nothing.
            w.pending_levels -= 1;
            w.heal(2);
            if w.pending_levels == 0 {
                self.scene = Scene::Playing;
                return;
            }
        }
    }

    /// Bank a run in progress before the program exits.
    pub fn quit(&mut self) {
        self.finish_run();
    }

    /// Record the current run in the save (once) and write it to disk.
    fn finish_run(&mut self) -> Summary {
        let content = content::get();
        let Some(w) = &self.world else {
            return Summary { time: 0.0, kills: 0, level: 0, reward: Reward::default(), age: 0.0 };
        };
        let result = RunResult {
            character: &content.characters[self.character].id,
            time: w.time,
            kills: w.kills,
            level: w.player.level,
            build: &w.build,
        };
        let reward = if self.recorded {
            Reward { best: self.save.best.get(result.character).copied().unwrap_or(0.0), ..Reward::default() }
        } else {
            self.recorded = true;
            self.save.character = result.character.to_string();
            let r = meta::record_run(&mut self.save, content, &result);
            self.toasts.push(&r.feats);
            self.store_save();
            r
        };
        Summary { time: w.time, kills: w.kills, level: w.player.level, reward, age: 0.0 }
    }

    pub fn render(&self, cv: &mut Canvas) {
        match (&self.scene, &self.world) {
            (Scene::Hub(hub), _) => meta::screens::hub(
                cv,
                hub,
                &self.save,
                self.floors.as_ref().map(arena::Floors::menu),
                self.clock,
            ),
            (Scene::Title, _) | (_, None) => ui::screens::title(cv, self),
            (scene, Some(w)) => {
                let death = match scene {
                    Scene::Dying(t) => Some(1.0 - t / DEATH_TIME),
                    Scene::Dead(_) => Some(1.0),
                    _ => None,
                };
                scene::draw_world(cv, w, self.floors.as_ref(), death);
                if matches!(scene, Scene::Playing | Scene::Paused) {
                    ui::hud::draw(cv, w, self.clock);
                }
                match scene {
                    Scene::Paused => ui::screens::paused(cv, w, self.clock),
                    Scene::LevelUp(o) => ui::screens::level_up(cv, w, o, self.clock),
                    Scene::Dead(s) => ui::screens::dead(cv, w, s, self.clock),
                    _ => {}
                }
            }
        }
        if matches!(self.scene, Scene::Title | Scene::Hub(_) | Scene::Dead(_)) {
            meta::screens::toast(cv, &self.toasts);
        }
        if self.quit_prompt {
            ui::screens::quit_prompt(cv);
        }
    }
}

/// Seconds as M:SS.
pub fn clock_text(secs: f32) -> String {
    let s = secs.max(0.0) as u32;
    format!("{}:{:02}", s / 60, s % 60)
}
