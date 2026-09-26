//! Headless modes for checking art and balance without a terminal.

use std::io;
use std::path::Path;
use std::time::Instant;

use crate::bot::Bot;
use crate::engine::TICK_HZ;
use crate::game::{Controls, DEATH_TIME, Game, Scene, clock_text};
use crate::meta;
use crate::meta::Toasts;
use crate::meta::characters::CharacterDef;
use crate::meta::hub::{Hub, Tab};
use crate::meta::save::Save;
use crate::render::canvas::{Blit, Canvas};
use crate::render::font;
use crate::render::palette;
use crate::render::png;
use crate::render::scene::over_legs;
use crate::render::sprite::{SpriteId, bank};
use crate::sim::Start;

/// Upscale for written PNGs.
const PNG_SCALE: usize = 4;

pub struct DumpOptions<'a> {
    pub dir: &'a Path,
    pub seconds: f32,
    pub seed: u64,
    pub every: f32,
    pub size: (i32, i32),
    pub hero: Option<&'a str>,
    pub start: Start,
}

/// A `start` save on `hero`, or on the default hero.
fn start_save(hero: Option<&str>, start: Start) -> Save {
    start.save(hero.unwrap_or(&Save::default().character))
}

/// Play a bot run and write PNG frames: the title, the hub tabs, periodic
/// gameplay frames, the first level-up screens, a pause overlay, the death
/// transition and the death screen.
pub fn dump_frames(o: &DumpOptions) -> io::Result<()> {
    std::fs::create_dir_all(o.dir)?;
    let hub_frames = dump_hub(o)?;
    let mut game = Game::new(start_save(o.hero, o.start), None, o.seed, o.size, true);
    let mut cv = Canvas::new(o.size.0, o.size.1);
    let mut written = 0;
    let mut write = |name: &str, game: &Game, cv: &mut Canvas| -> io::Result<()> {
        game.render(cv);
        png::write(&o.dir.join(name), cv, PNG_SCALE)?;
        written += 1;
        Ok(())
    };

    for _ in 0..30 {
        game.update(&Controls::default());
    }
    write("title.png", &game, &mut cv)?;
    game.start_run();

    let mut bot = Bot::new(o.seed);
    let total = (o.seconds * TICK_HZ as f32) as u64;
    let every = ((o.every * TICK_HZ as f32) as u64).max(1);
    let (mut levelups, mut paused) = (0, false);
    for tick in 1..=total {
        let c = bot.controls(&game);
        game.update(&c);
        match &game.scene {
            Scene::LevelUp(offer)
                if offer.age > 0.3 && offer.age <= 0.3 + 1.0 / TICK_HZ as f32 && levelups < 3 =>
            {
                levelups += 1;
                write(&format!("levelup_{levelups}.png"), &game, &mut cv)?;
            }
            Scene::Dying(t) if (*t - DEATH_TIME * 0.5).abs() < 0.5 / TICK_HZ as f32 => {
                write("dying.png", &game, &mut cv)?;
            }
            Scene::Dead(s) if s.age > 1.6 => {
                write("dead.png", &game, &mut cv)?;
                break;
            }
            Scene::Playing if !paused && tick >= total / 2 => {
                paused = true;
                game.scene = Scene::Paused;
                write("paused.png", &game, &mut cv)?;
                game.scene = Scene::Playing;
            }
            _ => {}
        }
        if tick % every == 0 {
            write(&format!("frame_{:05}.png", tick / TICK_HZ as u64), &game, &mut cv)?;
        }
    }
    let time = game.world.as_ref().map_or(0.0, |w| w.time);
    let written = written + hub_frames;
    println!("wrote {written} frames to {} (survived {})", o.dir.display(), clock_text(time));
    Ok(())
}

/// Hub screens shown by `--dump-frames` and `--clip --show NAME`: tab and
/// hero, shop, or feat cursor (clamped to the last feat).
pub const HUB_SHOTS: [(&str, Tab, usize); 6] = [
    ("hub_heroes", Tab::Heroes, 0),
    ("hub_locked", Tab::Heroes, 3),
    ("hub_shop", Tab::Shop, 2),
    ("hub_items", Tab::Shop, 10),
    ("hub_feats", Tab::Feats, 4),
    ("hub_feats_end", Tab::Feats, usize::MAX),
];

/// A game on hub shot `shot` over a mid-progress save, settled for a moment.
fn hub_game(seed: u64, size: (i32, i32), shot: usize) -> Game {
    let content = crate::content::get();
    let (_, tab, pick) = HUB_SHOTS[shot];
    let mut game = Game::new(meta::demo_save(), None, seed, size, true);
    let mut hub = Hub::new(pick.min(content.characters.len() - 1));
    hub.tab = tab;
    (hub.shop, hub.feat) = (pick, pick.min(content.meta.feats.len() - 1));
    game.scene = Scene::Hub(hub);
    for _ in 0..20 {
        game.update(&Controls::default());
    }
    game
}

/// Hub screens over a mid-progress save.
fn dump_hub(o: &DumpOptions) -> io::Result<usize> {
    let mut cv = Canvas::new(o.size.0, o.size.1);
    for (shot, (name, ..)) in HUB_SHOTS.iter().enumerate() {
        hub_game(o.seed, o.size, shot).render(&mut cv);
        png::write(&o.dir.join(format!("{name}.png")), &cv, PNG_SCALE)?;
    }
    Ok(HUB_SHOTS.len())
}

/// What a clip shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Show {
    Run,
    Title,
    /// The run's pause screen after focus is lost.
    Paused,
    /// An index into `HUB_SHOTS`.
    Hub(usize),
}

impl Show {
    pub fn parse(s: &str) -> Result<Show, String> {
        match s {
            "run" => Ok(Show::Run),
            "title" => Ok(Show::Title),
            "paused" => Ok(Show::Paused),
            _ => HUB_SHOTS.iter().position(|h| h.0 == s).map(Show::Hub).ok_or_else(|| {
                let hubs: Vec<_> = HUB_SHOTS.iter().map(|h| h.0).collect();
                format!("--show must be run, title, paused or one of {}, not {s}", hubs.join(", "))
            }),
        }
    }
}

pub struct ClipOptions<'a> {
    pub dir: &'a Path,
    pub from: f32,
    pub seconds: f32,
    pub fps: u32,
    pub size: (i32, i32),
    pub seed: u64,
    pub hero: Option<&'a str>,
    pub start: Start,
    /// Items granted when the run starts, repeated for stacks.
    pub items: Vec<usize>,
    /// Seconds into the clip to level up; its card screen stays up a moment.
    pub levelup: Option<f32>,
    pub show: Show,
}

/// Seconds a forced level-up's cards stay up before the bot picks.
const CARD_HOLD: f32 = 1.4;

/// Fast-forward an unkillable bot run to `from` without rendering, then write
/// every frame at 1x for `seconds`. Level-up screens are skipped unless one
/// is forced with `levelup`, so the footage stays on the action.
pub fn clip(o: &ClipOptions) -> io::Result<()> {
    std::fs::create_dir_all(o.dir)?;
    let frames = ((o.seconds * o.fps as f32).round() as usize).max(1);
    let every = (TICK_HZ / o.fps.clamp(1, TICK_HZ)) as u64;
    let run = matches!(o.show, Show::Run | Show::Paused);
    let mut game = match o.show {
        Show::Hub(shot) => {
            let mut game = hub_game(o.seed, o.size, shot);
            game.toasts = Toasts::default();
            game
        }
        _ => Game::new(start_save(o.hero, o.start), None, o.seed, o.size, true),
    };
    let mut bot = Bot::new(o.seed);
    let hold = if o.levelup.is_some() { CARD_HOLD } else { 0.0 };
    let mut advance = |game: &mut Game| {
        let c = match &game.scene {
            Scene::LevelUp(offer) if offer.age < hold => Controls::default(),
            _ if run => bot.controls(game),
            _ => Controls::default(),
        };
        game.update(&c);
        if let Some(w) = game.world.as_mut() {
            w.player.hp = w.max_hp();
        }
    };

    if run {
        game.start_run();
        let w = game.world.as_mut().expect("run has a world");
        for &item in &o.items {
            w.add_item(item);
        }
        while game.world.as_ref().is_some_and(|w| w.time < o.from) {
            advance(&mut game);
        }
        if o.show == Show::Paused {
            game.focus_lost();
        }
    }

    let levelup = o.levelup.map(|s| (s * TICK_HZ as f32) as u64);
    let mut cv = Canvas::new(o.size.0, o.size.1);
    let (mut tick, mut written) = (0u64, 0);
    loop {
        if o.levelup.is_none() && matches!(game.scene, Scene::LevelUp(_)) {
            advance(&mut game);
            continue;
        }
        if tick.is_multiple_of(every) {
            game.render(&mut cv);
            png::write(&o.dir.join(format!("{written:04}.png")), &cv, 1)?;
            written += 1;
            if written == frames {
                break;
            }
        }
        if levelup == Some(tick)
            && let Some(w) = game.world.as_mut()
        {
            w.player.xp = w.player.xp_next;
        }
        advance(&mut game);
        tick += 1;
    }
    let time = game.world.as_ref().map_or(0.0, |w| w.time);
    println!("wrote {written} frames to {} (at {})", o.dir.display(), clock_text(time));
    Ok(())
}

/// An unkillable bot run with visuals on that renders and upscales every
/// tick like the live loop, printing entity counts and frame cost per minute.
pub fn stress(minutes: f32, seed: u64, size: (i32, i32), hero: Option<&str>) {
    let mut game = Game::new(start_save(hero, Start::Fresh), None, seed, size, true);
    game.start_run();
    let mut bot = Bot::new(seed);
    let mut cv = Canvas::new(size.0, size.1);
    let mut rgb = Vec::new();
    // The live loop's upscale for this size.
    let k = ((crate::app::PIXEL_BUDGET / (size.0 * size.1) as f32).sqrt() as usize).max(1);
    let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0;
    let (mut sim, mut draw, mut ticks) = ((0.0f64, 0.0f64), (0.0f64, 0.0f64), 0u32);
    println!("min  enemies shots gems  parts texts  sim avg/max ms  draw avg/max ms");
    for tick in 1..=(minutes * 60.0 * TICK_HZ as f32) as u64 {
        if let Some(w) = game.world.as_mut() {
            w.player.hp = w.max_hp();
        }
        let c = bot.controls(&game);
        let t0 = Instant::now();
        game.update(&c);
        let t1 = Instant::now();
        game.render(&mut cv);
        cv.scaled_rgb(k, &mut rgb);
        let (s, d) = (ms(t1 - t0), ms(t1.elapsed()));
        sim = (sim.0 + s, sim.1.max(s));
        draw = (draw.0 + d, draw.1.max(d));
        ticks += 1;
        if tick % (60 * TICK_HZ as u64) == 0 {
            let w = game.world.as_ref().expect("run has a world");
            let n = f64::from(ticks);
            println!(
                "{:>3}  {:>7} {:>5} {:>4} {:>6} {:>5}  {:>6.2} / {:<6.2} {:>6.2} / {:<6.2}",
                tick / (60 * TICK_HZ as u64),
                w.enemies.len() + w.director.pending.len(),
                w.shots.len(),
                w.gems.len(),
                w.fx.particles.len(),
                w.fx.texts.len(),
                sim.0 / n,
                sim.1,
                draw.0 / n,
                draw.1,
            );
            (sim, draw, ticks) = ((0.0, 0.0), (0.0, 0.0), 0);
        }
    }
}

/// Every sprite in the bank on one labeled sheet.
pub fn sheet(path: &Path) -> io::Result<()> {
    let sprites = bank().all();
    let cell_w = sprites.iter().map(|s| s.first().w.max(font::width(s.name))).max().unwrap_or(16) + 6;
    let cell_h = sprites.iter().map(|s| s.first().h).max().unwrap_or(16) + 14;
    let cols = 5;
    let rows = sprites.len().div_ceil(cols) as i32;
    let max_frames = sprites.iter().map(|s| s.frames.len()).max().unwrap_or(1) as i32;
    let mut cv = Canvas::new(cell_w * cols as i32 * max_frames.min(2), cell_h * rows);
    cv.clear(palette::DUSK);
    for (i, s) in sprites.iter().enumerate() {
        let x = (i % cols) as i32 * cell_w * max_frames.min(2);
        let y = (i / cols) as i32 * cell_h;
        for (k, f) in s.frames.iter().take(2).enumerate() {
            cv.blit(f, x + 3 + k as i32 * cell_w, y + 3, Default::default());
        }
        font::draw(&mut cv, x + 3, y + cell_h - 8, s.name, palette::FOG);
    }
    png::write(path, &cv, PNG_SCALE)?;
    println!("wrote {} sprites to {}", sprites.len(), path.display());
    Ok(())
}

/// Every hero's poses on one sheet, one hero per row: idle, walk, attack,
/// the attack over each stride (what running while attacking shows), and
/// dash. A cream dot marks the muzzle on attack frames.
pub fn poses(path: &Path) -> io::Result<()> {
    let bank = bank();
    let heroes = &crate::content::get().characters;
    let (cell, row_h, label_w) = (19, 24, 34);
    let groups = |ch: &CharacterDef| {
        let frames =
            |id: Option<SpriteId>| id.map_or(Vec::new(), |id| bank.get(id).frames.iter().collect::<Vec<_>>());
        let walk = frames(Some(ch.sprite_id));
        let attack = frames(ch.attack_id);
        let running: Vec<_> = if ch.legs > 0 {
            walk.iter().flat_map(|s| attack.iter().map(|a| over_legs(a, s, ch.legs + 1))).collect()
        } else {
            Vec::new()
        };
        (frames(ch.idle_id), walk, attack, running, frames(ch.dash_id))
    };
    let widest = heroes
        .iter()
        .map(|ch| {
            let (i, w, a, r, d) = groups(ch);
            i.len() + w.len() + a.len() + r.len() + d.len() + 4
        })
        .max()
        .unwrap_or(1) as i32;
    let mut cv = Canvas::new(label_w + widest * cell, row_h * heroes.len() as i32 + 4);
    cv.clear(palette::DUSK);
    for (r, ch) in heroes.iter().enumerate() {
        let y = 2 + r as i32 * row_h;
        font::draw(&mut cv, 2, y + 7, &ch.id.to_uppercase(), palette::FOG);
        let (idle, walk, attack, running, dash) = groups(ch);
        let running: Vec<_> = running.iter().collect();
        let mut x = label_w;
        for (frames, muzzle) in [(idle, false), (walk, false), (attack, true), (running, true), (dash, false)]
        {
            for f in frames {
                let (cx, cy) = (x + cell / 2, y + row_h / 2);
                cv.fill_rect(x + 1, y + 1, cell - 2, row_h - 2, palette::NIGHT);
                cv.blit_centered(f, cx, cy, Blit::default());
                if muzzle {
                    let (mx, my) = (ch.muzzle.0.round() as i32, ch.muzzle.1.round() as i32);
                    cv.put(cx + mx, cy + my, palette::CREAM);
                }
                x += cell;
            }
            x += 4;
        }
    }
    png::write(path, &cv, 6)?;
    println!("wrote {} heroes to {}", heroes.len(), path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Visual effects (hit stop in particular) must never change a run.
    #[test]
    fn visuals_do_not_change_the_run() {
        let run = |seed: u64, visuals: bool| {
            let mut game = Game::new(Save::default(), None, seed, (256, 144), visuals);
            game.start_run();
            let mut bot = Bot::new(seed);
            for _ in 0..TICK_HZ * 200 {
                let c = bot.controls(&game);
                game.update(&c);
                let w = game.world.as_ref().expect("run has a world");
                if w.time >= 150.0 || matches!(game.scene, Scene::Dead(_)) {
                    break;
                }
            }
            let w = game.world.as_ref().expect("run has a world");
            (w.time.to_bits(), w.kills, w.player.hp, w.player.level)
        };
        for seed in 1..=8 {
            assert_eq!(run(seed, false), run(seed, true), "seed {seed}");
        }
    }
}
