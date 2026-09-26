//! Stateless hashing, value noise and ordered dither for procedural art.

/// Integer hash of a lattice point, uniform in [0, 1).
pub fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1) ^ seed;
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// Smooth value noise in [0, 1).
pub fn value(x: f32, y: f32, seed: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (ix, iy) = (x0 as i32, y0 as i32);
    let a = hash2(ix, iy, seed);
    let b = hash2(ix + 1, iy, seed);
    let c = hash2(ix, iy + 1, seed);
    let d = hash2(ix + 1, iy + 1, seed);
    let top = a + (b - a) * sx;
    let bottom = c + (d - c) * sx;
    top + (bottom - top) * sy
}

/// Two octaves of value noise, roughly in [0, 1).
pub fn fbm(x: f32, y: f32, seed: u32) -> f32 {
    value(x, y, seed) * 0.65 + value(x * 2.3, y * 2.3, seed ^ 0x9e37) * 0.35
}

const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// 4x4 ordered dither threshold in [0, 1).
pub fn bayer(x: i32, y: i32) -> f32 {
    (BAYER[(y & 3) as usize][(x & 3) as usize] as f32 + 0.5) / 16.0
}

/// Quantize `v` in [0, 1] to `steps` levels with ordered dither, returning a
/// level in 0..steps.
pub fn dither(v: f32, steps: usize, x: i32, y: i32) -> usize {
    let s = v.clamp(0.0, 1.0) * (steps - 1) as f32;
    let lo = s.floor();
    let level = if s - lo > bayer(x, y) { lo + 1.0 } else { lo };
    (level as usize).min(steps - 1)
}
