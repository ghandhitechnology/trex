//! Screen-space lighting: dithered vignettes and the closing iris on death.
//! Alpha is stepped with an ordered dither so it keeps the pixel-art look.

use super::canvas::Canvas;
use super::noise;
use super::palette::Color;

/// Darken toward `tint` away from `center`. `reach` scales the lit area
/// (1.0 is about the screen), `strength` is the alpha at the far corners.
pub fn vignette(cv: &mut Canvas, center: (i32, i32), tint: Color, reach: f32, strength: u8) {
    const STEPS: usize = 5;
    let (sx, sy) = (1.0 / (cv.w as f32 * 0.6 * reach), 1.0 / (cv.h as f32 * 0.62 * reach));
    for y in 0..cv.h {
        let dy = (y - center.1) as f32 * sy;
        let dy2 = dy * dy;
        for x in 0..cv.w {
            let dx = (x - center.0) as f32 * sx;
            let d = dx * dx + dy2;
            if d < 0.3 {
                continue;
            }
            let v = ((d - 0.3) / 1.1).min(1.0);
            let v = v * v * (3.0 - 2.0 * v);
            let level = noise::dither(v, STEPS, x, y);
            if level > 0 {
                cv.blend(x, y, tint, (level as u32 * strength as u32 / (STEPS as u32 - 1)) as u8);
            }
        }
    }
}

/// Fill everything outside a circle of `radius` with `c`, dithering a 4px
/// edge just inside it. A radius of zero covers the whole screen.
pub fn iris(cv: &mut Canvas, center: (i32, i32), radius: f32, c: Color) {
    let edge = 4.0;
    for y in 0..cv.h {
        let dy = (y - center.1) as f32;
        for x in 0..cv.w {
            let dx = (x - center.0) as f32;
            let d = (dx * dx + dy * dy).sqrt() - radius + edge;
            if d >= edge || (d > 0.0 && d / edge > noise::bayer(x, y)) {
                cv.put(x, y, c);
            }
        }
    }
}
