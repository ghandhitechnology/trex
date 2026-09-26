//! Pre-rendered arena floor and walls, generated once from a seed.

use super::canvas::Canvas;
use super::palette::{self, Color};
use crate::engine::{Rect, Rng};

/// Border drawn around the arena, in pixels. World (0, 0) is floor pixel (PAD, PAD).
pub const PAD: i32 = 16;
const TILE: i32 = 16;
const WALL: i32 = 10;

pub fn floor(arena: Rect, seed: u64) -> Canvas {
    let (aw, ah) = (arena.w as i32, arena.h as i32);
    let mut cv = Canvas::new(aw + PAD * 2, ah + PAD * 2);
    let mut rng = Rng::new(seed ^ 0xf1_00_4f);
    cv.clear(palette::INK);

    // Flagstones with slight per-tile tone and mortar lines.
    for ty in 0..(ah + TILE - 1) / TILE {
        for tx in 0..(aw + TILE - 1) / TILE {
            let tone = palette::DUSK.mix(palette::NIGHT, rng.below(70) as u8);
            let (x0, y0) = (PAD + tx * TILE, PAD + ty * TILE);
            cv.fill_rect(x0, y0, TILE, TILE, tone);
            let hi = tone.mix(palette::SLATE, 70);
            cv.hline(x0 + 1, x0 + TILE - 2, y0 + 1, hi);
            cv.hline(x0, x0 + TILE - 1, y0, palette::NIGHT);
            cv.vline(x0, y0, y0 + TILE - 1, palette::NIGHT);
            for _ in 0..6 {
                let (x, y) = (
                    x0 + 1 + rng.below(TILE as usize - 2) as i32,
                    y0 + 2 + rng.below(TILE as usize - 3) as i32,
                );
                let c =
                    if rng.chance(0.5) { tone.mix(palette::SLATE, 90) } else { tone.mix(palette::INK, 70) };
                cv.put(x, y, c);
            }
            decorate(&mut cv, &mut rng, x0, y0);
        }
    }

    // Tar pools.
    for _ in 0..(aw * ah / 22000) {
        let x = PAD + 24 + rng.below((aw - 48) as usize) as i32;
        let y = PAD + 24 + rng.below((ah - 48) as usize) as i32;
        let (rx, ry) = (6 + rng.below(8) as i32, 3 + rng.below(4) as i32);
        cv.blend_ellipse(x, y + 1, rx + 1, ry + 1, palette::NIGHT, 255);
        cv.blend_ellipse(x, y, rx, ry, palette::INK, 255);
        cv.hline(x - rx / 2, x - rx / 2 + 2, y - ry / 2, palette::PLUM);
        cv.put(x + rx / 3, y, palette::PLUM);
    }

    walls(&mut cv, aw, ah);
    cv
}

fn decorate(cv: &mut Canvas, rng: &mut Rng, x0: i32, y0: i32) {
    let at = |rng: &mut Rng| {
        (x0 + 3 + rng.below(TILE as usize - 6) as i32, y0 + 3 + rng.below(TILE as usize - 6) as i32)
    };
    let roll = rng.f32();
    if roll < 0.10 {
        // Pebble.
        let (x, y) = at(rng);
        cv.fill_rect(x, y, 2, 1, palette::SLATE);
        cv.put(x, y - 1, palette::MAUVE);
        cv.fill_rect(x, y + 1, 2, 1, palette::NIGHT);
    } else if roll < 0.17 {
        // Crack.
        let (mut x, mut y) = at(rng);
        for _ in 0..5 + rng.below(4) {
            cv.put(x, y, palette::NIGHT);
            x += if rng.chance(0.6) { 1 } else { 0 };
            y += rng.below(3) as i32 - 1;
        }
    } else if roll < 0.26 {
        // Moss tuft.
        let (x, y) = at(rng);
        let tuft: [(i32, i32, Color); 6] = [
            (0, 0, palette::MOSS),
            (1, -1, palette::LEAF),
            (2, 0, palette::MOSS),
            (-1, 1, palette::DEEP),
            (1, 1, palette::DEEP),
            (3, 1, palette::DEEP),
        ];
        for (dx, dy, c) in tuft {
            cv.put(x + dx, y + dy, c);
        }
    } else if roll < 0.28 {
        // Old bone.
        let (x, y) = at(rng);
        cv.hline(x, x + 3, y, palette::HAZE);
        cv.put(x - 1, y - 1, palette::FOG);
        cv.put(x - 1, y + 1, palette::HAZE);
        cv.put(x + 4, y - 1, palette::FOG);
        cv.put(x + 4, y + 1, palette::HAZE);
    }
}

/// Brick wall face above the arena, dark rim on the other sides, inner shadow.
fn walls(cv: &mut Canvas, aw: i32, ah: i32) {
    let (x0, y0, x1, y1) = (PAD, PAD, PAD + aw, PAD + ah);
    // Top wall face.
    let top = y0 - WALL;
    cv.fill_rect(x0 - 4, top, aw + 8, WALL, palette::SLATE);
    for row in 0..(WALL / 5) {
        let y = top + row * 5;
        cv.hline(x0 - 4, x1 + 3, y, palette::NIGHT);
        cv.hline(x0 - 4, x1 + 3, y + 1, palette::MAUVE);
        let off = if row % 2 == 0 { 0 } else { 6 };
        let mut x = x0 - 4 + off;
        while x < x1 + 4 {
            cv.vline(x, y + 1, y + 4, palette::NIGHT);
            x += 12;
        }
    }
    cv.hline(x0 - 4, x1 + 3, top - 1, palette::HAZE);
    // Side and bottom rims.
    cv.fill_rect(x0 - 4, y0, 4, ah + 4, palette::NIGHT);
    cv.fill_rect(x1, y0, 4, ah + 4, palette::NIGHT);
    cv.fill_rect(x0 - 4, y1, aw + 8, 4, palette::NIGHT);
    cv.vline(x0 - 1, y0, y1, palette::SLATE);
    cv.vline(x1, y0, y1, palette::SLATE);
    cv.hline(x0 - 1, x1, y1, palette::SLATE);
    // Soft shadow under the top wall.
    for i in 0..5 {
        let a = (130 - i * 26) as u8;
        for x in x0..x1 {
            cv.blend(x, y0 + i, palette::INK, a);
        }
    }
}
