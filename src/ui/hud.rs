//! In-run HUD: hearts and kills on top, timer centered, and a bottom strip
//! with the dash cooldown, level and XP bar.

use crate::game::clock_text;
use crate::game::world::World;
use crate::items::Stat;
use crate::render::canvas::{Blit, Canvas};
use crate::render::font;
use crate::render::fx::READY_TIME;
use crate::render::palette::{self, INK};
use crate::render::sprite::bank;

pub fn draw(cv: &mut Canvas, w: &World, clock: f32) {
    hearts(cv, w, clock);

    let t = clock_text(w.time);
    font::draw_big_centered(cv, cv.w / 2, 3, &t, palette::BONE, INK, 2);

    let kills = w.kills.to_string();
    let kx = cv.w - 4 - font::width(&kills);
    font::draw_outlined(cv, kx, 4, &kills, palette::FOG, INK);
    cv.blit(bank().named("skull").first(), kx - 10, 3, Blit::default());

    bottom(cv, w, clock);
}

fn hearts(cv: &mut Canvas, w: &World, clock: f32) {
    let bank = bank();
    let (full, half, empty) = (bank.named("heart_full"), bank.named("heart_half"), bank.named("heart_empty"));
    let max = w.max_hp();
    let low = w.player.hp <= 2;
    for i in 0..(max + 1) / 2 {
        let hp = w.player.hp - i * 2;
        let s = match hp {
            2.. => full,
            1 => half,
            _ => empty,
        };
        // Last hearts beat when low; a fresh hit flashes them white.
        let beat = low && hp > 0 && (clock * 3.0).fract() < 0.18;
        let y = if beat { 2 } else { 3 };
        let flash = (w.player.hurt > 0.0 && hp > 0).then_some(palette::BONE);
        cv.blit(s.first(), 3 + i * 8, y, Blit { flash, ..Blit::default() });
    }
}

fn bottom(cv: &mut Canvas, w: &World, clock: f32) {
    let y0 = cv.h - 12;

    // Dash cooldown: the icon fills from the bottom and brightens when ready,
    // pulsing for a moment as it comes back.
    let icon = bank().named("dash_icon").first();
    let (ix, iy) = (3, y0);
    let p = &w.player;
    let max = w.stats.get(Stat::DashCooldown);
    let ready = p.dash_cd <= 0.0 || max <= 0.0;
    let pulse = w.fx.hero.ready / READY_TIME;
    cv.fill_rect(ix, iy, 11, 11, INK);
    if ready && pulse > 0.0 {
        let (edge, glow, fill) = if pulse > 0.5 {
            (palette::ICE, palette::CYAN, palette::BONE)
        } else {
            (palette::CYAN, palette::BLUE, palette::ICE)
        };
        cv.rect(ix - 1, iy - 1, 13, 13, glow);
        cv.rect(ix, iy, 11, 11, edge);
        cv.blit(icon, ix + 1, iy + 1, Blit { flash: Some(fill), ..Blit::default() });
    } else if ready {
        cv.rect(ix, iy, 11, 11, palette::CYAN);
        cv.blit(icon, ix + 1, iy + 1, Blit::default());
    } else {
        let frac = 1.0 - (p.dash_cd / max).clamp(0.0, 1.0);
        let h = (frac * 9.0).round() as i32;
        cv.rect(ix, iy, 11, 11, palette::SLATE);
        cv.fill_rect(ix + 1, iy + 10 - h, 9, h, palette::NAVY);
        cv.blit(icon, ix + 1, iy + 1, Blit { flash: Some(palette::MAUVE), ..Blit::default() });
    }

    // Level plate.
    let lv = format!("LV{}", p.level);
    let lx = ix + 14;
    font::draw_outlined(cv, lx, y0 + 3, &lv, palette::GOLD, INK);

    // XP bar with quarter ticks and a traveling glint.
    let (x0, x1) = (lx + font::width(&lv) + 4, cv.w - 4);
    let (by, bw) = (y0 + 3, x1 - x0);
    let frac = (p.xp / p.xp_next).clamp(0.0, 1.0);
    let fill = (frac * (bw - 2) as f32) as i32;
    cv.fill_rect(x0, by, bw, 5, INK);
    cv.fill_rect(x0 + 1, by + 1, bw - 2, 3, palette::NIGHT);
    for q in 1..4 {
        cv.vline(x0 + 1 + (bw - 2) * q / 4, by + 1, by + 3, palette::DUSK);
    }
    if fill > 0 {
        cv.fill_rect(x0 + 1, by + 1, fill, 3, palette::SKY);
        cv.hline(x0 + 1, x0 + fill, by + 1, palette::CYAN);
        cv.hline(x0 + 1, x0 + fill, by + 3, palette::BLUE);
        // Fresh XP lights up the head of the bar.
        if w.fx.hero.xp > 0.0 {
            let head = (x0 + fill - 8).max(x0 + 1);
            cv.fill_rect(head, by + 1, x0 + fill - head + 1, 3, palette::CYAN);
            cv.hline(head, x0 + fill, by + 1, palette::ICE);
        }
        let glint = x0 + 1 + ((clock * 70.0) as i32).rem_euclid(bw + 40) - 20;
        for k in 0..3 {
            let gx = glint + k;
            if gx > x0 && gx <= x0 + fill {
                cv.vline(gx, by + 1, by + 2, palette::ICE);
            }
        }
        cv.vline(x0 + fill, by + 1, by + 3, palette::ICE);
    }

    crate::items::draw::hud(cv, w, clock);
}
