//! Drawing for item weapons and item UI: auras, mines, blades, meteors,
//! beams, the shield bubble, active cooldowns, the synergy banner, and
//! level-up card tags.

use super::weapons::{self, BANNER_TIME, BEAM_TIME, FALL_TIME, WeaponKind};
use super::{Build, On};
use crate::content::{self, Content};
use crate::game::world::World;
use crate::render::canvas::{Blit, Canvas};
use crate::render::font;
use crate::render::palette::{self, Color, INK};
use crate::render::sprite::{SpriteId, bank};

/// The two main colors of a sprite, for tinting beams and auras.
fn tint(sprite: SpriteId) -> (Color, Color) {
    let c = &bank().get(sprite).colors;
    let first = c.first().copied().map_or(palette::BONE, palette::color);
    let second = c.get(1).copied().map_or(first, palette::color);
    (first, second)
}

/// Ground-level item visuals, drawn under creatures: auras and mines.
pub fn under(cv: &mut Canvas, w: &World) {
    let cam = &w.camera;
    let (px, py) = cam.to_screen(w.player.pos);
    for (k, a) in w.gear.weapons.iter().enumerate() {
        let WeaponKind::Aura { radius, .. } = a.kind else { continue };
        let r = (radius * a.area * w.item_stats.get(super::Stat::Area)) as i32;
        let (c1, c2) = tint(a.sprite);
        let t = w.gear.pulse * 2.0 + k as f32;
        cv.blend_ellipse(px, py + 2, r, r * 3 / 4, c1, 34 + (t.sin() * 10.0) as u8);
        let dots = (r / 3).max(8);
        for d in 0..dots {
            let ang = t * 0.6 + d as f32 / dots as f32 * std::f32::consts::TAU;
            let x = px + (ang.cos() * r as f32) as i32;
            let y = py + 2 + (ang.sin() * r as f32 * 0.75) as i32;
            cv.put(x, y, if d % 2 == 0 { c2 } else { c1 });
        }
    }
    for m in &w.gear.mines {
        let (x, y) = cam.to_screen(m.pos);
        let s = bank().get(m.sprite);
        let f = if m.arm > 0.0 { s.first() } else { s.frame_at(w.gear.pulse, 4.0) };
        let alpha = if m.arm > 0.0 { 170 } else { 255 };
        cv.blit_centered(f, x, y, Blit { alpha, ..Blit::default() });
    }
}

/// Item visuals drawn over creatures: blades, beams, meteors, shield.
pub fn over(cv: &mut Canvas, w: &World) {
    let cam = &w.camera;
    let (px, py) = cam.to_screen(w.player.pos);
    for (k, a) in w.gear.weapons.iter().enumerate() {
        if !matches!(a.kind, WeaponKind::Orbit { .. }) {
            continue;
        }
        let f = bank().get(a.sprite).frame_at(w.gear.pulse, 12.0);
        for pos in weapons::blades(w, k) {
            let (x, y) = cam.to_screen(pos);
            cv.blit_centered(f, x, y, Blit::default());
        }
    }

    for b in &w.gear.beams {
        let (c1, c2) = tint(b.sprite);
        let fade = b.life / BEAM_TIME;
        let half = ((b.width / 2.0 * fade).round() as i32).max(0);
        let (ex, ey) = (px + (b.dir.x * b.len) as i32, py + (b.dir.y * b.len) as i32);
        let (nx, ny) = (-b.dir.y, b.dir.x);
        for o in -half - 1..=half + 1 {
            let (dx, dy) = ((nx * o as f32).round() as i32, (ny * o as f32).round() as i32);
            let c = if o.abs() > half {
                INK
            } else if o.abs() * 2 > half {
                c1
            } else {
                c2
            };
            cv.line(px + dx, py + dy, ex + dx, ey + dy, c);
        }
        cv.line(px, py, ex, ey, palette::CREAM);
        cv.fill_circle(ex, ey, half + 1, c2);
    }

    for m in &w.gear.meteors {
        let (x, y) = cam.to_screen(m.pos);
        let t = (m.fall / FALL_TIME).clamp(0.0, 1.5);
        let r = m.radius as i32;
        cv.blend_ellipse(x, y, ((1.0 - t.min(1.0)) * r as f32) as i32 + 2, 2, INK, 140);
        cv.circle(x, y, r, palette::BLOOD);
        if (m.fall * 16.0) as i32 % 2 == 0 {
            cv.circle(x, y, r - 1, palette::RED);
        }
        let (mx, my) = (x + (t * 50.0) as i32, y - (t * 130.0) as i32);
        for k in 1..4 {
            let (tx, ty) = (mx + k * 4, my - k * 10);
            cv.fill_circle(tx, ty, 3 - k, if k == 1 { palette::GOLD } else { palette::EMBER });
        }
        cv.blit_centered(bank().get(m.sprite).frame_at(m.fall, 10.0), mx, my, Blit::default());
    }

    if w.gear.shield > 0.0 && (w.gear.shield > 0.6 || (w.gear.shield * 10.0) as i32 % 2 == 0) {
        cv.circle(px, py, 11, palette::SKY);
        cv.circle(px, py, 12, palette::NAVY);
        cv.put(px - 6, py - 7, palette::ICE);
        cv.put(px - 7, py - 6, palette::ICE);
    }
}

/// Active item cooldowns (bottom left, above the dash strip) and the synergy
/// banner, which drops below the boss bar and stage banner when they show.
pub fn hud(cv: &mut Canvas, w: &World, clock: f32) {
    let content = content::get();
    let mut shown: Vec<usize> = Vec::new();
    let y = cv.h - 33;
    for t in w.triggers.iter().filter(|t| t.def.on == On::Active) {
        let Some(item) = t.def.owner else { continue };
        if shown.contains(&item) {
            continue;
        }
        let x = 3 + shown.len() as i32 * 21;
        shown.push(item);
        let icon = bank().get(content.items[item].sprite_id).first();
        let ready = t.cd <= 0.0;
        cv.fill_rect(x, y, 19, 19, INK);
        cv.rect(x, y, 19, 19, if ready { palette::GOLD } else { palette::SLATE });
        cv.blit(icon, x + 10 - icon.w / 2, y + 10 - icon.h / 2, Blit::default());
        if !ready {
            let frac = (t.cd / t.def.cooldown.max(0.01)).clamp(0.0, 1.0);
            let h = (17.0 * frac).ceil() as i32;
            for yy in y + 1..y + 1 + h {
                for xx in x + 1..x + 18 {
                    cv.blend(xx, yy, INK, 170);
                }
            }
            let secs = (t.cd.ceil() as i32).to_string();
            font::draw_centered(cv, x + 10, y + 7, &secs, palette::FOG, INK);
        } else if (clock * 2.0) as i32 % 2 == 0 {
            cv.rect(x - 1, y - 1, 21, 21, palette::AMBER);
        }
    }
    if !shown.is_empty() {
        font::draw_outlined(cv, 3, y - 8, "SPACE", palette::HAZE, INK);
    }

    if let Some((k, t)) = w.gear.banner {
        let s = &content.synergies[k];
        let age = BANNER_TIME - t;
        let drop = ((0.18 - age).max(0.0) * 60.0) as i32;
        let cx = cv.w / 2;
        let boss = w.enemies.iter().any(|e| !e.dead && content.enemies[e.kind].boss().is_some());
        let base = if w.director.banner.is_some() {
            (cv.h as f32 * 0.24) as i32 + 16
        } else if boss {
            33
        } else {
            22
        };
        let top = base - drop;
        font::draw_centered(cv, cx, top, "COMBO", palette::PINK, INK);
        let color = if (age * 8.0) as i32 % 2 == 0 && age < 0.6 { palette::CREAM } else { palette::GOLD };
        font::draw_big_centered(cv, cx, top + 8, &s.name, color, INK, 2);
        font::draw_centered(cv, cx, top + 22, &s.desc, palette::BONE, INK);
    }
}

/// The corner tag on a level-up card: COMBO, NEW, or the next stack count.
pub fn card_tag(cv: &mut Canvas, content: &Content, build: &Build, item: usize, x: i32, y: i32, card_w: i32) {
    let stacks = build.stacks(item);
    let (tag, color) = if build.completes(content, item).is_some() {
        ("COMBO".to_string(), palette::PINK)
    } else if stacks == 0 {
        ("NEW".to_string(), palette::LIME)
    } else {
        (format!("X{}", stacks + 1), palette::GOLD)
    };
    font::draw_outlined(cv, x + card_w - 3 - font::width(&tag), y + 4, &tag, color, INK);
}
