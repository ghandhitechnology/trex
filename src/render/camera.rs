use crate::engine::{Rect, Rng, Vec2, damp};

/// Follows a target inside the arena bounds, with trauma-based screen shake.
pub struct Camera {
    pub center: Vec2,
    pub view: (i32, i32),
    trauma: f32,
    shake: Vec2,
    /// Clock for the shake noise.
    t: f32,
    /// Random phase so shakes differ between runs.
    seed: f32,
}

impl Camera {
    pub fn new(center: Vec2, view: (i32, i32)) -> Self {
        Self { center, view, trauma: 0.0, shake: Vec2::ZERO, t: 0.0, seed: 0.0 }
    }

    /// Add shake. Trauma is capped at 1; offset grows with trauma squared.
    pub fn kick(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }

    pub fn update(&mut self, target: Vec2, bounds: Rect, dt: f32, rng: &mut Rng) {
        self.center = self.center.lerp(target, damp(10.0, dt));
        let half = Vec2::new(self.view.0 as f32 / 2.0, self.view.1 as f32 / 2.0);
        let clamp_axis = |c: f32, lo: f32, size: f32, h: f32| {
            if size <= h * 2.0 { lo + size / 2.0 } else { c.clamp(lo + h, lo + size - h) }
        };
        self.center.x = clamp_axis(self.center.x, bounds.x - 16.0, bounds.w + 32.0, half.x);
        self.center.y = clamp_axis(self.center.y, bounds.y - 16.0, bounds.h + 32.0, half.y);
        // Smooth shake: layered sines instead of white noise, so a hit reads as
        // a jolt that rings out rather than per-frame jitter.
        if self.trauma <= 0.0 {
            self.seed = rng.range(0.0, 100.0);
        }
        self.t += dt;
        self.trauma = (self.trauma - dt * 1.9).max(0.0);
        let mag = 5.0 * self.trauma * self.trauma;
        let (t, s) = (self.t * 38.0, self.seed);
        let wob = |a: f32, b: f32| (t * a + s).sin() * 0.6 + (t * b + s * 1.7).sin() * 0.4;
        self.shake = Vec2::new(wob(1.0, 2.3), wob(1.3, 1.9)) * mag;
    }

    /// World coordinate at the top-left screen pixel.
    pub fn origin(&self) -> (i32, i32) {
        let c = self.center + self.shake;
        ((c.x - self.view.0 as f32 / 2.0).round() as i32, (c.y - self.view.1 as f32 / 2.0).round() as i32)
    }

    pub fn to_screen(&self, p: Vec2) -> (i32, i32) {
        let (ox, oy) = self.origin();
        (p.x.round() as i32 - ox, p.y.round() as i32 - oy)
    }

    /// True if a point (with margin) is visible.
    pub fn sees(&self, p: Vec2, margin: f32) -> bool {
        let half = Vec2::new(self.view.0 as f32 / 2.0 + margin, self.view.1 as f32 / 2.0 + margin);
        (p.x - self.center.x).abs() < half.x && (p.y - self.center.y).abs() < half.y
    }
}
