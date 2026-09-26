//! Engine core: math, deterministic RNG, spatial grid.

pub mod grid;
pub mod math;
pub mod rng;

pub use grid::Grid;
pub use math::{Rect, Vec2, damp};
pub use rng::Rng;

/// Simulation rate. The sim always advances in steps of `DT`.
pub const TICK_HZ: u32 = 60;
pub const DT: f32 = 1.0 / TICK_HZ as f32;
