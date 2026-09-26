//! Full-screen menus and overlays.

use crate::content;
use crate::game::world::World;
use crate::game::{Game, Offer, Summary, clock_text};
use crate::items::Rarity;
use crate::render::arena::PAD;
use crate::render::canvas::{Blit, Canvas};
use crate::render::font;
use crate::render::palette::{self, Color, INK};
use crate::render::sprite::bank;

fn blink(clock: f32, hz: f32) -> bool {
    (clock * hz) as i32 % 2 == 0
}

pub fn title(cv: &mut Canvas, g: &Game) {
    let bank = bank();
    match &g.floor {
        Some(f) => {
            let sx = ((g.clock * 0.08).sin() * 110.0) as i32 + f.w / 2 - cv.w / 2;
            let sy = ((g.clock * 0.05).cos() * 50.0) as i32 + f.h / 2 - cv.h / 2 - PAD / 2;
            cv.copy_view(f, sx, sy, INK);
        }
        None => cv.clear(palette::DUSK),
    }
    cv.wash(INK, 110);

    let cx = cv.w / 2;
    let logo = bank.named("logo").first();
    let top = (cv.h / 2 - 62).max(4);
    cv.blit_scaled(logo, cx - logo.w, top, 2);

    let content = content::get();
    let ch = &content.characters[g.character];
    let rex = bank.get(ch.sprite_id);
    let bob = ((g.clock * 3.0).sin() * 1.5).round() as i32;
    let ry = top + logo.h * 2 + 6;
    cv.blend_ellipse(cx, ry + 32, 12, 3, INK, 140);
    let f = rex.frame_at(g.clock, 4.0);
    cv.blit_scaled(f, cx - f.w, ry + bob, 2);

    let y = ry + 38;
    font::draw_centered(cv, cx, y, &ch.name, palette::GOLD, INK);
    let mut y = y + 8;
    for line in font::wrap(&ch.desc, cv.w - 16) {
        font::draw_centered(cv, cx, y, &line, palette::HAZE, INK);
        y += font::LINE_H;
    }
    let y = y + 5;
    if blink(g.clock, 1.6) {
        font::draw_centered(cv, cx, y, "PRESS SPACE", palette::CREAM, INK);
    }

    let best = g.save.best.get(&ch.id).copied().unwrap_or(0.0);
    let line = format!("BEST {}", clock_text(best));
    let bones = g.save.bones.to_string();
    let total = font::width(&line) + 16 + 10 + font::width(&bones);
    let x = cx - total / 2;
    font::draw_outlined(cv, x, y + 12, &line, palette::FOG, INK);
    let bx = x + font::width(&line) + 16;
    cv.blit(bank.named("bone").first(), bx, y + 12, Blit::default());
    font::draw_outlined(cv, bx + 11, y + 12, &bones, palette::GOLD, INK);

    font::draw_centered(cv, cx, cv.h - 9, "WASD MOVE  SPACE DASH  P PAUSE  Q QUIT", palette::HAZE, INK);
}

pub fn paused(cv: &mut Canvas, clock: f32) {
    cv.wash(INK, 150);
    let (cx, cy) = (cv.w / 2, cv.h / 2);
    font::draw_big_centered(cv, cx, cy - 14, "PAUSED", palette::BONE, INK, 2);
    if blink(clock, 1.2) {
        font::draw_centered(cv, cx, cy + 4, "P RESUME   Q QUIT", palette::HAZE, INK);
    } else {
        font::draw_centered(cv, cx, cy + 4, "P RESUME   Q QUIT", palette::FOG, INK);
    }
}

fn rarity_color(r: Rarity) -> Color {
    match r {
        Rarity::Common => palette::HAZE,
        Rarity::Rare => palette::SKY,
        Rarity::Epic => palette::GOLD,
    }
}

pub fn level_up(cv: &mut Canvas, w: &World, o: &Offer, clock: f32) {
    let content = content::get();
    let bank = bank();
    cv.wash(INK, 160);
    let cx = cv.w / 2;
    let n = o.items.len() as i32;
    let gap = 6;
    let card_w = ((cv.w - 12 - gap * (n - 1)) / n).min(74);
    let card_h = 84.min(cv.h - 52);
    let x0 = cx - (card_w * n + gap * (n - 1)) / 2;
    let top = ((cv.h - card_h - 44) / 2).max(8);
    let y0 = top + 20;
    font::draw_big_centered(cv, cx, top, "LEVEL UP", palette::GOLD, INK, 2);
    let pop = (o.age * 6.0).min(1.0);

    for (k, &item) in o.items.iter().enumerate() {
        let it = &content.items[item];
        let sel = k == o.cursor;
        let x = x0 + k as i32 * (card_w + gap);
        let y = y0 + if sel { -2 } else { 0 } + ((1.0 - pop) * 12.0) as i32;
        let border = if sel { palette::BONE } else { rarity_color(it.rarity) };

        cv.fill_rect(x + 1, y + 2, card_w, card_h, INK);
        cv.fill_rect(x, y, card_w, card_h, if sel { palette::DUSK } else { palette::NIGHT });
        cv.rect(x, y, card_w, card_h, border);
        cv.hline(x + 1, x + card_w - 2, y + 1, rarity_color(it.rarity).mix(palette::NIGHT, 120));

        let icon = bank.get(it.sprite_id).first();
        let ix = x + card_w / 2 - icon.w;
        cv.blend_ellipse(x + card_w / 2, y + 8 + icon.h * 2, icon.w - 2, 2, INK, 150);
        cv.blit_scaled(icon, ix, y + 6, 2);

        crate::items::draw::card_tag(cv, content, &w.build, item, x, y, card_w);

        let mut ty = y + 12 + icon.h * 2;
        for line in font::wrap(&it.name, card_w - 6) {
            font::draw_centered(
                cv,
                x + card_w / 2,
                ty,
                &line,
                if sel { palette::CREAM } else { palette::BONE },
                INK,
            );
            ty += font::LINE_H;
        }
        ty += 2;
        for line in font::wrap(&it.desc, card_w - 6) {
            if ty + font::GLYPH_H > y + card_h - 2 {
                break;
            }
            font::draw(cv, x + card_w / 2 - font::width(&line) / 2, ty, &line, palette::FOG);
            ty += font::LINE_H;
        }
        font::draw(cv, x + 3, y + card_h - 8, &(k + 1).to_string(), palette::MAUVE);
    }

    let x = x0 + o.cursor as i32 * (card_w + gap) + card_w / 2;
    let arrow = bank.named("cursor").first();
    let ay = y0 + card_h + 2 + if blink(clock, 3.0) { 0 } else { 1 };
    cv.blit(arrow, x - arrow.w / 2, ay, Blit::default());
    font::draw_centered(cv, cx, cv.h - 16, "A D SELECT   SPACE TAKE", palette::HAZE, INK);
}

pub fn dead(cv: &mut Canvas, s: &Summary, clock: f32) {
    let bank = bank();
    cv.wash(INK, 120);
    let cx = cv.w / 2;
    let y = (cv.h / 2 - 50).max(4);
    font::draw_big_centered(cv, cx, y, "EXTINCT", palette::RED, INK, 3);
    font::draw_big_centered(cv, cx, y + 24, &clock_text(s.time), palette::BONE, INK, 2);

    if s.reward.new_best {
        if blink(clock, 3.0) {
            font::draw_centered(cv, cx, y + 40, "NEW BEST", palette::GOLD, INK);
        }
    } else {
        font::draw_centered(
            cv,
            cx,
            y + 40,
            &format!("BEST {}", clock_text(s.reward.best)),
            palette::HAZE,
            INK,
        );
    }

    let bones = format!("+{}", s.reward.bones);
    let bw = 10 + font::width(&bones);
    cv.blit(bank.named("bone").first(), cx - bw / 2, y + 51, Blit::default());
    font::draw_outlined(cv, cx - bw / 2 + 11, y + 51, &bones, palette::GOLD, INK);

    let stats = format!("KILLS {}   LV {}", s.kills, s.level);
    font::draw_centered(cv, cx, y + 62, &stats, palette::FOG, INK);

    if s.age > 0.7 {
        font::draw_centered(cv, cx, cv.h - 16, "SPACE RETRY   ESC MENU", palette::HAZE, INK);
    }
}

pub fn quit_prompt(cv: &mut Canvas) {
    let (cx, cy) = (cv.w / 2, cv.h / 2);
    let (w, h) = (96, 34);
    cv.wash(INK, 90);
    cv.fill_rect(cx - w / 2 + 1, cy - h / 2 + 2, w, h, INK);
    cv.fill_rect(cx - w / 2, cy - h / 2, w, h, palette::NIGHT);
    cv.rect(cx - w / 2, cy - h / 2, w, h, palette::RED);
    font::draw_big_centered(cv, cx, cy - 11, "QUIT?", palette::BONE, INK, 2);
    font::draw_centered(cv, cx, cy + 5, "Y YES   N NO", palette::FOG, INK);
}
