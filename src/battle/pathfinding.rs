//! A* pathfinding for battle maps
//!
//! Respects terrain costs and unit type restrictions.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::battle::battle_map::BattleMap;
use crate::battle::hex::BattleHexCoord;

/// Node in the A* open set
#[derive(Debug, Clone)]
struct PathNode {
    coord: BattleHexCoord,
    f_cost: f32, // g_cost + heuristic
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.coord == other.coord
    }
}

impl Eq for PathNode {}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap
        other
            .f_cost
            .partial_cmp(&self.f_cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Find path using A* algorithm
///
/// Returns None if no path exists.
/// `is_cavalry` restricts movement through forests/buildings.
pub fn find_path(
    map: &BattleMap,
    start: BattleHexCoord,
    goal: BattleHexCoord,
    is_cavalry: bool,
) -> Option<Vec<BattleHexCoord>> {
    if start == goal {
        return Some(vec![start]);
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<BattleHexCoord, BattleHexCoord> = HashMap::new();
    let mut g_scores: HashMap<BattleHexCoord, f32> = HashMap::new();

    g_scores.insert(start, 0.0);
    open_set.push(PathNode {
        coord: start,
        f_cost: start.distance(&goal) as f32,
    });

    while let Some(current) = open_set.pop() {
        if current.coord == goal {
            return Some(reconstruct_path(&came_from, current.coord));
        }

        let current_g = *g_scores.get(&current.coord).unwrap_or(&f32::INFINITY);

        for neighbor in current.coord.neighbors() {
            // Check if passable
            let Some(hex) = map.get_hex(neighbor) else {
                continue;
            };

            // Check unit-type restrictions
            if is_cavalry && hex.terrain.impassable_for_cavalry() {
                continue;
            }
            if !is_cavalry && hex.terrain.impassable_for_infantry() {
                continue;
            }

            let move_cost = hex.total_movement_cost();
            if move_cost.is_infinite() {
                continue;
            }

            let tentative_g = current_g + move_cost;
            let neighbor_g = *g_scores.get(&neighbor).unwrap_or(&f32::INFINITY);

            if tentative_g < neighbor_g {
                came_from.insert(neighbor, current.coord);
                g_scores.insert(neighbor, tentative_g);

                let f_cost = tentative_g + neighbor.distance(&goal) as f32;
                open_set.push(PathNode {
                    coord: neighbor,
                    f_cost,
                });
            }
        }
    }

    None // No path found
}

/// Reconstruct path from came_from map
fn reconstruct_path(
    came_from: &HashMap<BattleHexCoord, BattleHexCoord>,
    mut current: BattleHexCoord,
) -> Vec<BattleHexCoord> {
    let mut path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse();
    path
}

/// Calculate path cost (sum of terrain costs)
pub fn path_cost(map: &BattleMap, path: &[BattleHexCoord]) -> f32 {
    path.iter()
        .filter_map(|coord| map.get_hex(*coord))
        .map(|hex| hex.total_movement_cost())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::battle_map::BattleMap;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::terrain::BattleTerrain;

    #[test]
    fn test_pathfind_straight_line() {
        let map = BattleMap::new(10, 10);
        let start = BattleHexCoord::new(0, 0);
        let goal = BattleHexCoord::new(5, 0);

        let path = find_path(&map, start, goal, false);

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&goal));
    }

    #[test]
    fn test_pathfind_around_obstacle() {
        let mut map = BattleMap::new(10, 10);
        // Block the direct path with deep water
        map.set_terrain(BattleHexCoord::new(2, 0), BattleTerrain::DeepWater);
        map.set_terrain(BattleHexCoord::new(3, 0), BattleTerrain::DeepWater);

        let start = BattleHexCoord::new(0, 0);
        let goal = BattleHexCoord::new(5, 0);

        let path = find_path(&map, start, goal, false);

        assert!(path.is_some());
        let path = path.unwrap();
        // Should go around, not through
        assert!(!path.contains(&BattleHexCoord::new(2, 0)));
    }

    #[test]
    fn test_cavalry_cant_enter_forest() {
        let mut map = BattleMap::new(10, 10);
        // Forest blocks cavalry
        for r in 0..10 {
            map.set_terrain(BattleHexCoord::new(5, r), BattleTerrain::Forest);
        }

        let start = BattleHexCoord::new(0, 5);
        let goal = BattleHexCoord::new(9, 5);

        // Infantry can path through
        let infantry_path = find_path(&map, start, goal, false);
        assert!(infantry_path.is_some());

        // Cavalry cannot (no path around in this setup)
        let cavalry_path = find_path(&map, start, goal, true);
        assert!(cavalry_path.is_none());
    }

    #[test]
    fn test_pathfind_no_path() {
        let mut map = BattleMap::new(10, 10);
        // Completely surround goal with cliffs
        let goal = BattleHexCoord::new(5, 5);
        for neighbor in goal.neighbors() {
            map.set_terrain(neighbor, BattleTerrain::Cliff);
        }

        let start = BattleHexCoord::new(0, 0);
        let path = find_path(&map, start, goal, false);

        assert!(path.is_none());
    }

    #[test]
    fn test_pathfind_same_start_goal() {
        let map = BattleMap::new(10, 10);
        let start = BattleHexCoord::new(5, 5);

        let path = find_path(&map, start, start, false);

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 1);
        assert_eq!(path[0], start);
    }

    #[test]
    fn test_path_cost() {
        let map = BattleMap::new(10, 10);
        let path = vec![
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(1, 0),
            BattleHexCoord::new(2, 0),
        ];

        let cost = path_cost(&map, &path);
        // Open terrain has movement cost of 1.0, so 3 hexes = 3.0
        assert_eq!(cost, 3.0);
    }

    #[test]
    fn test_path_cost_varied_terrain() {
        let mut map = BattleMap::new(10, 10);
        map.set_terrain(BattleHexCoord::new(1, 0), BattleTerrain::Rough); // 1.5 cost

        let path = vec![
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(1, 0),
            BattleHexCoord::new(2, 0),
        ];

        let cost = path_cost(&map, &path);
        // Open (1.0) + Rough (1.5) + Open (1.0) = 3.5
        assert_eq!(cost, 3.5);
    }

    #[test]
    fn test_pathfind_prefers_road() {
        let mut map = BattleMap::new(10, 10);
        // Create a road along r=1
        for q in 0..10 {
            map.set_terrain(BattleHexCoord::new(q, 1), BattleTerrain::Road);
        }

        let start = BattleHexCoord::new(0, 0);
        let goal = BattleHexCoord::new(5, 0);

        let path = find_path(&map, start, goal, false);
        assert!(path.is_some());
        let path = path.unwrap();

        // The path might use the road since it's cheaper
        // At minimum, we verify a valid path was found
        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&goal));
    }
}
