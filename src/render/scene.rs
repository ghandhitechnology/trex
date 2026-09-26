//! Draws a world: floor, pickups, creatures sorted by depth, shots, effects.

use super::arena::PAD;
use super::canvas::{Blit, Canvas};
use super::palette;
use super::sprite::bank;
use crate::content;
use crate::enemies::AiState;
use crate::enemies::director::WARN_TIME;
use crate::game::world::World;

enum Body {
    Enemy(usize),
    Player,
}

pub fn draw_world(cv: &mut Canvas, w: &World, floor: Option<&Canvas>, dead: bool) {
    let bank = bank();
    let content = content::get();
    let cam = &w.camera;
    let (ox, oy) = cam.origin();
    match floor {
        Some(f) => cv.copy_view(f, ox + PAD, oy + PAD, palette::INK),
        None => cv.clear(palette::DUSK),
    }

    let warn = bank.named("warn");
    for p in &w.director.pending {
        let (x, y) = cam.to_screen(p.pos);
        let t = WARN_TIME - p.timer;
        let f = warn.frame_at(t, 8.0);
        cv.blit_centered(f, x, y, Blit { alpha: 150 + (t / WARN_TIME * 105.0) as u8, ..Blit::default() });
    }

    let (gem, gem_big) = (bank.named("gem"), bank.named("gem_big"));
    for g in &w.gems {
        let (x, y) = cam.to_screen(g.pos);
        let bob = ((g.age * 5.0 + g.pos.x * 0.3).sin() * 1.5).round() as i32;
        let s = if g.value >= 5 { gem_big } else { gem };
        cv.blend_ellipse(x, y + 3, 2, 1, palette::INK, 90);
        cv.blit_centered(s.frame_at(g.age + g.pos.y * 0.05, 3.0), x, y - 1 + bob, Blit::default());
    }
    crate::items::draw::under(cv, w);

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
    if !dead {
        cv.blend_ellipse(px, py + 6, 5, 2, palette::INK, 120);
        bodies.push((py, Body::Player));
    }
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
                let flash = if e.flash > 0.0 {
                    Some(palette::BONE)
                } else if windup {
                    Some(palette::RED)
                } else {
                    None
                };
                let f = s.frame_at(e.anim + e.phase, 6.0);
                cv.blit_centered(f, x, y, Blit { flip_x: e.vel.x < -1.0, flash, alpha: 255 });
                if def.hp >= 40.0 && e.hp < e.max_hp {
                    let bw = 12;
                    let fill = ((e.hp / e.max_hp) * bw as f32).ceil() as i32;
                    let by = y - f.h / 2 - 3;
                    cv.fill_rect(x - bw / 2 - 1, by - 1, bw + 2, 3, palette::INK);
                    cv.fill_rect(x - bw / 2, by, fill, 1, palette::RED);
                }
            }
            Body::Player => {
                let p = &w.player;
                let blink = p.invuln > 0.0 && p.dash_time <= 0.0 && (p.invuln * 16.0) as i32 % 2 == 1;
                if !blink {
                    let s = bank.get(ch.sprite_id);
                    let moving = p.vel.len_sq() > 25.0;
                    let f = if moving { s.frame_at(p.anim * 1.5, 6.0) } else { s.first() };
                    let flash = (p.hurt > 0.0).then_some(palette::BONE);
                    cv.blit_centered(f, px, py, Blit { flip_x: p.facing < 0.0, flash, alpha: 255 });
                }
                let dash_max = w.stats.get(crate::items::Stat::DashCooldown);
                if p.dash_cd > 0.0 && dash_max > 0.0 {
                    let bw = 10;
                    let fill = ((1.0 - p.dash_cd / dash_max) * bw as f32) as i32;
                    cv.fill_rect(px - bw / 2 - 1, py + 10, bw + 2, 3, palette::INK);
                    cv.fill_rect(px - bw / 2, py + 11, fill, 1, palette::CYAN);
                }
            }
        }
    }

    for s in &w.shots {
        let (x, y) = cam.to_screen(s.pos);
        let spr = bank.get(s.sprite);
        if !s.hostile {
            let back = s.vel.norm() * 3.0;
            cv.put(x - back.x as i32, y - back.y as i32, palette::AMBER);
        }
        cv.blit_centered(spr.frame_at(s.age, 12.0), x, y, Blit::default());
    }
    crate::items::draw::over(cv, w);

    w.fx.draw_over(cv, cam);

    if w.fx.flash > 0.0 {
        cv.wash(palette::RED, (w.fx.flash / 0.08 * 70.0) as u8);
    }
    if dead {
        cv.wash(palette::MAROON, 90);
    }
}
