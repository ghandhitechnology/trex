//! Enemy rendering: bodies, elite auras, shields, telegraphs, the boss bar,
//! stage banners, and the stage ground palette.

use super::boss::{self, Move};
use super::director::Banner;
use super::{AiState, Behavior, Enemy};
use crate::content;
use crate::game::world::World;
use crate::render::canvas::{Blit, Canvas};
use crate::render::font;
use crate::render::palette::{self, CLEAR, Color, INK};
use crate::render::sprite::{Frame, bank};

/// Seconds the ground takes to shift to a new stage's colors.
const GROUND_FADE: f32 = 3.0;
/// How much of the original floor color survives the recolor (0-255).
const GROUND_KEEP: u8 = 70;

/// Recolor the floor to the current stage's ramp, fading from the last one.
pub fn ground(cv: &mut Canvas, w: &World) {
    let waves = &content::get().waves;
    let d = &w.director;
    let t = (d.stage_age / GROUND_FADE).min(1.0);
    let to = waves.stages[d.stage].ramp;
    let from = if t < 1.0 { waves.stages[d.prev_stage].ramp } else { to };
    if from.is_none() && to.is_none() {
        return;
    }
    let (lf, lt) = (from.map(lut), to.map(lut));
    let k = (t * 255.0) as u8;
    for p in &mut cv.px {
        let l = luma(*p) as usize;
        let b = lt.as_ref().map_or(*p, |m| m[l].mix(*p, GROUND_KEEP));
        *p = if k == 255 {
            b
        } else {
            let a = lf.as_ref().map_or(*p, |m| m[l].mix(*p, GROUND_KEEP));
            a.mix(b, k)
        };
    }
}

fn luma(c: Color) -> u8 {
    ((c.r as u32 * 77 + c.g as u32 * 150 + c.b as u32 * 29) >> 8) as u8
}

/// Brightness to ramp color: ink, dark, mid, light at rising luma stops.
fn lut(ramp: [Color; 3]) -> [Color; 256] {
    let stops = [(0.0, INK), (24.0, ramp[0]), (48.0, ramp[1]), (120.0, ramp[2])];
    std::array::from_fn(|l| {
        let l = l as f32;
        let i = stops.iter().rposition(|s| s.0 <= l).unwrap_or(0).min(stops.len() - 2);
        let (a, b) = (stops[i], stops[i + 1]);
        let t = ((l - a.0) / (b.0 - a.0)).clamp(0.0, 1.0);
        a.1.mix(b.1, (t * 255.0) as u8)
    })
}

fn blinking(timer: f32, rate: f32) -> bool {
    (timer * rate) as i32 % 2 == 0
}

/// Telegraphs on the ground, drawn under every creature.
pub fn under(cv: &mut Canvas, w: &World) {
    let content = content::get();
    let cam = &w.camera;
    for e in w.enemies.iter().filter(|e| !e.dead) {
        let def = &content.enemies[e.kind];
        let (x, y) = cam.to_screen(e.pos);
        let windup = e.state == AiState::Windup;
        match def.behavior {
            Behavior::Charge { speed, time, .. } if windup => {
                let end = e.pos + e.dir * speed * time;
                let (ex, ey) = cam.to_screen(end);
                dashed(cv, x, y, ex, ey, palette::RED, blinking(e.timer, 16.0));
            }
            Behavior::Orbit { .. } if windup => {
                let (ex, ey) = cam.to_screen(e.pos + e.dir * 40.0);
                dashed(cv, x, y, ex, ey, palette::PINK, true);
            }
            Behavior::Kamikaze { radius, fuse, .. } if e.state != AiState::Move => {
                zone(
                    cv,
                    x,
                    y,
                    radius,
                    1.0 - e.timer / fuse,
                    blinking(e.timer, 6.0 + 30.0 * (1.0 - e.timer / fuse)),
                );
            }
            Behavior::Slam { radius, windup: total, .. } if windup => {
                zone(cv, x, y, radius, 1.0 - e.timer / total, true);
            }
            Behavior::Blink { .. } if windup => marker(cv, w, e, 0.6),
            Behavior::Burrow { .. } if windup => {
                zone(cv, x, y, 10.0, 1.0 - e.timer / 0.6, blinking(e.timer, 12.0))
            }
            Behavior::Boss(_) if windup => match e.mv {
                Move::Charge { .. } => {
                    let (ex, ey) = cam.to_screen(e.pos + e.dir * 320.0);
                    let locked = e.timer <= boss::CHARGE_LOCK;
                    let c = if locked { palette::RED } else { palette::BLOOD };
                    let perp = e.dir.perp() * 5.0;
                    for side in [-1.0, 1.0] {
                        let (ax, ay) = cam.to_screen(e.pos + perp * side);
                        let (bx, by) = cam.to_screen(e.pos + perp * side + e.dir * 320.0);
                        dashed(cv, ax, ay, bx, by, c, locked || blinking(e.timer, 10.0));
                    }
                    if locked {
                        cv.line(x, y, ex, ey, palette::EMBER);
                    }
                }
                Move::Snipe { .. } => {
                    let (ex, ey) = cam.to_screen(e.pos + e.dir * 240.0);
                    if e.timer <= boss::SNIPE_LOCK {
                        cv.line(x, y, ex, ey, palette::RED);
                    } else if blinking(e.timer, 14.0) {
                        dashed(cv, x, y, ex, ey, palette::BLOOD, true);
                    }
                }
                Move::Slam => zone(cv, x, y, boss::SLAM_RADIUS, 1.0 - e.timer, true),
                Move::Erupt if e.timer <= boss::ERUPT_LOCK => {
                    zone(cv, x, y, boss::ERUPT_RADIUS, 1.0 - e.timer / boss::ERUPT_LOCK, true)
                }
                Move::Teleport => marker(cv, w, e, 0.9),
                _ => {}
            },
            _ => {}
        }
    }
}

/// Faint copy of the enemy where it is about to reappear.
fn marker(cv: &mut Canvas, w: &World, e: &Enemy, total: f32) {
    let def = &content::get().enemies[e.kind];
    let (x, y) = w.camera.to_screen(e.aim);
    let t = 1.0 - (e.timer / total).clamp(0.0, 1.0);
    let f = bank().get(def.sprite_id).first();
    let alpha = (40.0 + 120.0 * t) as u8;
    cv.blend_ellipse(x, y + f.h / 2 - 2, f.w / 2, 2, palette::INK, 90);
    cv.blit_centered(f, x, y, Blit { flip_x: false, flash: Some(palette::ICE), alpha });
}

/// A danger circle that fills as `t` goes 0 to 1.
fn zone(cv: &mut Canvas, x: i32, y: i32, radius: f32, t: f32, on: bool) {
    let r = radius as i32;
    cv.blend_ellipse(x, y, r, r, palette::RED, 40);
    let inner = (radius * t.clamp(0.0, 1.0)) as i32;
    if inner > 0 {
        cv.blend_ellipse(x, y, inner, inner, palette::RED, 70);
    }
    if on {
        cv.circle(x, y, r, palette::RED);
    }
}

fn dashed(cv: &mut Canvas, x0: i32, y0: i32, x1: i32, y1: i32, c: Color, on: bool) {
    if !on {
        return;
    }
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let (mut x, mut y, mut err, mut n) = (x0, y0, dx + dy, 0);
    loop {
        if n % 6 < 4 {
            cv.put(x, y, c);
        }
        if (x == x1 && y == y1) || n > 600 {
            break;
        }
        n += 1;
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// One enemy at screen position (x, y).
pub fn body(cv: &mut Canvas, w: &World, i: usize, x: i32, y: i32) {
    let content = content::get();
    let bank = bank();
    let e = &w.enemies[i];
    let def = &content.enemies[e.kind];
    let windup = e.state == AiState::Windup;

    if e.hidden {
        let burrowed = matches!(def.behavior, Behavior::Burrow { .. }) || e.mv == Move::Erupt;
        if burrowed {
            let name = if def.boss().is_some() { "mound_big" } else { "mound" };
            let s = bank.named(name);
            let shake = if windup { ((e.anim * 40.0) as i32 % 3) - 1 } else { 0 };
            cv.blit_centered(s.frame_at(e.anim, 8.0), x + shake, y + 2, Blit::default());
        }
        return;
    }

    let s = bank.get(def.sprite_id);
    let f = s.frame_at(e.anim + e.phase, 6.0);
    let flip = if e.vel.x.abs() > 1.0 { e.vel.x < 0.0 } else { w.player.pos.x < e.pos.x };
    let flash = if e.flash > 0.0 {
        Some(if e.shield > 0.0 { palette::ICE } else { palette::BONE })
    } else if windup && blinking(e.timer, 20.0) {
        Some(palette::RED)
    } else {
        None
    };
    if let Some(el) = e.elite {
        let a = 170 + ((e.anim * 6.0).sin() * 85.0) as i32;
        aura(cv, f, x, y, flip, el.color(), a.clamp(0, 255) as u8);
    }
    cv.blit_centered(f, x, y, Blit { flip_x: flip, flash, alpha: 255 });
    if e.shield > 0.0 && e.shield_max > 0.0 {
        let r = def.radius as i32 + 4;
        let a = (60.0 + 140.0 * e.shield / e.shield_max) as u8;
        cv.blend_ellipse(x, y, r, r, palette::SKY, 28);
        ring(cv, x, y, r, palette::ICE, a);
        cv.put(x - r / 2, y - r / 2 - 1, palette::BONE);
    }
    let big = def.hp >= 40.0 || e.elite.is_some();
    if def.boss().is_none() && big && e.hp < e.max_hp {
        let bw = 12;
        let fill = ((e.hp / e.max_hp) * bw as f32).ceil() as i32;
        let by = y - f.h / 2 - 3;
        cv.fill_rect(x - bw / 2 - 1, by - 1, bw + 2, 3, INK);
        cv.fill_rect(x - bw / 2, by, fill, 1, palette::RED);
    }
}

/// Colored one-pixel rim just outside the sprite's ink outline.
fn aura(cv: &mut Canvas, f: &Frame, cx: i32, cy: i32, flip: bool, c: Color, a: u8) {
    let (x0, y0) = (cx - f.w / 2, cy - f.h / 2);
    let solid =
        |x: i32, y: i32| x >= 0 && y >= 0 && x < f.w && y < f.h && f.px[(y * f.w + x) as usize] != CLEAR;
    for sy in -1..=f.h {
        for sx in -1..=f.w {
            if !solid(sx, sy)
                && (solid(sx - 1, sy) || solid(sx + 1, sy) || solid(sx, sy - 1) || solid(sx, sy + 1))
            {
                let px = if flip { f.w - 1 - sx } else { sx };
                cv.blend(x0 + px, y0 + sy, c, a);
            }
        }
    }
}

fn ring(cv: &mut Canvas, cx: i32, cy: i32, r: i32, c: Color, a: u8) {
    let (outer, inner) = (r * r + r, (r - 1) * (r - 1) + (r - 1));
    for dy in -r..=r {
        for dx in -r..=r {
            let d = dx * dx + dy * dy;
            if d <= outer && d > inner {
                cv.blend(cx + dx, cy + dy, c, a);
            }
        }
    }
}

/// Boss health bar and banners, drawn over the world.
pub fn over(cv: &mut Canvas, w: &World) {
    let content = content::get();
    let boss = w.enemies.iter().find(|e| !e.dead && content.enemies[e.kind].boss().is_some());
    if let Some(e) = boss {
        let def = &content.enemies[e.kind];
        let bw = (cv.w - 90).clamp(60, 150);
        let (x0, y) = (cv.w / 2 - bw / 2, 25);
        font::draw_centered(cv, cv.w / 2, y - 7, &def.name, palette::BLUSH, INK);
        cv.fill_rect(x0 - 1, y - 1, bw + 2, 5, INK);
        cv.fill_rect(x0, y, bw, 3, palette::MAROON);
        let fill = ((e.hp / e.max_hp).clamp(0.0, 1.0) * bw as f32).ceil() as i32;
        if fill > 0 {
            cv.fill_rect(x0, y, fill, 3, palette::RED);
            cv.hline(x0, x0 + fill - 1, y, palette::EMBER);
        }
        for mark in [0.55, 0.25] {
            cv.vline(x0 + (bw as f32 * mark) as i32, y, y + 2, INK);
        }
    }
    if let Some(b) = &w.director.banner {
        let shown = Banner::TIME - b.time;
        let visible = shown > 0.1 && (b.time > 0.5 || blinking(b.time, 10.0));
        if visible {
            let y = (cv.h as f32 * 0.24) as i32;
            font::draw_big_centered(cv, cv.w / 2, y, &b.text, b.color, INK, 2);
        }
    }
}
