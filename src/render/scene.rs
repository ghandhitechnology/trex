//! Draws a world: floor, pickups, enemies sorted by depth, the player, shots, effects,
//! ambience and lighting.

use super::arena::{Floor, Floors, PAD};
use super::canvas::{Blit, Canvas};
use super::light;
use super::noise;
use super::palette::{self, Color};
use super::sprite::bank;
use crate::content;
use crate::enemies;
use crate::enemies::director::WARN_TIME;
use crate::game::world::World;
use crate::meta::characters::CharacterDef;

/// `death` is the death transition progress: `None` while alive, 0 at the
/// killing blow, 1 once the iris has closed.
pub fn draw_world(cv: &mut Canvas, w: &World, floors: Option<&Floors>, death: Option<f32>) {
    let bank = bank();
    let content = content::get();
    let cam = &w.camera;
    let (ox, oy) = cam.origin();
    let floor = floors.map(|fs| stage_floor(cv, w, fs, (ox + PAD, oy + PAD)));
    if floor.is_none() {
        cv.clear(palette::DUSK);
    }
    enemies::draw::ground(cv, w);

    // Spawn warnings: a ring closing in on the marker.
    let warn = bank.named("warn");
    for p in &w.director.pending {
        let (x, y) = cam.to_screen(p.pos);
        let t = (WARN_TIME - p.timer) / WARN_TIME;
        let r = (10.0 * (1.0 - t) + 3.0) as i32;
        cv.blend_ellipse(x, y + 1, r, (r / 2).max(1), palette::MAROON, 90);
        let f = warn.frame_at(t * WARN_TIME, 6.0 + t * 10.0);
        cv.blit_centered(f, x, y, Blit { alpha: 120 + (t * 135.0) as u8, ..Blit::default() });
    }
    enemies::draw::under(cv, w);

    let (gem, gem_big) = (bank.named("gem"), bank.named("gem_big"));
    for g in &w.gems {
        let (x, y) = cam.to_screen(g.pos);
        if x < -8 || y < -8 || x > cv.w + 8 || y > cv.h + 8 {
            continue;
        }
        let s = if g.value >= 5 { gem_big } else { gem };
        if g.pull && g.vel.len_sq() > 400.0 {
            // Magnet streak and a trailing sparkle.
            let back = g.vel.norm() * -6.0;
            cv.line(x, y, x + back.x as i32, y + back.y as i32, palette::BLUE);
            cv.line(x, y, x + (back.x * 0.5) as i32, y + (back.y * 0.5) as i32, palette::CYAN);
            if (g.age * 30.0) as i32 % 3 == 0 {
                cv.put(x + back.x as i32, y + back.y as i32 - 1, palette::ICE);
            }
            cv.blit_centered(s.first(), x, y, Blit::default());
            continue;
        }
        let bob = ((g.age * 5.0 + g.pos.x * 0.3).sin() * 1.5).round() as i32;
        cv.blend_ellipse(x, y + 3, 2, 1, palette::INK, 90);
        cv.blit_centered(s.frame_at(g.age + g.pos.y * 0.05, 3.0), x, y - 1 + bob, Blit::default());
        // Periodic glint.
        let phase = (g.age * 0.8 + noise::hash2(g.pos.x as i32, g.pos.y as i32, 3)).fract();
        if phase < 0.12 {
            let (gx, gy) = (x + 2, y - 3 + bob);
            cv.put(gx, gy, palette::BONE);
            if phase > 0.03 && phase < 0.09 {
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    cv.put(gx + dx, gy + dy, palette::ICE);
                }
            }
        }
    }
    crate::items::draw::under(cv, w);

    // Shadows first so no creature's shadow covers another creature.
    let mut bodies: Vec<(i32, usize)> = Vec::with_capacity(w.enemies.len());
    for (i, e) in w.enemies.iter().enumerate() {
        let def = &content.enemies[e.kind];
        let f = bank.get(def.sprite_id).first();
        let (x, y) = cam.to_screen(e.pos);
        if x < -f.w || y < -f.h || x > cv.w + f.w || y > cv.h + f.h {
            continue;
        }
        cv.blend_ellipse(x, y + f.h / 2 - 2, (f.w / 2 - 2).max(2), 2, palette::INK, 110);
        bodies.push((y, i));
    }
    let (px, py) = cam.to_screen(w.player.pos);
    cv.blend_ellipse(px, py + 6, 5, 2, palette::INK, 120);
    bodies.sort_by_key(|b| b.0);

    w.fx.draw_under(cv, cam);
    let ch = &content.characters[w.character];
    for &(_, i) in &bodies {
        let (x, y) = cam.to_screen(w.enemies[i].pos);
        enemies::draw::body(cv, w, i, x, y, death.is_some());
    }
    // The player always draws over the crowd so it never gets lost in it.
    player(cv, w, ch, (px, py), death);

    for s in &w.shots {
        let (x, y) = cam.to_screen(s.pos);
        let spr = bank.get(s.sprite);
        let tint = |k: usize, or: Color| spr.colors.get(k).map_or(or, |&i| palette::color(i));
        if !s.hostile {
            // Glow and a two-pixel trail in the shot's own colors.
            let (head, tail) = (tint(0, palette::AMBER), tint(1, palette::EMBER));
            let dir = s.vel.norm();
            let (b1, b2) = (dir * 3.0, dir * 5.0);
            cv.blend(x - b1.x as i32, y - b1.y as i32, head, 200);
            cv.blend(x - b2.x as i32, y - b2.y as i32, tail, 110);
            cv.blend_ellipse(x, y, 3, 3, head, 34);
        } else {
            cv.blend_ellipse(x, y, 4, 4, tint(0, palette::RED), 40);
        }
        cv.blit_centered(spr.frame_at(s.age, 12.0), x, y, Blit::default());
    }
    crate::items::draw::over(cv, w);

    w.fx.draw_over(cv, cam);
    if let Some(f) = floor {
        f.ambient(cv, (ox, oy), w.time);
    }

    let shade = floor.map_or(palette::INK, |f| f.biome.shade());
    light::vignette(cv, (px, py - 4), shade, 1.0, 150);
    let low = w.player.hp > 0 && w.player.hp <= 2 && death.is_none();
    if low {
        let pulse = ((w.time * 4.0).sin() * 0.5 + 0.5) * 70.0;
        light::vignette(cv, (px, py), palette::BLOOD, 0.8, 40 + pulse as u8);
    }
    if w.fx.glare > 0.0 {
        cv.wash(palette::CREAM, (w.fx.glare / enemies::death::GLARE * 110.0) as u8);
    }
    if w.fx.flash > 0.0 {
        let k = w.fx.flash / 0.08;
        light::vignette(cv, (px, py), palette::RED, 0.55, (200.0 * k) as u8);
        cv.wash(palette::RED, (40.0 * k) as u8);
    }
    match death {
        Some(p) => dying(cv, (px, py), p),
        None => enemies::draw::over(cv, w),
    }
}

/// The current stage's biome floor, crossfading from the last stage's biome.
fn stage_floor<'a>(cv: &mut Canvas, w: &World, floors: &'a Floors, (sx, sy): (i32, i32)) -> &'a Floor {
    let stages = &content::get().waves.stages;
    let d = &w.director;
    let (from, to) = (stages[d.prev_stage].biome, stages[d.stage].biome);
    let floor = floors.get(to);
    cv.copy_view(&floor.cv, sx, sy, palette::INK);
    let t = d.stage_age / enemies::draw::GROUND_FADE;
    if from != to && t < 1.0 {
        cv.blend_view(&floors.get(from).cv, sx, sy, ((1.0 - t) * 255.0) as u8);
    }
    floor
}

fn player(cv: &mut Canvas, w: &World, ch: &CharacterDef, (px, py): (i32, i32), death: Option<f32>) {
    let p = &w.player;
    let s = bank().get(ch.sprite_id);
    if let Some(t) = death {
        // Turns to stone: a bright flash, then a fossil silhouette.
        let flash = if t < 0.08 { palette::BONE } else { palette::MAUVE };
        cv.blit_centered(s.first(), px, py, Blit { flip_x: p.facing < 0.0, flash: Some(flash), alpha: 255 });
        return;
    }
    let blink = p.invuln > 0.0 && p.dash_time <= 0.0 && (p.invuln * 16.0) as i32 % 2 == 1;
    if blink {
        return;
    }
    let moving = p.vel.len_sq() > 25.0;
    let f = match ch.idle_id {
        _ if moving => s.frame_at(p.anim * 1.5, 6.0),
        Some(idle) => bank().get(idle).frame_at(w.time, 2.5),
        None => s.first(),
    };
    let flash = (p.hurt > 0.0).then_some(palette::BONE);
    // Running bob: one pixel up on alternating strides.
    let bob = if moving && (p.anim * 9.0) as i32 % 2 == 0 { -1 } else { 0 };
    cv.blit_centered(f, px, py + bob, Blit { flip_x: p.facing < 0.0, flash, alpha: 255 });
}

/// Death transition: a dark iris closes on the player.
fn dying(cv: &mut Canvas, center: (i32, i32), t: f32) {
    let dark: Color = palette::INK.mix(palette::MAROON, 70);
    let full = ((cv.w * cv.w + cv.h * cv.h) as f32).sqrt();
    let a = (t / 0.7).min(1.0);
    let a = 1.0 - (1.0 - a) * (1.0 - a);
    let mut r = full * (1.0 - a) + 22.0 * a;
    if t > 0.8 {
        r *= 1.0 - (t - 0.8) / 0.2;
    }
    cv.wash(palette::MAROON, (t.min(1.0) * 50.0) as u8);
    light::iris(cv, center, r, dark);
}
