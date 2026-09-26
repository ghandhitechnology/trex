use super::math::{Rect, Vec2};

/// Uniform bucket grid over a fixed area, rebuilt every tick.
/// Points outside the area land in the nearest edge cell.
pub struct Grid {
    area: Rect,
    cell: f32,
    cols: usize,
    rows: usize,
    start: Vec<u32>,
    items: Vec<u32>,
    cell_of: Vec<u32>,
}

impl Grid {
    pub fn new(area: Rect, cell: f32) -> Self {
        let cols = (area.w / cell).ceil().max(1.0) as usize;
        let rows = (area.h / cell).ceil().max(1.0) as usize;
        Self {
            area,
            cell,
            cols,
            rows,
            start: vec![0; cols * rows + 1],
            items: Vec::new(),
            cell_of: Vec::new(),
        }
    }

    fn coords(&self, p: Vec2) -> (usize, usize) {
        let cx = ((p.x - self.area.x) / self.cell).floor();
        let cy = ((p.y - self.area.y) / self.cell).floor();
        (cx.clamp(0.0, (self.cols - 1) as f32) as usize, cy.clamp(0.0, (self.rows - 1) as f32) as usize)
    }

    /// Rebuild from positions; item ids are indices into `points`.
    pub fn rebuild(&mut self, points: impl Iterator<Item = Vec2>) {
        self.cell_of.clear();
        for p in points {
            let (cx, cy) = self.coords(p);
            self.cell_of.push((cy * self.cols + cx) as u32);
        }
        self.start.iter_mut().for_each(|s| *s = 0);
        for &c in &self.cell_of {
            self.start[c as usize + 1] += 1;
        }
        for i in 1..self.start.len() {
            self.start[i] += self.start[i - 1];
        }
        self.items.clear();
        self.items.resize(self.cell_of.len(), 0);
        let mut fill = self.start.clone();
        for (i, &c) in self.cell_of.iter().enumerate() {
            let slot = &mut fill[c as usize];
            self.items[*slot as usize] = i as u32;
            *slot += 1;
        }
    }

    /// Calls `f` with every item whose cell overlaps the circle's bounding box.
    pub fn query(&self, center: Vec2, radius: f32, mut f: impl FnMut(usize)) {
        if self.items.is_empty() {
            return;
        }
        let (x0, y0) = self.coords(center - Vec2::new(radius, radius));
        let (x1, y1) = self.coords(center + Vec2::new(radius, radius));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                let c = cy * self.cols + cx;
                for &i in &self.items[self.start[c] as usize..self.start[c + 1] as usize] {
                    f(i as usize);
                }
            }
        }
    }
}
