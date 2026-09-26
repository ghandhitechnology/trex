//! Full-screen menus and overlays.

use crate::content;
use crate::game::world::World;
use crate::game::{Game, Offer, Summary, clock_text};
use crate::items::Rarity;
use crate::render::arena::PAD;
use crate::render::canvas::{Blit, Canvas};
use crate::render::font::{self, TitleStyle};
use crate::render::light;
use crate::render::palette::{self, CLEAR, Color, INK};
use crate::render::sprite::bank;

fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Overshoots a little before settling.
fn ease_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) - 1.0;
    1.0 + 2.7 * t * t * t + 1.7 * t * t
}

/// Soft pulse between two colors.
fn pulse(a: Color, b: Color, clock: f32, hz: f32) -> Color {
    a.mix(b, (((clock * hz * std::f32::consts::TAU).sin() * 0.5 + 0.5) * 255.0) as u8)
}

/// A framed box with a drop shadow and a lit top edge.
pub fn panel(cv: &mut Canvas, x: i32, y: i32, w: i32, h: i32, body: Color, border: Color) {
    cv.fill_rect(x + 1, y + 2, w, h, INK);
    cv.fill_rect(x, y, w, h, body);
    cv.rect(x, y, w, h, border);
    cv.hline(x + 1, x + w - 2, y + 1, body.mix(border, 70));
}

pub fn title(cv: &mut Canvas, g: &Game) {
    let bank = bank();
    match g.floors.as_ref().map(|f| f.menu()) {
        Some(f) => {
            let sx = ((g.clock * 0.08).sin() * 110.0) as i32 + f.cv.w / 2 - cv.w / 2;
            let sy = ((g.clock * 0.05).cos() * 50.0) as i32 + f.cv.h / 2 - cv.h / 2 - PAD / 2;
            cv.copy_view(&f.cv, sx, sy, INK);
            f.ambient(cv, (sx - PAD, sy - PAD), g.clock);
            cv.wash(INK, 90);
            light::vignette(cv, (cv.w / 2, cv.h / 2 - 10), f.biome.shade(), 0.9, 200);
        }
        None => cv.clear(palette::DUSK),
    }

    let cx = cv.w / 2;
    let logo = bank.named("logo").first();
    let top = (cv.h / 2 - 62).max(4);
    let drop = ((1.0 - ease_back(g.clock / 0.8)) * -50.0) as i32;
    wavy_logo(cv, logo, cx - logo.w, top + drop, g.clock);

    let content = content::get();
    let ch = &content.characters[g.character];
    let rex = bank.get(ch.sprite_id);
    let bob = ((g.clock * 3.0).sin() * 1.5).round() as i32;
    let ry = top + logo.h * 2 + 6;
    cv.blend_ellipse(cx, ry + 30, 12 - bob.abs(), 3, INK, 150);
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
    let press = pulse(palette::CREAM, palette::AMBER, g.clock, 0.8);
    font::draw_centered(cv, cx, y, "PRESS SPACE", press, INK);

    let best = g.save.best.get(&ch.id).copied().unwrap_or(0.0);
    let line = format!("BEST {}", clock_text(best));
    let bones = g.save.bones.to_string();
    let total = font::width(&line) + 16 + 10 + font::width(&bones);
    let x = cx - total / 2;
    font::draw_outlined(cv, x, y + 12, &line, palette::FOG, INK);
    let bx = x + font::width(&line) + 16;
    cv.blit(bank.named("bone").first(), bx, y + 12, Blit::default());
    font::draw_outlined(cv, bx + 11, y + 12, &bones, palette::GOLD, INK);

    font::draw_centered(cv, cx, cv.h - 9, "WASD MOVE  SPACE DASH  P PAUSE  Q QUIT", palette::MAUVE, INK);
}

/// The logo at 2x: each column sways, a highlight sweeps across it, and a
/// dark extrusion sits underneath.
fn wavy_logo(cv: &mut Canvas, f: &crate::render::sprite::Frame, x: i32, y: i32, t: f32) {
    let k = 2;
    let sweep = ((t * 0.35).fract() * 2.2 - 0.4) * (f.w + f.h) as f32;
    let wave = |sx: i32| ((t * 2.2 + sx as f32 * 0.16).sin() * 1.4).round() as i32;
    for pass in 0..2 {
        for sy in 0..f.h {
            for sx in 0..f.w {
                let idx = f.px[(sy * f.w + sx) as usize];
                if idx == CLEAR {
                    continue;
                }
                let (px, py) = (x + sx * k, y + sy * k + wave(sx));
                if pass == 0 {
                    cv.fill_rect(px, py + 3, k, k, palette::MAROON.mix(INK, 120));
                    continue;
                }
                let mut c = palette::color(idx);
                let d = (sx + sy) as f32 - sweep;
                if idx != 0 && (-3.0..0.0).contains(&d) {
                    c = c.mix(palette::CREAM, 170);
                }
                cv.fill_rect(px, py, k, k, c);
            }
        }
    }
}

pub fn paused(cv: &mut Canvas, w: &World, clock: f32) {
    const PITCH: i32 = 18;
    let content = content::get();
    let bank = bank();
    cv.wash(INK, 140);
    let (cx, cy) = (cv.w / 2, cv.h / 2);
    let items = &w.build.items;
    let per_row = ((cv.w - 24) / PITCH).max(1) as usize;
    let rows = items.len().div_ceil(per_row) as i32;
    let pw = 120.max(items.len().min(per_row) as i32 * PITCH + 12);
    // Active combos, comma separated, never splitting a name across lines.
    let mut combo_lines: Vec<String> = Vec::new();
    for k in w.build.synergies(content) {
        let name = &content.synergies[k].name;
        match combo_lines.last_mut() {
            Some(line) if font::width(&format!("{line}, {name}")) <= pw - 12 => {
                *line = format!("{line}, {name}")
            }
            _ => combo_lines.push(name.clone()),
        }
    }
    let ph = 44 + rows * PITCH + combo_lines.len() as i32 * font::LINE_H;
    let (px, py) = (cx - pw / 2, cy - ph / 2);
    panel(cv, px, py, pw, ph, palette::NIGHT, palette::SLATE);
    let st = TitleStyle { top: palette::BONE, bottom: palette::FOG, outline: INK, k: 2 };
    font::draw_title(cv, cx, py + 7, "PAUSED", st, |_| 0);

    for (row, chunk) in items.chunks(per_row).enumerate() {
        let n = chunk.len() as i32;
        let x0 = cx - n * PITCH / 2;
        let y = py + 24 + row as i32 * PITCH;
        for (k, &(item, stacks)) in chunk.iter().enumerate() {
            let icon = bank.get(content.items[item].sprite_id).first();
            let (x, y) = (x0 + k as i32 * PITCH + PITCH / 2, y + PITCH / 2);
            cv.blit_centered(icon, x, y, Blit::default());
            if stacks > 1 {
                let n = stacks.to_string();
                font::draw_outlined(cv, x + 8 - font::width(&n), y + 3, &n, palette::GOLD, INK);
            }
        }
    }
    let mut y = py + 26 + rows * PITCH;
    for line in &combo_lines {
        font::draw_centered(cv, cx, y, line, palette::PINK, INK);
        y += font::LINE_H;
    }
    let hint = pulse(palette::HAZE, palette::FOG, clock, 0.6);
    font::draw_centered(cv, cx, py + ph - 10, "P RESUME   Q QUIT", hint, INK);
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
    cv.wash(INK, (ease_out(o.age / 0.15) * 165.0) as u8);
    if o.age < 0.06 {
        cv.wash(palette::CREAM, 70);
    }
    let cx = cv.w / 2;
    let n = o.items.len() as i32;
    let gap = 6;
    let card_w = ((cv.w - 12 - gap * (n - 1)) / n).min(74);
    let card_h = 84.min(cv.h - 52);
    let x0 = cx - (card_w * n + gap * (n - 1)) / 2;
    let top = ((cv.h - card_h - 44) / 2).max(8);
    let y0 = top + 20;

    let st = TitleStyle { top: palette::CREAM, bottom: palette::GOLD, outline: INK, k: 2 };
    font::draw_title(cv, cx, top, "LEVEL UP", st, |i| {
        let t = o.age * 9.0 - i as f32 * 0.7;
        if (0.0..std::f32::consts::PI).contains(&t) { -(t.sin() * 3.0).round() as i32 } else { 0 }
    });

    for (k, &item) in o.items.iter().enumerate() {
        let it = &content.items[item];
        let appear = ease_back((o.age - k as f32 * 0.06) / 0.25);
        if appear <= 0.0 {
            continue;
        }
        let sel = k == o.cursor;
        let rc = rarity_color(it.rarity);
        let x = x0 + k as i32 * (card_w + gap);
        let lift = if sel { -3 + ((clock * 4.0).sin() * 0.8).round() as i32 } else { 0 };
        let y = y0 + lift + ((1.0 - appear) * 28.0) as i32;
        let border = if sel { palette::BONE } else { rc.mix(palette::NIGHT, 60) };

        panel(cv, x, y, card_w, card_h, if sel { palette::DUSK } else { palette::NIGHT }, border);
        cv.fill_rect(x + 1, y + 1, card_w - 2, 2, rc.mix(palette::NIGHT, if sel { 40 } else { 110 }));

        let icon = bank.get(it.sprite_id).first();
        let (icx, icy) = (x + card_w / 2, y + 7 + icon.h);
        cv.blend_ellipse(icx, icy, icon.w + 1, icon.h, rc, if sel { 46 } else { 22 });
        cv.blend_ellipse(icx, y + 8 + icon.h * 2, icon.w - 2, 2, INK, 150);
        let bob = if sel { ((clock * 3.0).sin() * 1.2).round() as i32 } else { 0 };
        cv.blit_scaled(icon, icx - icon.w, y + 6 + bob, 2);

        crate::items::draw::card_tag(cv, content, &w.build, item, x, y, card_w);

        let mut ty = y + 12 + icon.h * 2;
        for line in font::wrap(&it.name, card_w - 6) {
            let c = if sel { palette::CREAM } else { palette::BONE };
            font::draw_centered(cv, x + card_w / 2, ty, &line, c, INK);
            ty += font::LINE_H;
        }
        ty += 2;
        for line in font::wrap(&it.desc, card_w - 6) {
            if ty + font::GLYPH_H > y + card_h - 2 {
                break;
            }
            let c = if sel { palette::FOG } else { palette::HAZE };
            font::draw(cv, x + card_w / 2 - font::width(&line) / 2, ty, &line, c);
            ty += font::LINE_H;
        }
        font::draw(cv, x + 3, y + card_h - 7, &(k + 1).to_string(), palette::MAUVE);
    }

    if o.age > 0.2 {
        let x = x0 + o.cursor as i32 * (card_w + gap) + card_w / 2;
        let arrow = bank.named("cursor").first();
        let ay = y0 + card_h + 2 + if (clock * 3.0).fract() < 0.5 { 0 } else { 1 };
        cv.blit(arrow, x - arrow.w / 2, ay, Blit::default());
    }
    font::draw_centered(cv, cx, cv.h - 16, "A D SELECT   SPACE TAKE", palette::HAZE, INK);
}

pub fn dead(cv: &mut Canvas, w: &World, s: &Summary, clock: f32) {
    let bank = bank();
    let a = s.age;
    let cx = cv.w / 2;
    // Clear of the feat toast strip on top.
    let y = (cv.h / 2 - 53).max(16);

    // Title drops in and lands with a jolt.
    let drop = ((1.0 - ease_back(a / 0.35)) * -40.0) as i32;
    let jolt = if (0.3..0.42).contains(&a) { ((a * 90.0).sin() * 2.0) as i32 } else { 0 };
    let st = TitleStyle { top: palette::RED, bottom: palette::BLOOD, outline: INK, k: 3 };
    font::draw_title(cv, cx + jolt, y + drop, "EXTINCT", st, |_| 0);

    // The fossil.
    let ch = &content::get().characters[w.character];
    let f = bank.get(ch.sprite_id).first();
    let fy = y + 22;
    cv.blend_ellipse(cx, fy + f.h * 2 - 1, f.w - 2, 2, INK, 160);
    cv.blit_big(f, cx - f.w, fy, 2, Blit { flash: Some(palette::MAUVE), ..Blit::default() });

    // Time counts up.
    let count = ease_out((a - 0.35) / 0.6);
    let ty = fy + f.h * 2 + 5;
    font::draw_big_centered(cv, cx, ty, &clock_text(s.time * count), palette::BONE, INK, 2);

    let ly = ty + 16;
    if a > 0.9 {
        if s.reward.new_best {
            let c = pulse(palette::GOLD, palette::CREAM, clock, 1.5);
            font::draw_centered(cv, cx, ly, "NEW BEST", c, INK);
        } else {
            let best = format!("BEST {}", clock_text(s.reward.best));
            font::draw_centered(cv, cx, ly, &best, palette::HAZE, INK);
        }
    }

    if a > 1.0 {
        let earned = (s.reward.bones as f32 * ease_out((a - 1.0) / 0.5)).round() as u32;
        let bones = format!("+{earned}");
        let bw = 10 + font::width(&bones);
        cv.blit(bank.named("bone").first(), cx - bw / 2, ly + 10, Blit::default());
        font::draw_outlined(cv, cx - bw / 2 + 11, ly + 10, &bones, palette::GOLD, INK);
        let stats = format!("KILLS {}   LV {}", s.kills, s.level);
        font::draw_centered(cv, cx, ly + 20, &stats, palette::FOG, INK);
    }

    if a > 0.7 {
        font::draw_centered(cv, cx, cv.h - 10, "SPACE RETRY   ESC MENU", palette::HAZE, INK);
    }
}

pub fn quit_prompt(cv: &mut Canvas) {
    let (cx, cy) = (cv.w / 2, cv.h / 2);
    let (w, h) = (96, 34);
    cv.wash(INK, 90);
    panel(cv, cx - w / 2, cy - h / 2, w, h, palette::NIGHT, palette::RED);
    let st = TitleStyle { top: palette::BONE, bottom: palette::FOG, outline: INK, k: 2 };
    font::draw_title(cv, cx, cy - 11, "QUIT?", st, |_| 0);
    font::draw_centered(cv, cx, cy + 5, "Y YES   N NO", palette::FOG, INK);
}
