//! Sprites are defined in code as string pixel grids (see `palette` for the
//! character map). Each area keeps its own `sprites.rs` with a `SPRITES` list;
//! the bank collects them all at startup and content refers to them by name.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::palette::{self, CLEAR};

/// A sprite as written in source.
pub struct SpriteDef {
    pub name: &'static str,
    /// Add a 1px ink outline around the art (grows each frame by 1px per side).
    pub outline: bool,
    /// One or more frames of equal size, rows top to bottom.
    pub frames: &'static [&'static [&'static str]],
}

/// A compiled frame: palette indices, `CLEAR` for transparent.
pub struct Frame {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

pub struct Sprite {
    pub frames: Vec<Frame>,
    /// Distinct non-ink colors, most common first. Used for particles.
    pub colors: Vec<u8>,
}

impl Sprite {
    /// Frame for an animation clock `t` (seconds) at `fps`.
    pub fn frame_at(&self, t: f32, fps: f32) -> &Frame {
        let i = (t.max(0.0) * fps) as usize % self.frames.len();
        &self.frames[i]
    }

    pub fn first(&self) -> &Frame {
        &self.frames[0]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpriteId(pub u16);

pub struct SpriteBank {
    sprites: Vec<Sprite>,
    names: HashMap<&'static str, SpriteId>,
}

impl SpriteBank {
    fn build() -> Result<Self, String> {
        let lists: [&[SpriteDef]; 5] = [
            crate::render::sprites::SPRITES,
            crate::ui::sprites::SPRITES,
            crate::items::sprites::SPRITES,
            crate::enemies::sprites::SPRITES,
            crate::meta::sprites::SPRITES,
        ];
        let mut bank = SpriteBank { sprites: Vec::new(), names: HashMap::new() };
        for def in lists.iter().flat_map(|l| l.iter()) {
            let id = SpriteId(bank.sprites.len() as u16);
            if bank.names.insert(def.name, id).is_some() {
                return Err(format!("duplicate sprite name `{}`", def.name));
            }
            bank.sprites.push(compile(def)?);
        }
        Ok(bank)
    }

    pub fn id(&self, name: &str) -> Option<SpriteId> {
        self.names.get(name).copied()
    }

    pub fn get(&self, id: SpriteId) -> &Sprite {
        &self.sprites[id.0 as usize]
    }

    /// Sprite by name; panics if missing (engine-required sprites only).
    pub fn named(&self, name: &str) -> &Sprite {
        match self.id(name) {
            Some(id) => self.get(id),
            None => panic!("missing sprite `{name}`"),
        }
    }
}

static BANK: OnceLock<SpriteBank> = OnceLock::new();

/// The global sprite bank, compiled on first use.
pub fn bank() -> &'static SpriteBank {
    BANK.get_or_init(|| SpriteBank::build().unwrap_or_else(|e| panic!("sprites: {e}")))
}

pub fn compile(def: &SpriteDef) -> Result<Sprite, String> {
    let err = |msg: String| format!("sprite `{}`: {msg}", def.name);
    if def.frames.is_empty() {
        return Err(err("no frames".into()));
    }
    let h = def.frames[0].len();
    let w = def.frames[0].first().map_or(0, |r| r.chars().count());
    if w == 0 || h == 0 {
        return Err(err("empty frame".into()));
    }
    let mut frames = Vec::with_capacity(def.frames.len());
    let mut counts = [0u32; 32];
    for (fi, rows) in def.frames.iter().enumerate() {
        if rows.len() != h {
            return Err(err(format!("frame {fi} has {} rows, expected {h}", rows.len())));
        }
        let mut px = Vec::with_capacity(w * h);
        for (ri, row) in rows.iter().enumerate() {
            if row.chars().count() != w {
                return Err(err(format!("frame {fi} row {ri} is not {w} wide")));
            }
            for c in row.chars() {
                let idx = palette::index_of(c)
                    .ok_or_else(|| err(format!("unknown color `{c}` in frame {fi} row {ri}")))?;
                if idx != CLEAR {
                    counts[idx as usize] += 1;
                }
                px.push(idx);
            }
        }
        let frame = Frame { w: w as i32, h: h as i32, px };
        frames.push(if def.outline { outlined(&frame) } else { frame });
    }
    let mut colors: Vec<u8> = (1..32u8).filter(|&i| counts[i as usize] > 0).collect();
    colors.sort_by_key(|&i| std::cmp::Reverse(counts[i as usize]));
    Ok(Sprite { frames, colors })
}

/// Grow by 1px each side and ring opaque pixels with ink.
fn outlined(f: &Frame) -> Frame {
    let (w, h) = (f.w + 2, f.h + 2);
    let mut px = vec![CLEAR; (w * h) as usize];
    for y in 0..f.h {
        for x in 0..f.w {
            px[((y + 1) * w + x + 1) as usize] = f.px[(y * f.w + x) as usize];
        }
    }
    let solid = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && px[(y * w + x) as usize] != CLEAR;
    let mut out = px.clone();
    for y in 0..h {
        for x in 0..w {
            if px[(y * w + x) as usize] == CLEAR
                && (solid(x - 1, y) || solid(x + 1, y) || solid(x, y - 1) || solid(x, y + 1))
            {
                out[(y * w + x) as usize] = 0;
            }
        }
    }
    Frame { w, h, px: out }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_sprites_compile() {
        if let Err(e) = SpriteBank::build() {
            panic!("{e}");
        }
    }
}
