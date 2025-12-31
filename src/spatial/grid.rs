//! Generic grid for spatial data

use crate::core::types::Vec2;

/// Generic 2D grid with configurable cell size
#[derive(Debug, Clone)]
pub struct Grid<T: Clone + Default> {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub origin: Vec2,
    data: Vec<T>,
}

impl<T: Clone + Default> Grid<T> {
    pub fn new(width: usize, height: usize, cell_size: f32, origin: Vec2) -> Self {
        Self {
            width,
            height,
            cell_size,
            origin,
            data: vec![T::default(); width * height],
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x < self.width && y < self.height {
            Some(&self.data[y * self.width + x])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x < self.width && y < self.height {
            Some(&mut self.data[y * self.width + x])
        } else {
            None
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, value: T) {
        if x < self.width && y < self.height {
            self.data[y * self.width + x] = value;
        }
    }

    /// Convert world position to cell coordinates
    #[inline]
    pub fn world_to_cell(&self, pos: Vec2) -> (usize, usize) {
        let x = ((pos.x - self.origin.x) / self.cell_size).floor() as i32;
        let y = ((pos.y - self.origin.y) / self.cell_size).floor() as i32;
        (
            x.max(0).min(self.width as i32 - 1) as usize,
            y.max(0).min(self.height as i32 - 1) as usize,
        )
    }

    /// Sample grid at world position
    pub fn sample(&self, pos: Vec2) -> Option<&T> {
        let (x, y) = self.world_to_cell(pos);
        self.get(x, y)
    }

    /// Cell center in world coordinates
    pub fn cell_center(&self, x: usize, y: usize) -> Vec2 {
        Vec2::new(
            self.origin.x + (x as f32 + 0.5) * self.cell_size,
            self.origin.y + (y as f32 + 0.5) * self.cell_size,
        )
    }
}
