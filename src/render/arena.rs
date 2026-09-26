//! Pre-rendered arena floor and walls, generated once per run from a seed,
//! plus the animated ambience (motes, torches) drawn on top every frame.
//!
//! Each stage names a biome. Biomes only change the look: ground ramp, ruin
//! slabs, props, walls, light tint and ambient motes. A run generates every
//! biome from one seed, so the arena keeps its layout as the biome changes.

use serde::Deserialize;

use super::canvas::{Blit, Canvas};
use super::noise;
use super::palette::{self, Color};
use super::sprite::bank;
use crate::engine::{Rect, Rng};

/// Border drawn around the arena, in pixels. World (0, 0) is floor pixel (PAD, PAD).
pub const PAD: i32 = 16;
const TILE: i32 = 16;
const WALL: i32 = 12;
const TORCH_EVERY: i32 = 96;

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Biome {
    /// Violet flagstones, tar pools, fossils.
    #[default]
    Tarpit,
    /// Teal bog, ferns, puddles, fireflies.
    Fernbog,
    /// Umber ash, cooled magma seams, obsidian, embers.
    Ashfall,
}

impl Biome {
    pub const ALL: [Biome; 3] = [Biome::Tarpit, Biome::Fernbog, Biome::Ashfall];

    pub fn for_seed(seed: u64) -> Biome {
        Self::ALL[(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) >> 40) as usize % 3]
    }

    /// Color the screen edges fade toward.
    pub fn shade(self) -> Color {
        match self {
            Biome::Tarpit => palette::INK,
            Biome::Fernbog => palette::INK.mix(palette::DEEP, 110),
            Biome::Ashfall => palette::INK.mix(palette::MAROON, 90),
        }
    }

    fn look(self) -> Look {
        use palette::*;
        match self {
            Biome::Tarpit => Look {
                ground: [NIGHT, NIGHT.mix(DUSK, 120), DUSK],
                slab: [NIGHT, DUSK.mix(SLATE, 60), DUSK.mix(SLATE, 150)],
                wall: [NIGHT, SLATE, MAUVE, HAZE],
                rim: NIGHT,
                props: &[("p_ribs", 2), ("p_skull", 2), ("p_bone", 3), ("p_rock", 5), ("p_tuft_dim", 6)],
                pool: Some([NIGHT, INK, PLUM]),
                seam: None,
            },
            Biome::Fernbog => Look {
                ground: [INK.mix(DEEP, 150), DEEP.mix(NIGHT, 90), DEEP.mix(NAVY, 70)],
                slab: [INK.mix(DEEP, 150), DEEP.mix(SLATE, 110), SLATE.mix(DEEP, 80)],
                wall: [INK.mix(DEEP, 120), DEEP.mix(SLATE, 140), SLATE, MOSS],
                rim: INK.mix(DEEP, 120),
                props: &[("p_fern", 6), ("p_tuft", 8), ("p_bloom", 3), ("p_mossrock", 4), ("p_bone", 1)],
                pool: Some([INK.mix(DEEP, 150), NAVY, BLUE]),
                seam: None,
            },
            Biome::Ashfall => Look {
                ground: [INK.mix(MAROON, 40), INK.mix(UMBER, 110), NIGHT.mix(UMBER, 90)],
                slab: [INK, NIGHT.mix(UMBER, 120), DUSK.mix(UMBER, 110)],
                wall: [INK, NIGHT.mix(UMBER, 140), SLATE.mix(UMBER, 90), EMBER.mix(MAROON, 120)],
                rim: INK,
                props: &[("p_obsidian", 2), ("p_basalt", 5), ("p_skull", 1), ("p_ashpile", 4)],
                pool: None,
                seam: Some([MAROON.mix(INK, 80), BLOOD, EMBER]),
            },
        }
    }

    /// Wall torch flame colors: core, body, glow.
    fn flame(self) -> [Color; 3] {
        match self {
            Biome::Tarpit => [palette::CREAM, palette::AMBER, palette::EMBER],
            Biome::Fernbog => [palette::ICE, palette::CYAN, palette::SKY],
            Biome::Ashfall => [palette::GOLD, palette::EMBER, palette::RED],
        }
    }
}

struct Look {
    /// Ground ramp, dark to light.
    ground: [Color; 3],
    /// Ruin slab: gap, face, highlight.
    slab: [Color; 3],
    /// Wall: mortar, face, light, cap.
    wall: [Color; 4],
    rim: Color,
    /// Prop sprite names with weights.
    props: &'static [(&'static str, u32)],
    /// Pool rim, body, sheen.
    pool: Option<[Color; 3]>,
    /// Glowing seam: edge, body, core.
    seam: Option<[Color; 3]>,
}

/// A generated arena backdrop.
pub struct Floor {
    pub cv: Canvas,
    pub biome: Biome,
}

/// One floor per biome, all from the same seed.
pub struct Floors {
    floors: Vec<Floor>,
    /// The biome shown behind the menus.
    menu: Biome,
}

impl Floors {
    pub fn new(arena: Rect, seed: u64) -> Floors {
        let floors = Biome::ALL.iter().map(|&b| floor(arena, seed, b)).collect();
        Floors { floors, menu: Biome::for_seed(seed) }
    }

    pub fn get(&self, biome: Biome) -> &Floor {
        &self.floors[biome as usize]
    }

    pub fn menu(&self) -> &Floor {
        self.get(self.menu)
    }
}

fn floor(arena: Rect, seed: u64, biome: Biome) -> Floor {
    let look = biome.look();
    let (aw, ah) = (arena.w as i32, arena.h as i32);
    let mut cv = Canvas::new(aw + PAD * 2, ah + PAD * 2);
    let mut rng = Rng::new(seed ^ 0xf1_00_4f);
    let s = rng.next_u32();
    cv.clear(palette::INK);

    ground(&mut cv, &look, aw, ah, s);
    slabs(&mut cv, &look, &mut rng, aw, ah, s);
    if let Some(pool) = look.pool {
        pools(&mut cv, pool, &mut rng, aw, ah);
    }
    if let Some(seam) = look.seam {
        seams(&mut cv, seam, &mut rng, aw, ah);
    }
    props(&mut cv, &look, &mut rng, aw, ah);
    walls(&mut cv, &look, aw, ah);
    Floor { cv, biome }
}

/// Dithered three-tone ground with large soft blotches and fine grit.
fn ground(cv: &mut Canvas, look: &Look, aw: i32, ah: i32, s: u32) {
    for y in 0..ah {
        for x in 0..aw {
            let n =
                noise::fbm(x as f32 / 38.0, y as f32 / 30.0, s) * 0.85 + noise::hash2(x, y, s ^ 0x55) * 0.15;
            let v = ((n - 0.28) * 1.7).clamp(0.0, 1.0);
            let c = look.ground[noise::dither(v, 3, x, y)];
            cv.put(PAD + x, PAD + y, c);
        }
    }
}

/// Patches of old flagstones where a low-frequency noise field is high.
fn slabs(cv: &mut Canvas, look: &Look, rng: &mut Rng, aw: i32, ah: i32, s: u32) {
    let [gap, face, hi] = look.slab;
    let lo = face.mix(gap, 110);
    for ty in 0..ah / TILE {
        for tx in 0..aw / TILE {
            let n = noise::value(tx as f32 / 3.5, ty as f32 / 3.0, s ^ 0xab);
            if n < 0.56 || (n < 0.66 && rng.chance(0.55)) {
                continue;
            }
            let (x0, y0) = (PAD + tx * TILE, PAD + ty * TILE);
            let tone = face.mix(gap, rng.below(50) as u8);
            cv.fill_rect(x0 + 1, y0 + 1, TILE - 1, TILE - 1, tone);
            cv.hline(x0 + 1, x0 + TILE - 1, y0 + 1, hi.mix(tone, 60));
            cv.vline(x0 + 1, y0 + 2, y0 + TILE - 2, hi.mix(tone, 150));
            cv.hline(x0 + 2, x0 + TILE - 1, y0 + TILE - 1, lo);
            cv.vline(x0 + TILE - 1, y0 + 2, y0 + TILE - 1, lo);
            // Grit and a chipped corner.
            for _ in 0..4 {
                let (x, y) = (x0 + 3 + rng.below(11) as i32, y0 + 3 + rng.below(11) as i32);
                cv.put(x, y, if rng.chance(0.5) { lo } else { hi.mix(tone, 120) });
            }
            if rng.chance(0.35) {
                let (cx, cy) = (x0 + if rng.chance(0.5) { 1 } else { TILE - 3 }, y0 + TILE - 3);
                cv.fill_rect(cx, cy, 2, 2, gap);
            }
            if rng.chance(0.25) {
                let (cx, cy) = (x0 + 4 + rng.below(6) as i32, y0 + 3 + rng.below(8) as i32);
                crack(cv, rng, cx, cy, lo);
            }
        }
    }
}

fn crack(cv: &mut Canvas, rng: &mut Rng, mut x: i32, mut y: i32, c: Color) {
    for _ in 0..4 + rng.below(5) {
        cv.put(x, y, c);
        x += if rng.chance(0.65) { 1 } else { 0 };
        y += rng.below(3) as i32 - 1;
    }
}

fn pools(cv: &mut Canvas, [rim, body, sheen]: [Color; 3], rng: &mut Rng, aw: i32, ah: i32) {
    for _ in 0..(aw * ah / 20000) {
        let x = PAD + 24 + rng.below((aw - 48) as usize) as i32;
        let y = PAD + 24 + rng.below((ah - 48) as usize) as i32;
        let (rx, ry) = (6 + rng.below(9) as i32, 3 + rng.below(4) as i32);
        cv.blend_ellipse(x, y + 1, rx + 1, ry + 1, rim, 255);
        cv.blend_ellipse(x, y, rx, ry, body, 255);
        cv.blend_ellipse(x + 1, y + 1, rx - 2, ry - 1, body.mix(palette::INK, 60), 255);
        cv.hline(x - rx / 2, x - rx / 2 + 2, y - ry / 2, sheen);
        cv.put(x + rx / 3, y, sheen.mix(body, 100));
    }
}

/// Branching cooled-magma seams with a dim glowing core.
fn seams(cv: &mut Canvas, [edge, body, core]: [Color; 3], rng: &mut Rng, aw: i32, ah: i32) {
    for _ in 0..(aw * ah / 14000) {
        let (mut x, mut y) = (
            (PAD + 12 + rng.below((aw - 24) as usize) as i32) as f32,
            (PAD + 12 + rng.below((ah - 24) as usize) as i32) as f32,
        );
        let mut a = rng.angle();
        for i in 0..14 + rng.below(16) {
            let (ix, iy) = (x as i32, y as i32);
            cv.put(ix, iy + 1, edge);
            cv.put(ix, iy, if i % 5 == 2 { core } else { body });
            a += rng.range(-0.6, 0.6);
            x += a.cos();
            y += a.sin() * 0.7;
        }
    }
}

fn props(cv: &mut Canvas, look: &Look, rng: &mut Rng, aw: i32, ah: i32) {
    let bank = bank();
    let total: u32 = look.props.iter().map(|p| p.1).sum();
    for _ in 0..(aw * ah / 1500) {
        let mut roll = rng.below(total as usize) as u32;
        let name = look.props.iter().find(|p| {
            let hit = roll < p.1;
            roll = roll.saturating_sub(p.1);
            hit
        });
        let Some(&(name, _)) = name else { continue };
        let f = bank.named(name).first();
        let x = PAD + 6 + rng.below((aw - 12 - f.w) as usize) as i32;
        let y = PAD + 14 + rng.below((ah - 20 - f.h) as usize) as i32;
        cv.blit(f, x, y, Blit { flip_x: rng.chance(0.5), ..Blit::default() });
    }
}

/// Brick wall face above the arena, dark rim on the other sides, inner shadow.
fn walls(cv: &mut Canvas, look: &Look, aw: i32, ah: i32) {
    let [mortar, face, light, cap] = look.wall;
    let (x0, y0, x1, y1) = (PAD, PAD, PAD + aw, PAD + ah);
    let top = y0 - WALL;
    cv.fill_rect(x0 - 4, top, aw + 8, WALL, face);
    for row in 0..WALL / 4 {
        let y = top + 1 + row * 4;
        cv.hline(x0 - 4, x1 + 3, y, mortar);
        cv.hline(x0 - 4, x1 + 3, y + 1, light.mix(face, 110));
        let off = if row % 2 == 0 { 0 } else { 7 };
        let mut x = x0 - 4 + off;
        while x < x1 + 4 {
            cv.vline(x, y + 1, y + 3, mortar);
            cv.put(x + 1, y + 1, light);
            x += 14;
        }
    }
    cv.hline(x0 - 4, x1 + 3, top, cap);
    cv.hline(x0 - 4, x1 + 3, top - 1, mortar);
    cv.hline(x0 - 4, x1 + 3, y0 - 1, mortar);
    // Torch brackets; flames are animated in `ambient`.
    let mut tx = x0 + TORCH_EVERY / 2;
    while tx < x1 {
        cv.fill_rect(tx - 2, y0 - 5, 5, 2, mortar);
        cv.fill_rect(tx - 1, y0 - 6, 3, 1, light);
        tx += TORCH_EVERY;
    }
    // Side and bottom rims.
    cv.fill_rect(x0 - 4, y0, 4, ah + 4, look.rim);
    cv.fill_rect(x1, y0, 4, ah + 4, look.rim);
    cv.fill_rect(x0 - 4, y1, aw + 8, 4, look.rim);
    cv.vline(x0 - 1, y0, y1, face);
    cv.vline(x1, y0, y1, face);
    cv.hline(x0 - 1, x1, y1, face);
    // Soft shadow under the top wall.
    for i in 0..6 {
        let a = (150 - i * 25) as u8;
        for x in x0..x1 {
            cv.blend(x, y0 + i, palette::INK, a);
        }
    }
}

impl Floor {
    /// Animated ambience over the world: floating motes and flickering wall
    /// torches. `origin` is the world coordinate at the top-left screen pixel.
    pub fn ambient(&self, cv: &mut Canvas, origin: (i32, i32), t: f32) {
        torches(cv, self.biome, self.cv.w - PAD, origin, t);
        motes(cv, self.biome, origin, t);
    }
}

fn motes(cv: &mut Canvas, biome: Biome, origin: (i32, i32), t: f32) {
    let (vw, vh) = (cv.w + 16, cv.h + 16);
    let n = (cv.w * cv.h / 1400) as u32;
    for i in 0..n {
        let (hx, hy, hp) =
            (noise::hash2(i as i32, 1, 7), noise::hash2(i as i32, 2, 7), noise::hash2(i as i32, 3, 7));
        let phase = hp * 40.0;
        let (bx, by, c, a) = match biome {
            // Spores drifting up slowly.
            Biome::Tarpit => {
                let y = hy * 600.0 - t * (3.0 + hp * 4.0);
                let x = hx * 900.0 + (t * 0.7 + phase).sin() * 4.0;
                (x, y, if i % 3 == 0 { palette::GRAPE } else { palette::MAUVE }, 90)
            }
            // Fireflies wandering and blinking.
            Biome::Fernbog => {
                let x = hx * 900.0 + (t * 0.45 + phase).sin() * 14.0;
                let y = hy * 600.0 + (t * 0.6 + phase * 1.3).cos() * 9.0;
                let glow = ((t * 1.7 + phase).sin() * 0.5 + 0.5).powi(3);
                if glow < 0.15 {
                    continue;
                }
                (x, y, if i % 4 == 0 { palette::SPROUT } else { palette::LIME }, (glow * 220.0) as u8)
            }
            // Embers rising and flickering.
            Biome::Ashfall => {
                let y = hy * 600.0 - t * (10.0 + hp * 14.0);
                let x = hx * 900.0 + (t * 1.3 + phase).sin() * 5.0;
                let flick = (t * 9.0 + phase).sin() > -0.4;
                (x, y, if flick { palette::AMBER } else { palette::EMBER }, 200)
            }
        };
        let sx = (bx as i32 - origin.0).rem_euclid(vw) - 8;
        let sy = (by as i32 - origin.1).rem_euclid(vh) - 8;
        if biome == Biome::Fernbog && a > 150 {
            cv.blend(sx - 1, sy, c, a / 4);
            cv.blend(sx + 1, sy, c, a / 4);
            cv.blend(sx, sy - 1, c, a / 4);
            cv.blend(sx, sy + 1, c, a / 4);
        }
        cv.blend(sx, sy, c, a);
    }
}

fn torches(cv: &mut Canvas, biome: Biome, right: i32, (ox, oy): (i32, i32), t: f32) {
    let [core, body, glow] = biome.flame();
    let y = PAD - 8 - oy;
    if y < -12 || y > cv.h + 12 {
        return;
    }
    let mut wx = PAD + TORCH_EVERY / 2;
    while wx < right {
        let x = wx - ox;
        wx += TORCH_EVERY;
        if x < -16 || x > cv.w + 16 {
            continue;
        }
        let k = (t * 11.0 + x as f32 * 0.37).sin();
        let r = if k > 0.3 { 9 } else { 8 };
        cv.blend_ellipse(x, y + 2, r + 3, r - 2, glow, 22);
        cv.blend_ellipse(x, y + 2, r - 3, r - 5, body, 30);
        let lean = if k > 0.6 {
            1
        } else if k < -0.6 {
            -1
        } else {
            0
        };
        cv.put(x + lean, y - 2, body);
        cv.fill_rect(x - 1, y - 1, 3, 2, body);
        cv.put(x, y, core);
        cv.put(x + lean, y - 1, core);
        cv.hline(x - 1, x + 1, y + 1, glow);
    }
}
