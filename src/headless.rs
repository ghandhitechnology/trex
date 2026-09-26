//! Headless modes for checking art and balance without a terminal.

use std::io;
use std::path::Path;
use std::time::Instant;

use crate::bot::Bot;
use crate::engine::TICK_HZ;
use crate::game::{Controls, DEATH_TIME, Game, Scene, clock_text};
use crate::meta;
use crate::meta::hub::{Hub, Tab};
use crate::meta::save::Save;
use crate::render::canvas::Canvas;
use crate::render::font;
use crate::render::palette;
use crate::render::png;
use crate::render::sprite::bank;
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

/// Hub screens over a mid-progress save.
fn dump_hub(o: &DumpOptions) -> io::Result<usize> {
    let mut game = Game::new(meta::demo_save(), None, o.seed, o.size, true);
    let mut cv = Canvas::new(o.size.0, o.size.1);
    let last_feat = crate::content::get().meta.feats.len() - 1;
    let shots: [(&str, Tab, usize); 6] = [
        ("hub_heroes.png", Tab::Heroes, 0),
        ("hub_locked.png", Tab::Heroes, 3),
        ("hub_shop.png", Tab::Shop, 2),
        ("hub_items.png", Tab::Shop, 10),
        ("hub_feats.png", Tab::Feats, 4),
        ("hub_feats_end.png", Tab::Feats, last_feat),
    ];
    for (name, tab, pick) in shots {
        let mut hub = Hub::new(pick.min(crate::content::get().characters.len() - 1));
        hub.tab = tab;
        (hub.shop, hub.feat) = (pick, pick);
        game.scene = Scene::Hub(hub);
        for _ in 0..20 {
            game.update(&Controls::default());
        }
        game.render(&mut cv);
        png::write(&o.dir.join(name), &cv, PNG_SCALE)?;
    }
    Ok(shots.len())
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
