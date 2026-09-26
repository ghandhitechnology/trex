//! Drawing for the hub (heroes, shop, feats) and feat banners.

use super::defs::FeatReward;
use super::hub::{FLASH, Hub, Tab, shop_layout};
use super::save::Save;
use super::{Buy, TOAST_TIME, Toasts};
use crate::content::{self, Content};
use crate::game::clock_text;
use crate::items::{Build, Stat};
use crate::meta::characters::CharacterDef;
use crate::render::arena::PAD;
use crate::render::canvas::{Blit, Canvas};
use crate::render::font;
use crate::render::palette::{self, Color, INK};
use crate::render::sprite::{Frame, bank};

const CELL: i32 = 20;
const GAP: i32 = 3;
const TOP: i32 = 17;

fn blink(clock: f32, hz: f32) -> bool {
    (clock * hz) as i32 % 2 == 0
}

/// Shop grid columns that fit a canvas this wide.
pub fn shop_cols(w: i32) -> usize {
    ((w - 16 + GAP) / (CELL + GAP)).clamp(3, 12) as usize
}

/// Nearest-neighbor scaled frame, optionally flipped or filled with one color.
fn blit_big(cv: &mut Canvas, f: &Frame, x: i32, y: i32, k: i32, tint: Option<Color>) {
    for sy in 0..f.h {
        for sx in 0..f.w {
            let idx = f.px[(sy * f.w + sx) as usize];
            if idx == palette::CLEAR {
                continue;
            }
            let c = match tint {
                Some(t) if idx != 0 => t,
                _ => palette::color(idx),
            };
            cv.fill_rect(x + sx * k, y + sy * k, k, k, c);
        }
    }
}

fn panel(cv: &mut Canvas, x: i32, y: i32, w: i32, h: i32, border: Color) {
    cv.fill_rect(x + 1, y + 2, w, h, INK);
    cv.fill_rect(x, y, w, h, palette::NIGHT);
    cv.rect(x, y, w, h, border);
}

/// Bone icon plus a number; returns the drawn width.
fn bones_label(cv: &mut Canvas, x: i32, y: i32, n: u32, c: Color) -> i32 {
    let text = n.to_string();
    cv.blit(bank().named("bone").first(), x, y, Blit::default());
    font::draw_outlined(cv, x + 11, y, &text, c, INK);
    11 + font::width(&text)
}

fn bones_width(n: u32) -> i32 {
    11 + font::width(&n.to_string())
}

fn backdrop(cv: &mut Canvas, floor: Option<&Canvas>, clock: f32) {
    match floor {
        Some(f) => {
            let sx = ((clock * 0.06).sin() * 90.0) as i32 + f.w / 2 - cv.w / 2;
            let sy = ((clock * 0.04).cos() * 40.0) as i32 + f.h / 2 - cv.h / 2 - PAD / 2;
            cv.copy_view(f, sx, sy, INK);
        }
        None => cv.clear(palette::DUSK),
    }
    cv.wash(INK, 150);
}

pub fn hub(cv: &mut Canvas, h: &Hub, save: &Save, floor: Option<&Canvas>, clock: f32) {
    let content = content::get();
    backdrop(cv, floor, clock);
    header(cv, h, save, clock);
    match h.tab {
        Tab::Heroes => heroes(cv, h, save, content, clock),
        Tab::Shop => shop(cv, h, save, content, clock),
        Tab::Feats => feats(cv, h, save, content, clock),
    }
    let hint = if h.on_tabs {
        "A D TAB   S OPEN   ESC BACK"
    } else {
        match h.tab {
            Tab::Heroes => "A D PICK   W TABS   ESC BACK",
            Tab::Shop => "WASD MOVE   SPACE BUY   ESC BACK",
            Tab::Feats => "W S MOVE   ESC BACK",
        }
    };
    font::draw_centered(cv, cv.w / 2, cv.h - 8, hint, palette::MAUVE, INK);
}

fn header(cv: &mut Canvas, h: &Hub, save: &Save, clock: f32) {
    let spacing = 14;
    let total: i32 = Tab::ALL.iter().map(|t| font::width(t.label())).sum::<i32>() + spacing * 2;
    let mut x = cv.w / 2 - total / 2;
    for t in Tab::ALL {
        let label = t.label();
        let w = font::width(label);
        let sel = t == h.tab;
        let color = match (sel, h.on_tabs) {
            (true, true) if blink(clock, 3.0) => palette::CREAM,
            (true, _) => palette::GOLD,
            _ => palette::HAZE,
        };
        if sel && h.on_tabs {
            cv.fill_rect(x - 3, 2, w + 6, 9, palette::DUSK);
            cv.rect(x - 3, 2, w + 6, 9, palette::GOLD);
        }
        font::draw_outlined(cv, x, 4, label, color, INK);
        if sel && !h.on_tabs {
            cv.hline(x, x + w - 1, 11, palette::GOLD);
        }
        x += w + spacing;
    }
    let color = if h.deny > 0.0 && blink(h.deny, 12.0) {
        palette::RED
    } else if h.bought > 0.0 {
        palette::CREAM
    } else {
        palette::GOLD
    };
    bones_label(cv, cv.w - 5 - bones_width(save.bones), 4, save.bones, color);
    for x in 0..cv.w {
        cv.blend(x, 13, palette::SLATE, 150);
    }
}

/// Stats the hero starts a run with, before upgrades.
fn start_stats(ch: &CharacterDef, content: &Content) -> crate::items::Stats {
    Build::default().stats(ch, content)
}

fn power(s: &crate::items::Stats) -> f32 {
    let crit = s.get(Stat::Crit) * (s.get(Stat::CritDamage) - 1.0);
    s.get(Stat::Damage)
        * s.get(Stat::FireRate)
        * s.count(Stat::Shots) as f32
        * (1.0 + crit)
        * (1.0 + 0.5 * s.count(Stat::Pierce) as f32)
}

fn heroes(cv: &mut Canvas, h: &Hub, save: &Save, content: &Content, clock: f32) {
    let bank = bank();
    let cx = cv.w / 2;
    let n = content.characters.len() as i32;
    let x0 = cx - (n * (CELL + 2) - 2) / 2;
    let lock = bank.named("lock").first();

    for (i, ch) in content.characters.iter().enumerate() {
        let x = x0 + i as i32 * (CELL + 2);
        let sel = i == h.hero;
        let open = super::hero_unlocked(save, content, i);
        let y = TOP + if sel { -1 } else { 0 };
        let border = match (sel, h.on_tabs) {
            (true, false) => palette::GOLD,
            (true, true) => palette::HAZE,
            _ => palette::SLATE,
        };
        panel(cv, x, y, CELL, CELL, border);
        let f = if sel && open {
            bank.get(ch.idle_id.unwrap_or(ch.sprite_id)).frame_at(clock, 2.5)
        } else {
            bank.get(ch.sprite_id).first()
        };
        let tint = (!open).then_some(palette::SLATE);
        cv.blit(f, x + CELL / 2 - f.w / 2, y + CELL / 2 - f.h / 2, Blit { flash: tint, ..Blit::default() });
        if !open {
            cv.blit(lock, x + CELL - lock.w + 1, y + CELL - lock.h + 1, Blit::default());
        }
    }

    let i = h.hero;
    let ch = &content.characters[i];
    let open = super::hero_unlocked(save, content, i);
    let pw = (cv.w - 16).min(236);
    let px = cx - pw / 2;
    let py = TOP + CELL + 6;

    // Portrait: idle, then a few steps of the walk.
    let walking = (clock / 2.5) as i32 % 2 == 1;
    let f = if walking {
        bank.get(ch.sprite_id).frame_at(clock, 6.0)
    } else {
        bank.get(ch.idle_id.unwrap_or(ch.sprite_id)).frame_at(clock, 2.5)
    };
    cv.blend_ellipse(px + 20, py + 36, 13, 3, INK, 150);
    blit_big(cv, f, px + 20 - f.w, py + 4, 2, (!open).then_some(palette::SLATE));

    let tx = px + 46;
    let tw = pw - 46;
    font::draw_outlined(cv, tx, py, &ch.name.to_uppercase(), palette::GOLD, INK);
    if let Some(&best) = save.best.get(&ch.id).filter(|&&b| b > 0.0) {
        let s = format!("BEST {}", clock_text(best));
        font::draw_outlined(cv, px + pw - font::width(&s), py, &s, palette::FOG, INK);
    }
    let mut y = py + 9;
    for line in font::wrap(&ch.desc, tw).into_iter().take(2) {
        font::draw_outlined(cv, tx, y, &line, palette::HAZE, INK);
        y += font::LINE_H;
    }

    // Stats, compared against the whole roster.
    let all: Vec<_> = content.characters.iter().map(|c| start_stats(c, content)).collect();
    let s = &all[i];
    let max = |f: &dyn Fn(&crate::items::Stats) -> f32| all.iter().map(f).fold(1e-3, f32::max);
    let y = y.max(py + 23) + 3;
    let (full, half) = (bank.named("heart_full").first(), bank.named("heart_half").first());
    font::draw_outlined(cv, tx, y + 1, "HP", palette::FOG, INK);
    let hp = s.count(Stat::MaxHp) as i32;
    for k in 0..(hp + 1) / 2 {
        let heart = if hp - k * 2 >= 2 { full } else { half };
        cv.blit(heart, tx + 18 + k * 8, y, Blit::default());
    }
    let bars: [(&str, f32, f32); 3] = [
        ("SPD", s.get(Stat::Speed), max(&|s| s.get(Stat::Speed))),
        ("PWR", power(s), max(&power)),
        ("RNG", s.get(Stat::Range), max(&|s| s.get(Stat::Range))),
    ];
    let bw = (tw - 20).min(70);
    for (k, (label, v, m)) in bars.into_iter().enumerate() {
        let by = y + 10 + k as i32 * 7;
        font::draw_outlined(cv, tx, by, label, palette::FOG, INK);
        let fill = ((v / m).clamp(0.08, 1.0) * bw as f32).round() as i32;
        cv.fill_rect(tx + 17, by, bw + 2, 5, INK);
        cv.fill_rect(tx + 18, by + 1, bw, 3, palette::DUSK);
        cv.fill_rect(tx + 18, by + 1, fill, 3, palette::SKY);
        cv.hline(tx + 18, tx + 17 + fill, by + 1, palette::CYAN);
    }

    // Action.
    let ay = cv.h - 27;
    if open {
        if !h.on_tabs && blink(clock, 1.6) {
            font::draw_centered(cv, cx, ay, "SPACE PLAY", palette::CREAM, INK);
        }
    } else {
        let can = save.bones >= ch.unlock;
        let label = if can { "SPACE UNLOCK" } else { "NEED" };
        let w = font::width(label) + 6 + bones_width(ch.unlock);
        let x = cx - w / 2;
        let c = if can { palette::CREAM } else { palette::HAZE };
        font::draw_outlined(cv, x, ay, label, c, INK);
        let cost_c = if h.deny > 0.0 && blink(h.deny, 12.0) { palette::RED } else { palette::GOLD };
        bones_label(cv, x + font::width(label) + 6, ay, ch.unlock, cost_c);
        if let Some(f) = super::feat_for_unlock(content, &ch.id) {
            let feat = &content.meta.feats[f];
            let line = format!("OR {}", feat.goal.text(content).to_uppercase());
            font::draw_centered(cv, cx, ay + 9, &line, palette::MAUVE, INK);
        }
    }
}

fn shop(cv: &mut Canvas, h: &Hub, save: &Save, content: &Content, clock: f32) {
    let bank = bank();
    let entries = super::shop_entries(content);
    if entries.is_empty() {
        return;
    }
    let cols = shop_cols(cv.w);
    let cells = shop_layout(&entries, cols);
    let sel = h.shop.min(entries.len() - 1);
    let row_h = CELL + 7;
    let label_h = 9;

    // Row tops, with a section label above each section's first row.
    let rows = cells.last().map_or(0, |c| c.0 + 1);
    let mut row_y = vec![0; rows];
    let mut labels = Vec::new();
    let mut y = TOP;
    for (r, top) in row_y.iter_mut().enumerate() {
        let first = cells.iter().position(|c| c.0 == r).unwrap_or(0);
        if r == 0 || std::mem::discriminant(&entries[first]) != std::mem::discriminant(&entries[first - 1]) {
            let label = if matches!(entries[first], Buy::Upgrade(_)) { "UPGRADES" } else { "ITEMS" };
            labels.push((y, label));
            y += label_h;
        }
        *top = y;
        y += row_h;
    }
    let area_bottom = cv.h - 42;
    let sel_bottom = row_y[cells[sel].0] + row_h;
    let scroll = (sel_bottom - area_bottom).max(0);

    let gw = cols as i32 * (CELL + GAP) - GAP;
    let gx = cv.w / 2 - gw / 2;
    for (ly, label) in labels {
        let ly = ly - scroll;
        if ly >= TOP {
            font::draw_outlined(cv, gx, ly, label, palette::MAUVE, INK);
        }
    }
    let lock = bank.named("lock").first();
    for (i, (&b, &(r, c))) in entries.iter().zip(&cells).enumerate() {
        let (x, y) = (gx + c as i32 * (CELL + GAP), row_y[r] - scroll);
        if y < TOP || y + CELL > area_bottom + 6 {
            continue;
        }
        let is_sel = i == sel;
        let price = super::price(save, content, b);
        let border = match (is_sel, h.on_tabs) {
            (true, false) => palette::GOLD,
            (true, true) => palette::HAZE,
            _ if price.is_none() => palette::MAUVE,
            _ => palette::SLATE,
        };
        let y = y - i32::from(is_sel && !h.on_tabs);
        panel(cv, x, y, CELL, CELL, border);
        let (sprite, dim) = match b {
            Buy::Upgrade(u) => (content.meta.upgrades[u].sprite_id, false),
            Buy::Item(it) => (content.items[it].sprite_id, price.is_some()),
            Buy::Hero(ch) => (content.characters[ch].sprite_id, price.is_some()),
        };
        let f = bank.get(sprite).first();
        let alpha = if dim { 110 } else { 255 };
        cv.blit(f, x + CELL / 2 - f.w / 2, y + CELL / 2 - f.h / 2, Blit { alpha, ..Blit::default() });
        match b {
            Buy::Upgrade(u) => {
                let def = &content.meta.upgrades[u];
                let max = def.costs.len() as i32;
                let lv = super::upgrade_level(save, &def.id) as i32;
                let pw = max * 3 - 1;
                for k in 0..max {
                    let c = if k < lv { palette::GOLD } else { palette::SLATE };
                    cv.fill_rect(x + CELL / 2 - pw / 2 + k * 3, y + CELL + 2, 2, 2, c);
                }
            }
            _ if dim => cv.blit(lock, x + CELL - lock.w + 1, y + CELL - lock.h + 1, Blit::default()),
            _ => {}
        }
    }

    // Detail of the selected entry.
    let b = entries[sel];
    let pw = (cv.w - 16).min(236);
    let (px, py) = (cv.w / 2 - pw / 2, cv.h - 37);
    panel(cv, px, py, pw, 25, palette::SLATE);
    let (name, desc, level) = match b {
        Buy::Upgrade(u) => {
            let d = &content.meta.upgrades[u];
            let lv = format!("LV {}/{}", super::upgrade_level(save, &d.id), d.costs.len());
            (&d.name, &d.desc, Some(lv))
        }
        Buy::Item(i) => (&content.items[i].name, &content.items[i].desc, None),
        Buy::Hero(i) => (&content.characters[i].name, &content.characters[i].desc, None),
    };
    let name = name.to_uppercase();
    font::draw_outlined(cv, px + 5, py + 5, &name, palette::CREAM, INK);
    if let Some(lv) = level {
        font::draw_outlined(cv, px + 11 + font::width(&name), py + 5, &lv, palette::HAZE, INK);
    }
    let desc_w = pw - 10;
    if let Some(line) = font::wrap(desc, desc_w).first() {
        font::draw_outlined(cv, px + 5, py + 14, line, palette::FOG, INK);
    }
    match super::price(save, content, b) {
        Some(cost) => {
            let deny = h.deny > 0.0 && blink(h.deny, 12.0);
            let c = if deny {
                palette::RED
            } else if save.bones >= cost {
                palette::GOLD
            } else {
                palette::HAZE
            };
            bones_label(cv, px + pw - 5 - bones_width(cost), py + 5, cost, c);
        }
        None => {
            let (text, c) = match b {
                Buy::Upgrade(_) => ("MAX", palette::GOLD),
                _ => ("IN POOL", palette::LIME),
            };
            let c = if h.bought > FLASH / 2.0 && blink(clock, 10.0) { palette::CREAM } else { c };
            font::draw_outlined(cv, px + pw - 5 - font::width(text), py + 5, text, c, INK);
        }
    }
}

fn feats(cv: &mut Canvas, h: &Hub, save: &Save, content: &Content, _clock: f32) {
    let bank = bank();
    let list = &content.meta.feats;
    if list.is_empty() {
        return;
    }
    let sel = h.feat.min(list.len() - 1);
    let (on, off) = (bank.named("medal").first(), bank.named("medal_off").first());
    let pw = (cv.w - 16).min(220);
    let px = cv.w / 2 - pw / 2;
    let row_h = 11;
    let area = (cv.h - 42 - TOP).max(row_h);
    let visible = (area / row_h) as usize;
    let first = sel.saturating_sub(visible.saturating_sub(2)).min(list.len().saturating_sub(visible));

    let done_n = list.iter().filter(|f| save.feats.contains(&f.id)).count();
    for (k, f) in list.iter().enumerate().skip(first).take(visible) {
        let y = TOP + (k - first) as i32 * row_h;
        let done = save.feats.contains(&f.id);
        let is_sel = k == sel;
        if is_sel {
            let c = if h.on_tabs { palette::SLATE } else { palette::GOLD };
            cv.fill_rect(px, y, pw, row_h - 1, palette::DUSK);
            cv.rect(px, y, pw, row_h - 1, c);
        }
        let medal = if done { on } else { off };
        cv.blit(medal, px + 2, y, Blit::default());
        let c = if done { palette::BONE } else { palette::HAZE };
        font::draw_outlined(cv, px + 14, y + 3, &f.name.to_uppercase(), c, INK);
        let rc = if done { palette::MAUVE } else { palette::GOLD };
        match &f.reward {
            FeatReward::Bones(n) => {
                bones_label(cv, px + pw - 4 - bones_width(*n), y + 3, *n, rc);
            }
            FeatReward::Unlock(id) => {
                let s = super::unlock_name(content, id).to_uppercase();
                font::draw_outlined(cv, px + pw - 4 - font::width(&s), y + 3, &s, rc, INK);
            }
        }
    }
    // Scroll marks.
    if first > 0 {
        font::draw_outlined(cv, px + pw + 3, TOP + 2, "+", palette::MAUVE, INK);
    }
    if first + visible < list.len() {
        font::draw_outlined(cv, px + pw + 3, TOP + area - 8, "+", palette::MAUVE, INK);
    }

    // Detail: goal and progress.
    let f = &list[sel];
    let py = cv.h - 37;
    panel(cv, px, py, pw, 25, palette::SLATE);
    font::draw_outlined(cv, px + 5, py + 5, &f.goal.text(content).to_uppercase(), palette::CREAM, INK);
    let count = format!("{done_n}/{}", list.len());
    font::draw_outlined(cv, px + pw - 5 - font::width(&count), py + 5, &count, palette::HAZE, INK);
    let target = f.goal.target();
    let value = super::goal_value(&f.goal, save, content).min(target);
    let done = save.feats.contains(&f.id);
    let text = if done {
        "DONE".to_string()
    } else if f.goal.is_time() {
        format!("{} / {}", clock_text(value), clock_text(target))
    } else {
        format!("{} / {}", value as u64, target as u64)
    };
    let tw = font::width(&text);
    let bw = pw - 16 - tw;
    let fill = if done { bw } else { ((value / target.max(1e-3)) * bw as f32) as i32 };
    cv.fill_rect(px + 5, py + 15, bw + 2, 5, INK);
    cv.fill_rect(px + 6, py + 16, bw, 3, palette::DUSK);
    cv.fill_rect(px + 6, py + 16, fill, 3, if done { palette::LIME } else { palette::GOLD });
    font::draw_outlined(
        cv,
        px + pw - 5 - tw,
        py + 15,
        &text,
        if done { palette::LIME } else { palette::FOG },
        INK,
    );
}

/// The current feat banner, sliding down over the top edge.
pub fn toast(cv: &mut Canvas, toasts: &Toasts) {
    let Some((i, age)) = toasts.current() else { return };
    let content = content::get();
    let f = &content.meta.feats[i];
    let slide = (age / 0.2).min((TOAST_TIME - age) / 0.2).clamp(0.0, 1.0);
    let title = format!("FEAT {}", f.name.to_uppercase());
    let reward = match &f.reward {
        FeatReward::Bones(n) => format!("+{n}"),
        FeatReward::Unlock(id) => super::unlock_name(content, id).to_uppercase(),
    };
    let bone = matches!(f.reward, FeatReward::Bones(_));
    let reward_w = font::width(&reward) + if bone { 11 } else { 0 };
    let (w, h) = ((font::width(&title) + reward_w + 30).min(cv.w - 4), 13);
    let x = cv.w / 2 - w / 2;
    let y = 1 - ((1.0 - slide) * (h + 3) as f32) as i32;
    panel(cv, x, y, w, h, palette::GOLD);
    cv.blit(bank().named("medal").first(), x + 3, y + 2, Blit::default());
    font::draw_outlined(cv, x + 14, y + 4, &title, palette::GOLD, INK);
    let rx = x + w - 5 - reward_w;
    if bone {
        cv.blit(bank().named("bone").first(), rx, y + 4, Blit::default());
    }
    font::draw_outlined(cv, rx + reward_w - font::width(&reward), y + 4, &reward, palette::CREAM, INK);
}
