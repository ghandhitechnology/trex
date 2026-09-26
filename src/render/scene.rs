//! Draws a world: floor, pickups, creatures sorted by depth, shots, effects,
//! ambience and lighting.

use super::arena::{Floor, PAD};
use super::canvas::{Blit, Canvas};
use super::light;
use super::noise;
use super::palette::{self, Color};
use super::sprite::bank;
use crate::content;
use crate::enemies::AiState;
use crate::enemies::director::WARN_TIME;
use crate::game::world::World;

enum Body {
    Enemy(usize),
    Player,
}

/// Seconds an enemy shows as a bright silhouette after spawning.
const SPAWN_POP: f32 = 0.12;

/// `death` is the death transition progress: `None` while alive, 0 at the
/// killing blow, 1 once the iris has closed.
pub fn draw_world(cv: &mut Canvas, w: &World, floor: Option<&Floor>, death: Option<f32>) {
    let bank = bank();
    let content = content::get();
    let cam = &w.camera;
    let (ox, oy) = cam.origin();
    match floor {
        Some(f) => cv.copy_view(&f.cv, ox + PAD, oy + PAD, palette::INK),
        None => cv.clear(palette::DUSK),
    }

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

    // Shadows first so no creature's shadow covers another creature.
    let mut bodies: Vec<(i32, Body)> = Vec::with_capacity(w.enemies.len() + 1);
    for (i, e) in w.enemies.iter().enumerate() {
        let def = &content.enemies[e.kind];
        let f = bank.get(def.sprite_id).first();
        let (x, y) = cam.to_screen(e.pos);
        if x < -f.w || y < -f.h || x > cv.w + f.w || y > cv.h + f.h {
            continue;
        }
        cv.blend_ellipse(x, y + f.h / 2 - 2, (f.w / 2 - 2).max(2), 2, palette::INK, 110);
        bodies.push((y, Body::Enemy(i)));
    }
    let (px, py) = cam.to_screen(w.player.pos);
    cv.blend_ellipse(px, py + 6, 5, 2, palette::INK, 120);
    bodies.push((py, Body::Player));
    bodies.sort_by_key(|b| b.0);

    w.fx.draw_under(cv, cam);
    let ch = &content.characters[w.character];
    for (_, b) in &bodies {
        match *b {
            Body::Enemy(i) => {
                let e = &w.enemies[i];
                let def = &content.enemies[e.kind];
                let s = bank.get(def.sprite_id);
                let (x, y) = cam.to_screen(e.pos);
                let windup = e.state == AiState::Windup && (e.timer * 20.0) as i32 % 2 == 0;
                let flash = if e.anim < SPAWN_POP && death.is_none() {
                    Some(palette::CREAM)
                } else if e.flash > 0.0 {
                    Some(palette::BONE)
                } else if windup {
                    Some(palette::RED)
                } else {
                    None
                };
                let f = s.frame_at(e.anim + e.phase, 6.0);
                cv.blit_centered(f, x, y, Blit { flip_x: e.vel.x < -1.0, flash, alpha: 255 });
                if e.burn_time > 0.0 {
                    burning(cv, x, y - f.h / 2, e.anim + e.phase * 3.0);
                }
                if e.slow_time > 0.0 {
                    let k = ((e.anim * 6.0) as i32).rem_euclid(f.w - 2);
                    cv.put(x - f.w / 2 + 1 + k, y + f.h / 2 - 2, palette::ICE);
                }
                if def.hp >= 40.0 && e.hp < e.max_hp {
                    hp_bar(cv, x, y - f.h / 2 - 3, e.hp / e.max_hp);
                }
            }
            Body::Player => player(cv, w, ch.sprite_id, (px, py), death),
        }
    }

    for s in &w.shots {
        let (x, y) = cam.to_screen(s.pos);
        let spr = bank.get(s.sprite);
        if !s.hostile {
            let dir = s.vel.norm();
            let (b1, b2) = (dir * 3.0, dir * 5.0);
            cv.blend(x - b1.x as i32, y - b1.y as i32, palette::AMBER, 200);
            cv.blend(x - b2.x as i32, y - b2.y as i32, palette::EMBER, 110);
            cv.blend_ellipse(x, y, 3, 3, palette::AMBER, 34);
        } else {
            let c = spr.colors.first().map_or(palette::RED, |&i| palette::color(i));
            cv.blend_ellipse(x, y, 4, 4, c, 40);
        }
        cv.blit_centered(spr.frame_at(s.age, 12.0), x, y, Blit::default());
    }

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
    if w.fx.flash > 0.0 {
        let k = w.fx.flash / 0.08;
        light::vignette(cv, (px, py), palette::RED, 0.55, (200.0 * k) as u8);
        cv.wash(palette::RED, (40.0 * k) as u8);
    }
    if let Some(p) = death {
        dying(cv, (px, py), p);
    }
}

fn player(cv: &mut Canvas, w: &World, sprite: crate::render::sprite::SpriteId, (px, py): (i32, i32), death: Option<f32>) {
    let p = &w.player;
    let s = bank().get(sprite);
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
    let f = if moving { s.frame_at(p.anim * 1.5, 6.0) } else { s.first() };
    let flash = (p.hurt > 0.0).then_some(palette::BONE);
    // Running bob: one pixel up on alternating strides.
    let bob = if moving && (p.anim * 9.0) as i32 % 2 == 0 { -1 } else { 0 };
    cv.blit_centered(f, px, py + bob, Blit { flip_x: p.facing < 0.0, flash, alpha: 255 });
}

/// Small flickering flames above a burning enemy.
fn burning(cv: &mut Canvas, x: i32, top: i32, t: f32) {
    for k in 0..3 {
        let ph = t * 7.0 + k as f32 * 2.1;
        let h = (ph.fract() * 5.0) as i32;
        let fx = x - 3 + k * 3 + (ph * 1.7).sin().round() as i32;
        let c = match h {
            0 | 1 => palette::GOLD,
            2 | 3 => palette::EMBER,
            _ => palette::BLOOD,
        };
        cv.put(fx, top + 2 - h, c);
    }
}

fn hp_bar(cv: &mut Canvas, x: i32, y: i32, frac: f32) {
    let bw = 12;
    let fill = (frac.clamp(0.0, 1.0) * bw as f32).ceil() as i32;
    cv.fill_rect(x - bw / 2 - 1, y - 1, bw + 2, 3, palette::INK);
    cv.fill_rect(x - bw / 2, y, bw, 1, palette::MAROON);
    cv.fill_rect(x - bw / 2, y, fill, 1, palette::RED);
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
