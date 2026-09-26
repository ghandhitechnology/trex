//! In-run HUD: hearts, timer, level, kills, XP bar.

use crate::game::clock_text;
use crate::game::world::World;
use crate::render::canvas::{Blit, Canvas};
use crate::render::font;
use crate::render::palette::{self, INK};
use crate::render::sprite::bank;

pub fn draw(cv: &mut Canvas, w: &World, clock: f32) {
    let bank = bank();

    // Hearts: two HP per heart.
    let (full, half, empty) = (bank.named("heart_full"), bank.named("heart_half"), bank.named("heart_empty"));
    let max = w.max_hp();
    let low = w.player.hp <= 2 && (clock * 4.0) as i32 % 2 == 0;
    for i in 0..(max + 1) / 2 {
        let hp = w.player.hp - i * 2;
        let s = match hp {
            2.. => full,
            1 => half,
            _ => empty,
        };
        let y = if low && hp > 0 { 2 } else { 3 };
        cv.blit(s.first(), 3 + i * 8, y, Blit::default());
    }

    // Timer.
    let t = clock_text(w.time);
    font::draw_big_centered(cv, cv.w / 2, 3, &t, palette::BONE, INK, 2);

    // Level and kills, right aligned.
    let lv = format!("LV {}", w.player.level);
    font::draw_outlined(cv, cv.w - 4 - font::width(&lv), 4, &lv, palette::GOLD, INK);
    let kills = w.kills.to_string();
    let kx = cv.w - 4 - font::width(&kills);
    font::draw_outlined(cv, kx, 13, &kills, palette::FOG, INK);
    cv.blit(bank.named("skull").first(), kx - 10, 12, Blit::default());

    // XP bar along the bottom edge.
    let (x0, x1, y) = (3, cv.w - 4, cv.h - 5);
    let fill = ((w.player.xp / w.player.xp_next).clamp(0.0, 1.0) * (x1 - x0 - 1) as f32) as i32;
    cv.fill_rect(x0, y, x1 - x0 + 1, 4, INK);
    cv.fill_rect(x0 + 1, y + 1, x1 - x0 - 1, 2, palette::NIGHT);
    if fill > 0 {
        cv.fill_rect(x0 + 1, y + 1, fill, 2, palette::SKY);
        cv.hline(x0 + 1, x0 + fill, y + 1, palette::CYAN);
    }

    crate::items::draw::hud(cv, w, clock);
}
