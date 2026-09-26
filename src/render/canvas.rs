use super::palette::{self, CLEAR, Color, INK};
use super::sprite::Frame;

/// CPU framebuffer. Coordinates are logical pixels, origin top-left.
pub struct Canvas {
    pub w: i32,
    pub h: i32,
    pub px: Vec<Color>,
}

/// Options for drawing a sprite frame.
#[derive(Clone, Copy, Debug)]
pub struct Blit {
    pub flip_x: bool,
    /// Paint every non-outline pixel with this color (hit flash, silhouettes).
    pub flash: Option<Color>,
    pub alpha: u8,
}

impl Default for Blit {
    fn default() -> Self {
        Self { flip_x: false, flash: None, alpha: 255 }
    }
}

impl Canvas {
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h, px: vec![INK; (w * h) as usize] }
    }

    pub fn resize(&mut self, w: i32, h: i32) {
        self.w = w;
        self.h = h;
        self.px = vec![INK; (w * h) as usize];
    }

    pub fn clear(&mut self, c: Color) {
        self.px.fill(c);
    }

    #[inline]
    pub fn put(&mut self, x: i32, y: i32, c: Color) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }

    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, c: Color, a: u8) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            let p = &mut self.px[(y * self.w + x) as usize];
            *p = p.mix(c, a);
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        let (x0, x1) = (x.clamp(0, self.w), (x + w).min(self.w));
        let (y0, y1) = (y.max(0), (y + h).min(self.h));
        for yy in y0..y1 {
            let row = (yy * self.w) as usize;
            self.px[row + x0 as usize..row + x1.max(x0) as usize].fill(c);
        }
    }

    /// One-pixel rectangle outline.
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        self.hline(x, x + w - 1, y, c);
        self.hline(x, x + w - 1, y + h - 1, c);
        self.vline(x, y, y + h - 1, c);
        self.vline(x + w - 1, y, y + h - 1, c);
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: Color) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.put(x, y, c);
        }
    }

    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, c: Color) {
        for y in y0.min(y1)..=y0.max(y1) {
            self.put(x, y, c);
        }
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err) = (x0, y0, dx + dy);
        loop {
            self.put(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, c: Color) {
        let rr = r * r + r;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= rr {
                    self.put(cx + dx, cy + dy, c);
                }
            }
        }
    }

    pub fn circle(&mut self, cx: i32, cy: i32, r: i32, c: Color) {
        let (outer, inner) = (r * r + r, (r - 1) * (r - 1) + (r - 1));
        for dy in -r..=r {
            for dx in -r..=r {
                let d = dx * dx + dy * dy;
                if d <= outer && (d > inner || r <= 0) {
                    self.put(cx + dx, cy + dy, c);
                }
            }
        }
    }

    /// Filled ellipse blended at `a` (shadows, glows).
    pub fn blend_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, c: Color, a: u8) {
        if rx <= 0 || ry <= 0 {
            return;
        }
        let (fx, fy) = (rx as f32 + 0.5, ry as f32 + 0.5);
        for dy in -ry..=ry {
            for dx in -rx..=rx {
                let (nx, ny) = (dx as f32 / fx, dy as f32 / fy);
                if nx * nx + ny * ny <= 1.0 {
                    self.blend(cx + dx, cy + dy, c, a);
                }
            }
        }
    }

    /// Draw a sprite frame with its top-left corner at (x, y).
    pub fn blit(&mut self, f: &Frame, x: i32, y: i32, o: Blit) {
        for sy in 0..f.h {
            let ty = y + sy;
            if ty < 0 || ty >= self.h {
                continue;
            }
            for sx in 0..f.w {
                let idx = f.px[(sy * f.w + if o.flip_x { f.w - 1 - sx } else { sx }) as usize];
                if idx == CLEAR {
                    continue;
                }
                let c = match o.flash {
                    Some(fc) if idx != 0 => fc,
                    _ => palette::color(idx),
                };
                if o.alpha == 255 {
                    self.put(x + sx, ty, c);
                } else {
                    self.blend(x + sx, ty, c, o.alpha);
                }
            }
        }
    }

    /// Draw a sprite frame centered on (cx, cy).
    pub fn blit_centered(&mut self, f: &Frame, cx: i32, cy: i32, o: Blit) {
        self.blit(f, cx - f.w / 2, cy - f.h / 2, o);
    }

    /// Nearest-neighbor enlarged sprite (UI icons, logo).
    pub fn blit_scaled(&mut self, f: &Frame, x: i32, y: i32, k: i32) {
        for sy in 0..f.h {
            for sx in 0..f.w {
                let idx = f.px[(sy * f.w + sx) as usize];
                if idx != CLEAR {
                    self.fill_rect(x + sx * k, y + sy * k, k, k, palette::color(idx));
                }
            }
        }
    }

    /// Copy a region of `src` whose top-left is (sx, sy) to (0, 0) of self,
    /// filling anything outside `src` with `fill`.
    pub fn copy_view(&mut self, src: &Canvas, sx: i32, sy: i32, fill: Color) {
        for y in 0..self.h {
            let yy = sy + y;
            let row = (y * self.w) as usize;
            if yy < 0 || yy >= src.h {
                self.px[row..row + self.w as usize].fill(fill);
                continue;
            }
            for x in 0..self.w {
                let xx = sx + x;
                self.px[row + x as usize] =
                    if xx < 0 || xx >= src.w { fill } else { src.px[(yy * src.w + xx) as usize] };
            }
        }
    }

    /// Blend the whole frame toward `c`.
    pub fn wash(&mut self, c: Color, a: u8) {
        for p in &mut self.px {
            *p = p.mix(c, a);
        }
    }

    /// RGB bytes, each logical pixel repeated `k` times in both axes.
    pub fn scaled_rgb(&self, k: usize, out: &mut Vec<u8>) {
        let (w, h) = (self.w as usize, self.h as usize);
        out.clear();
        out.reserve(w * h * k * k * 3);
        let mut row = Vec::with_capacity(w * k * 3);
        for y in 0..h {
            row.clear();
            for p in &self.px[y * w..(y + 1) * w] {
                for _ in 0..k {
                    row.extend_from_slice(&[p.r, p.g, p.b]);
                }
            }
            for _ in 0..k {
                out.extend_from_slice(&row);
            }
        }
    }
}
