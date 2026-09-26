use crate::engine::{Rect, Rng, Vec2, damp};

/// Follows a target inside the arena bounds, with trauma-based screen shake.
pub struct Camera {
    pub center: Vec2,
    pub view: (i32, i32),
    trauma: f32,
    shake: Vec2,
}

impl Camera {
    pub fn new(center: Vec2, view: (i32, i32)) -> Self {
        Self { center, view, trauma: 0.0, shake: Vec2::ZERO }
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
        self.center.x = clamp_axis(self.center.x, bounds.x - 12.0, bounds.w + 24.0, half.x);
        self.center.y = clamp_axis(self.center.y, bounds.y - 12.0, bounds.h + 24.0, half.y);
        self.trauma = (self.trauma - dt * 1.6).max(0.0);
        let mag = 6.0 * self.trauma * self.trauma;
        self.shake = Vec2::new(rng.range(-mag, mag), rng.range(-mag, mag));
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
