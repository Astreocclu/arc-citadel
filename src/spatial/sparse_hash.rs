//! Sparse hash grid for efficient spatial queries

use ahash::AHashMap;
use crate::core::types::{EntityId, Vec2};

/// Sparse hash grid for O(1) neighbor queries
pub struct SparseHashGrid {
    cell_size: f32,
    cells: AHashMap<(i32, i32), Vec<EntityId>>,
}

impl SparseHashGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: AHashMap::new(),
        }
    }

    #[inline]
    fn cell_coord(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, entity: EntityId, pos: Vec2) {
        let coord = self.cell_coord(pos);
        self.cells.entry(coord).or_default().push(entity);
    }

    pub fn remove(&mut self, entity: EntityId, pos: Vec2) {
        let coord = self.cell_coord(pos);
        if let Some(cell) = self.cells.get_mut(&coord) {
            cell.retain(|&e| e != entity);
        }
    }

    /// Query all entities in neighboring cells (3x3 neighborhood)
    pub fn query_neighbors(&self, pos: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        let (cx, cy) = self.cell_coord(pos);

        (-1..=1).flat_map(move |dx| {
            (-1..=1).flat_map(move |dy| {
                self.cells.get(&(cx + dx, cy + dy))
                    .into_iter()
                    .flatten()
                    .copied()
            })
        })
    }

    /// Query entities within radius
    pub fn query_radius(&self, center: Vec2, radius: f32, positions: &[Vec2]) -> Vec<EntityId> {
        let radius_sq = radius * radius;
        self.query_neighbors(center)
            .filter(|&entity| {
                let idx = entity.0.as_u128() as usize % positions.len();
                positions.get(idx)
                    .map(|pos| center.distance(pos) <= radius)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Rebuild grid from positions
    pub fn rebuild<'a>(&mut self, entities: impl Iterator<Item = (EntityId, Vec2)>) {
        self.clear();
        for (entity, pos) in entities {
            self.insert(entity, pos);
        }
    }
}
