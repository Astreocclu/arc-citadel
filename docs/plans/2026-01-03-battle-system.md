# Battle System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a tactical battle system with hex-based maps, fog of war, courier-delayed orders, and multi-LOD combat resolution that integrates with the existing combat module.

**Architecture:** Battles occur on 20m hex grids with dense terrain. Vision is scarce (fog of war). Orders go through couriers (not instant). Combat resolves at multiple LODs (individual → element → unit → formation) using existing categorical property comparisons. Two victory paths: damage and morale.

**Tech Stack:** Rust, serde for serialization, existing SoA patterns, existing combat module types

**Philosophy (READ FIRST):**
- NO multiplicative stacking (all modifiers ADDITIVE)
- Dense terrain that CONSTRAINS movement
- Vision is SCARCE - information must be gathered
- Orders through COURIERS - not instant
- Same simulation at ALL accessibility levels

---

## Task 1: Battle Constants Module

**Files:**
- Create: `src/battle/constants.rs`
- Modify: `src/battle/mod.rs`

**Step 1: Write the failing test**

```rust
// In src/battle/constants.rs at bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_size_reasonable() {
        assert!(BATTLE_HEX_SIZE_METERS > 10.0 && BATTLE_HEX_SIZE_METERS < 50.0);
    }

    #[test]
    fn test_speed_ordering() {
        assert!(CAVALRY_CHARGE_SPEED > CAVALRY_TROT_SPEED);
        assert!(CAVALRY_TROT_SPEED > INFANTRY_RUN_SPEED);
        assert!(INFANTRY_RUN_SPEED > INFANTRY_WALK_SPEED);
    }

    #[test]
    fn test_vision_ranges_positive() {
        assert!(BASE_VISION_RANGE > 0);
        assert!(SCOUT_VISION_BONUS > 0);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::constants`
Expected: FAIL with "cannot find value `BATTLE_HEX_SIZE_METERS`"

**Step 3: Write minimal implementation**

```rust
//! Battle system constants - all tunable values in one place
//!
//! These values are ADDITIVE, never multiplicative. No percentage modifiers.

// Battle map scale
pub const BATTLE_HEX_SIZE_METERS: f32 = 20.0;
pub const DEFAULT_BATTLE_WIDTH: u32 = 50;
pub const DEFAULT_BATTLE_HEIGHT: u32 = 40;

// Time
pub const BATTLE_TICK_MS: u32 = 100;
pub const BATTLE_TICK_SIM_SECONDS: f32 = 1.0;
pub const MAX_BATTLE_TICKS: u64 = 6000; // 10 minutes

// Movement (hexes per tick)
pub const INFANTRY_WALK_SPEED: f32 = 0.05;
pub const INFANTRY_RUN_SPEED: f32 = 0.10;
pub const CAVALRY_WALK_SPEED: f32 = 0.10;
pub const CAVALRY_TROT_SPEED: f32 = 0.20;
pub const CAVALRY_CHARGE_SPEED: f32 = 0.40;
pub const COURIER_SPEED: f32 = 0.30;
pub const ROUT_SPEED: f32 = 0.15;

// Vision (hexes)
pub const BASE_VISION_RANGE: u32 = 8;
pub const SCOUT_VISION_BONUS: u32 = 4;
pub const ELEVATION_VISION_BONUS: u32 = 2;
pub const FOREST_VISION_PENALTY: u32 = 4;

// Combat rates (per tick) - ADDITIVE
pub const BASE_CASUALTY_RATE: f32 = 0.02;
pub const FATIGUE_RATE_COMBAT: f32 = 0.02;
pub const FATIGUE_RATE_MARCH: f32 = 0.005;
pub const FATIGUE_RECOVERY_RATE: f32 = 0.01;

// Stress - ADDITIVE thresholds
pub const CONTAGION_STRESS: f32 = 0.10;
pub const OFFICER_DEATH_STRESS: f32 = 0.30;
pub const FLANK_STRESS: f32 = 0.20;

// Courier
pub const COURIER_INTERCEPTION_RANGE: u32 = 2;
pub const COURIER_INTERCEPTION_CHANCE_PATROL: f32 = 0.5;
pub const COURIER_INTERCEPTION_CHANCE_ALERT: f32 = 0.7;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_size_reasonable() {
        assert!(BATTLE_HEX_SIZE_METERS > 10.0 && BATTLE_HEX_SIZE_METERS < 50.0);
    }

    #[test]
    fn test_speed_ordering() {
        assert!(CAVALRY_CHARGE_SPEED > CAVALRY_TROT_SPEED);
        assert!(CAVALRY_TROT_SPEED > INFANTRY_RUN_SPEED);
        assert!(INFANTRY_RUN_SPEED > INFANTRY_WALK_SPEED);
    }

    #[test]
    fn test_vision_ranges_positive() {
        assert!(BASE_VISION_RANGE > 0);
        assert!(SCOUT_VISION_BONUS > 0);
    }
}
```

**Step 4: Update mod.rs**

```rust
//! Battle system - tactical combat with fog of war and courier delays
//!
//! NOT Total War: terrain is dense, vision is scarce, control is delegated.

pub mod constants;
pub mod battle_map;
pub mod planning;
pub mod execution;
pub mod courier;
pub mod resolution;
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib battle::constants`
Expected: PASS (3 tests)

**Step 6: Commit**

```bash
git add src/battle/constants.rs src/battle/mod.rs
git commit -m "feat(battle): add battle constants module"
```

---

## Task 2: Battle Hex Coordinate System

**Files:**
- Create: `src/battle/hex.rs`
- Modify: `src/battle/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_coord_creation() {
        let coord = BattleHexCoord::new(5, 10);
        assert_eq!(coord.q, 5);
        assert_eq!(coord.r, 10);
    }

    #[test]
    fn test_hex_distance_same() {
        let a = BattleHexCoord::new(0, 0);
        assert_eq!(a.distance(&a), 0);
    }

    #[test]
    fn test_hex_distance_adjacent() {
        let a = BattleHexCoord::new(0, 0);
        let b = BattleHexCoord::new(1, 0);
        assert_eq!(a.distance(&b), 1);
    }

    #[test]
    fn test_hex_neighbors_count() {
        let coord = BattleHexCoord::new(5, 5);
        assert_eq!(coord.neighbors().len(), 6);
    }

    #[test]
    fn test_hex_line() {
        let a = BattleHexCoord::new(0, 0);
        let b = BattleHexCoord::new(3, 0);
        let line = a.line_to(&b);
        assert_eq!(line.len(), 4); // Includes start and end
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::hex`
Expected: FAIL with "cannot find type `BattleHexCoord`"

**Step 3: Write minimal implementation**

```rust
//! Hex coordinate system for battle maps (axial coordinates)
//!
//! Uses axial coordinates (q, r) for easy neighbor calculation.

use serde::{Deserialize, Serialize};

/// Axial hex coordinate for battle map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BattleHexCoord {
    pub q: i32,
    pub r: i32,
}

impl BattleHexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Cube coordinate S (derived from q and r)
    pub fn s(&self) -> i32 {
        -self.q - self.r
    }

    /// Manhattan distance in hex space
    pub fn distance(&self, other: &Self) -> u32 {
        let dq = (self.q - other.q).abs();
        let dr = (self.r - other.r).abs();
        let ds = (self.s() - other.s()).abs();
        ((dq + dr + ds) / 2) as u32
    }

    /// Get all 6 neighboring hex coordinates
    pub fn neighbors(&self) -> [BattleHexCoord; 6] {
        [
            BattleHexCoord::new(self.q + 1, self.r),
            BattleHexCoord::new(self.q + 1, self.r - 1),
            BattleHexCoord::new(self.q, self.r - 1),
            BattleHexCoord::new(self.q - 1, self.r),
            BattleHexCoord::new(self.q - 1, self.r + 1),
            BattleHexCoord::new(self.q, self.r + 1),
        ]
    }

    /// Get hex coordinates in a line from self to other (inclusive)
    pub fn line_to(&self, other: &Self) -> Vec<BattleHexCoord> {
        let n = self.distance(other) as i32;
        if n == 0 {
            return vec![*self];
        }

        let mut results = Vec::with_capacity((n + 1) as usize);
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let q = self.q as f32 + (other.q - self.q) as f32 * t;
            let r = self.r as f32 + (other.r - self.r) as f32 * t;
            results.push(Self::round(q, r));
        }
        results
    }

    /// Round floating point hex to nearest integer hex
    fn round(q: f32, r: f32) -> Self {
        let s = -q - r;
        let mut rq = q.round();
        let mut rr = r.round();
        let rs = s.round();

        let q_diff = (rq - q).abs();
        let r_diff = (rr - r).abs();
        let s_diff = (rs - s).abs();

        if q_diff > r_diff && q_diff > s_diff {
            rq = -rr - rs;
        } else if r_diff > s_diff {
            rr = -rq - rs;
        }

        Self::new(rq as i32, rr as i32)
    }

    /// Get all hexes within range (inclusive)
    pub fn hexes_in_range(&self, range: u32) -> Vec<BattleHexCoord> {
        let range = range as i32;
        let mut results = Vec::new();
        for q in -range..=range {
            for r in (-range).max(-q - range)..=range.min(-q + range) {
                results.push(BattleHexCoord::new(self.q + q, self.r + r));
            }
        }
        results
    }
}

/// Direction enum for hex facing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HexDirection {
    #[default]
    East,
    NorthEast,
    NorthWest,
    West,
    SouthWest,
    SouthEast,
}

impl HexDirection {
    /// Get the hex offset for this direction
    pub fn offset(&self) -> BattleHexCoord {
        match self {
            HexDirection::East => BattleHexCoord::new(1, 0),
            HexDirection::NorthEast => BattleHexCoord::new(1, -1),
            HexDirection::NorthWest => BattleHexCoord::new(0, -1),
            HexDirection::West => BattleHexCoord::new(-1, 0),
            HexDirection::SouthWest => BattleHexCoord::new(-1, 1),
            HexDirection::SouthEast => BattleHexCoord::new(0, 1),
        }
    }

    /// Get opposite direction
    pub fn opposite(&self) -> Self {
        match self {
            HexDirection::East => HexDirection::West,
            HexDirection::NorthEast => HexDirection::SouthWest,
            HexDirection::NorthWest => HexDirection::SouthEast,
            HexDirection::West => HexDirection::East,
            HexDirection::SouthWest => HexDirection::NorthEast,
            HexDirection::SouthEast => HexDirection::NorthWest,
        }
    }

    /// All directions
    pub fn all() -> [HexDirection; 6] {
        [
            HexDirection::East,
            HexDirection::NorthEast,
            HexDirection::NorthWest,
            HexDirection::West,
            HexDirection::SouthWest,
            HexDirection::SouthEast,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_coord_creation() {
        let coord = BattleHexCoord::new(5, 10);
        assert_eq!(coord.q, 5);
        assert_eq!(coord.r, 10);
    }

    #[test]
    fn test_hex_distance_same() {
        let a = BattleHexCoord::new(0, 0);
        assert_eq!(a.distance(&a), 0);
    }

    #[test]
    fn test_hex_distance_adjacent() {
        let a = BattleHexCoord::new(0, 0);
        let b = BattleHexCoord::new(1, 0);
        assert_eq!(a.distance(&b), 1);
    }

    #[test]
    fn test_hex_neighbors_count() {
        let coord = BattleHexCoord::new(5, 5);
        assert_eq!(coord.neighbors().len(), 6);
    }

    #[test]
    fn test_hex_line() {
        let a = BattleHexCoord::new(0, 0);
        let b = BattleHexCoord::new(3, 0);
        let line = a.line_to(&b);
        assert_eq!(line.len(), 4);
    }

    #[test]
    fn test_hexes_in_range() {
        let center = BattleHexCoord::new(0, 0);
        let range_1 = center.hexes_in_range(1);
        assert_eq!(range_1.len(), 7); // Center + 6 neighbors
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(HexDirection::East.opposite(), HexDirection::West);
        assert_eq!(HexDirection::NorthEast.opposite(), HexDirection::SouthWest);
    }
}
```

**Step 4: Update mod.rs**

```rust
//! Battle system - tactical combat with fog of war and courier delays
//!
//! NOT Total War: terrain is dense, vision is scarce, control is delegated.

pub mod constants;
pub mod hex;
pub mod battle_map;
pub mod planning;
pub mod execution;
pub mod courier;
pub mod resolution;

pub use hex::{BattleHexCoord, HexDirection};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib battle::hex`
Expected: PASS (7 tests)

**Step 6: Commit**

```bash
git add src/battle/hex.rs src/battle/mod.rs
git commit -m "feat(battle): add hex coordinate system"
```

---

## Task 3: Battle Terrain Types

**Files:**
- Create: `src/battle/terrain.rs`
- Modify: `src/battle/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_terrain_no_penalty() {
        assert_eq!(BattleTerrain::Open.movement_cost(), 1.0);
    }

    #[test]
    fn test_forest_blocks_los() {
        assert!(BattleTerrain::Forest.blocks_los());
        assert!(!BattleTerrain::Open.blocks_los());
    }

    #[test]
    fn test_rough_provides_cover() {
        assert!(BattleTerrain::Rough.cover_value() > BattleTerrain::Open.cover_value());
    }

    #[test]
    fn test_water_impassable_for_infantry() {
        assert!(BattleTerrain::DeepWater.impassable_for_infantry());
        assert!(!BattleTerrain::ShallowWater.impassable_for_infantry());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::terrain`
Expected: FAIL with "cannot find type `BattleTerrain`"

**Step 3: Write minimal implementation**

```rust
//! Battle terrain types and their effects
//!
//! Dense terrain constrains movement - navigation is a puzzle.

use serde::{Deserialize, Serialize};

/// Primary terrain type for a battle hex
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BattleTerrain {
    #[default]
    Open,           // No movement penalty, no cover
    Rough,          // Slight penalty, light cover
    Forest,         // Heavy penalty, heavy cover, blocks LOS
    ShallowWater,   // Moderate penalty, fordable
    DeepWater,      // Impassable for infantry
    Cliff,          // Impassable, high ground advantage
    Road,           // Movement bonus
    Building,       // Cover, can be occupied
}

impl BattleTerrain {
    /// Movement cost multiplier (1.0 = normal)
    pub fn movement_cost(&self) -> f32 {
        match self {
            BattleTerrain::Open => 1.0,
            BattleTerrain::Rough => 1.5,
            BattleTerrain::Forest => 2.5,
            BattleTerrain::ShallowWater => 2.0,
            BattleTerrain::DeepWater => f32::INFINITY, // Impassable
            BattleTerrain::Cliff => f32::INFINITY,     // Impassable
            BattleTerrain::Road => 0.7,                // Faster
            BattleTerrain::Building => 3.0,            // Slow to enter
        }
    }

    /// Does this terrain block line of sight?
    pub fn blocks_los(&self) -> bool {
        matches!(self, BattleTerrain::Forest | BattleTerrain::Building)
    }

    /// Cover value (0.0 = none, 1.0 = full)
    pub fn cover_value(&self) -> f32 {
        match self {
            BattleTerrain::Open => 0.0,
            BattleTerrain::Rough => 0.2,
            BattleTerrain::Forest => 0.5,
            BattleTerrain::ShallowWater => 0.0,
            BattleTerrain::DeepWater => 0.0,
            BattleTerrain::Cliff => 0.0,
            BattleTerrain::Road => 0.0,
            BattleTerrain::Building => 0.7,
        }
    }

    /// Is this terrain impassable for infantry?
    pub fn impassable_for_infantry(&self) -> bool {
        matches!(self, BattleTerrain::DeepWater | BattleTerrain::Cliff)
    }

    /// Is this terrain impassable for cavalry?
    pub fn impassable_for_cavalry(&self) -> bool {
        matches!(
            self,
            BattleTerrain::DeepWater
                | BattleTerrain::Cliff
                | BattleTerrain::Forest
                | BattleTerrain::Building
        )
    }

    /// Can units hide in this terrain?
    pub fn provides_concealment(&self) -> bool {
        matches!(self, BattleTerrain::Forest | BattleTerrain::Building)
    }
}

/// Terrain features that can exist on a hex (in addition to base terrain)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainFeature {
    Hill,       // Elevation, vision bonus
    Ridge,      // Blocks LOS from one side
    Stream,     // Minor obstacle
    Bridge,     // Chokepoint
    Wall,       // Defensive, can be breached
    Gate,       // Chokepoint, can be closed
    Tower,      // High ground, archer platform
    Treeline,   // Concealment at forest edge
}

impl TerrainFeature {
    /// Additional movement cost
    pub fn movement_cost_modifier(&self) -> f32 {
        match self {
            TerrainFeature::Hill => 0.3,
            TerrainFeature::Ridge => 0.2,
            TerrainFeature::Stream => 0.5,
            TerrainFeature::Bridge => 0.0,
            TerrainFeature::Wall => 1.0,   // Must climb
            TerrainFeature::Gate => 0.0,   // If open
            TerrainFeature::Tower => 0.0,  // Enter from ground
            TerrainFeature::Treeline => 0.2,
        }
    }

    /// Defense bonus (additive)
    pub fn defense_bonus(&self) -> f32 {
        match self {
            TerrainFeature::Hill => 0.1,
            TerrainFeature::Ridge => 0.2,
            TerrainFeature::Stream => 0.0,
            TerrainFeature::Bridge => 0.0,
            TerrainFeature::Wall => 0.4,
            TerrainFeature::Gate => 0.3,
            TerrainFeature::Tower => 0.5,
            TerrainFeature::Treeline => 0.1,
        }
    }

    /// Does this block LOS?
    pub fn blocks_los(&self) -> bool {
        matches!(self, TerrainFeature::Ridge | TerrainFeature::Wall)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_terrain_no_penalty() {
        assert_eq!(BattleTerrain::Open.movement_cost(), 1.0);
    }

    #[test]
    fn test_forest_blocks_los() {
        assert!(BattleTerrain::Forest.blocks_los());
        assert!(!BattleTerrain::Open.blocks_los());
    }

    #[test]
    fn test_rough_provides_cover() {
        assert!(BattleTerrain::Rough.cover_value() > BattleTerrain::Open.cover_value());
    }

    #[test]
    fn test_water_impassable_for_infantry() {
        assert!(BattleTerrain::DeepWater.impassable_for_infantry());
        assert!(!BattleTerrain::ShallowWater.impassable_for_infantry());
    }

    #[test]
    fn test_cavalry_cant_enter_forest() {
        assert!(BattleTerrain::Forest.impassable_for_cavalry());
        assert!(!BattleTerrain::Open.impassable_for_cavalry());
    }

    #[test]
    fn test_road_faster_than_open() {
        assert!(BattleTerrain::Road.movement_cost() < BattleTerrain::Open.movement_cost());
    }

    #[test]
    fn test_feature_defense_bonuses() {
        assert!(TerrainFeature::Wall.defense_bonus() > TerrainFeature::Hill.defense_bonus());
    }
}
```

**Step 4: Update mod.rs**

```rust
//! Battle system - tactical combat with fog of war and courier delays
//!
//! NOT Total War: terrain is dense, vision is scarce, control is delegated.

pub mod constants;
pub mod hex;
pub mod terrain;
pub mod battle_map;
pub mod planning;
pub mod execution;
pub mod courier;
pub mod resolution;

pub use hex::{BattleHexCoord, HexDirection};
pub use terrain::{BattleTerrain, TerrainFeature};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib battle::terrain`
Expected: PASS (7 tests)

**Step 6: Commit**

```bash
git add src/battle/terrain.rs src/battle/mod.rs
git commit -m "feat(battle): add terrain types with movement and cover effects"
```

---

## Task 4: BattleHex and BattleMap

**Files:**
- Replace: `src/battle/battle_map.rs`
- Modify: `src/battle/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battle_hex_creation() {
        let hex = BattleHex::new(BattleHexCoord::new(0, 0), BattleTerrain::Open);
        assert_eq!(hex.terrain, BattleTerrain::Open);
        assert_eq!(hex.elevation, 0);
    }

    #[test]
    fn test_battle_map_creation() {
        let map = BattleMap::new(10, 10);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
    }

    #[test]
    fn test_battle_map_get_hex() {
        let map = BattleMap::new(10, 10);
        let hex = map.get_hex(BattleHexCoord::new(5, 5));
        assert!(hex.is_some());
    }

    #[test]
    fn test_battle_map_out_of_bounds() {
        let map = BattleMap::new(10, 10);
        let hex = map.get_hex(BattleHexCoord::new(100, 100));
        assert!(hex.is_none());
    }

    #[test]
    fn test_line_of_sight_open() {
        let map = BattleMap::new(10, 10);
        let from = BattleHexCoord::new(0, 0);
        let to = BattleHexCoord::new(5, 0);
        assert!(map.has_line_of_sight(from, to));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::battle_map`
Expected: FAIL with "cannot find type `BattleHex`"

**Step 3: Write minimal implementation**

```rust
//! Battle map with hex grid, terrain, and line of sight
//!
//! Maps are DENSE with terrain - navigation is a puzzle, not optional.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::battle::hex::BattleHexCoord;
use crate::battle::terrain::{BattleTerrain, TerrainFeature};
use crate::core::types::EntityId;

/// Visibility state for fog of war
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VisibilityState {
    #[default]
    Unknown,    // Never seen
    Remembered, // Seen before, not currently observed
    Observed,   // Currently visible
}

/// A single hex on the battle map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleHex {
    pub coord: BattleHexCoord,
    pub terrain: BattleTerrain,
    pub elevation: i8,
    pub features: Vec<TerrainFeature>,
    pub occupants: Vec<EntityId>,
    pub visibility: VisibilityState,
}

impl BattleHex {
    pub fn new(coord: BattleHexCoord, terrain: BattleTerrain) -> Self {
        Self {
            coord,
            terrain,
            elevation: 0,
            features: Vec::new(),
            occupants: Vec::new(),
            visibility: VisibilityState::Unknown,
        }
    }

    /// Total movement cost including features
    pub fn total_movement_cost(&self) -> f32 {
        let base = self.terrain.movement_cost();
        let feature_cost: f32 = self.features.iter().map(|f| f.movement_cost_modifier()).sum();
        base + feature_cost
    }

    /// Total cover value including features
    pub fn total_cover(&self) -> f32 {
        let base = self.terrain.cover_value();
        let feature_cover: f32 = self.features.iter().map(|f| f.defense_bonus()).sum();
        (base + feature_cover).min(1.0) // Cap at 1.0
    }

    /// Does this hex block line of sight?
    pub fn blocks_los(&self) -> bool {
        self.terrain.blocks_los() || self.features.iter().any(|f| f.blocks_los())
    }
}

/// Objective on the battle map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub coord: BattleHexCoord,
    pub name: String,
    pub required_for_victory: bool,
}

/// The full battle map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleMap {
    pub hexes: HashMap<BattleHexCoord, BattleHex>,
    pub width: u32,
    pub height: u32,
    pub friendly_deployment: Vec<BattleHexCoord>,
    pub enemy_deployment: Vec<BattleHexCoord>,
    pub objectives: Vec<Objective>,
}

impl BattleMap {
    /// Create a new battle map with open terrain
    pub fn new(width: u32, height: u32) -> Self {
        let mut hexes = HashMap::new();

        for q in 0..width as i32 {
            for r in 0..height as i32 {
                let coord = BattleHexCoord::new(q, r);
                hexes.insert(coord, BattleHex::new(coord, BattleTerrain::Open));
            }
        }

        Self {
            hexes,
            width,
            height,
            friendly_deployment: Vec::new(),
            enemy_deployment: Vec::new(),
            objectives: Vec::new(),
        }
    }

    /// Get a hex at the given coordinate
    pub fn get_hex(&self, coord: BattleHexCoord) -> Option<&BattleHex> {
        self.hexes.get(&coord)
    }

    /// Get a mutable hex at the given coordinate
    pub fn get_hex_mut(&mut self, coord: BattleHexCoord) -> Option<&mut BattleHex> {
        self.hexes.get_mut(&coord)
    }

    /// Check if coordinate is within map bounds
    pub fn in_bounds(&self, coord: BattleHexCoord) -> bool {
        coord.q >= 0
            && coord.r >= 0
            && coord.q < self.width as i32
            && coord.r < self.height as i32
    }

    /// Check line of sight between two hexes
    pub fn has_line_of_sight(&self, from: BattleHexCoord, to: BattleHexCoord) -> bool {
        let line = from.line_to(&to);

        // Check all hexes except start and end
        for coord in line.iter().skip(1).take(line.len().saturating_sub(2)) {
            if let Some(hex) = self.get_hex(*coord) {
                if hex.blocks_los() {
                    return false;
                }
            }
        }

        true
    }

    /// Set terrain at a coordinate
    pub fn set_terrain(&mut self, coord: BattleHexCoord, terrain: BattleTerrain) {
        if let Some(hex) = self.get_hex_mut(coord) {
            hex.terrain = terrain;
        }
    }

    /// Set elevation at a coordinate
    pub fn set_elevation(&mut self, coord: BattleHexCoord, elevation: i8) {
        if let Some(hex) = self.get_hex_mut(coord) {
            hex.elevation = elevation;
        }
    }

    /// Add a feature at a coordinate
    pub fn add_feature(&mut self, coord: BattleHexCoord, feature: TerrainFeature) {
        if let Some(hex) = self.get_hex_mut(coord) {
            if !hex.features.contains(&feature) {
                hex.features.push(feature);
            }
        }
    }

    /// Get elevation difference (positive = from is higher)
    pub fn elevation_difference(&self, from: BattleHexCoord, to: BattleHexCoord) -> i8 {
        let from_elev = self.get_hex(from).map(|h| h.elevation).unwrap_or(0);
        let to_elev = self.get_hex(to).map(|h| h.elevation).unwrap_or(0);
        from_elev - to_elev
    }

    /// Get all hexes visible from a position with given range
    pub fn visible_hexes(&self, from: BattleHexCoord, range: u32) -> Vec<BattleHexCoord> {
        from.hexes_in_range(range)
            .into_iter()
            .filter(|coord| self.in_bounds(*coord) && self.has_line_of_sight(from, *coord))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battle_hex_creation() {
        let hex = BattleHex::new(BattleHexCoord::new(0, 0), BattleTerrain::Open);
        assert_eq!(hex.terrain, BattleTerrain::Open);
        assert_eq!(hex.elevation, 0);
    }

    #[test]
    fn test_battle_map_creation() {
        let map = BattleMap::new(10, 10);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
    }

    #[test]
    fn test_battle_map_get_hex() {
        let map = BattleMap::new(10, 10);
        let hex = map.get_hex(BattleHexCoord::new(5, 5));
        assert!(hex.is_some());
    }

    #[test]
    fn test_battle_map_out_of_bounds() {
        let map = BattleMap::new(10, 10);
        let hex = map.get_hex(BattleHexCoord::new(100, 100));
        assert!(hex.is_none());
    }

    #[test]
    fn test_line_of_sight_open() {
        let map = BattleMap::new(10, 10);
        let from = BattleHexCoord::new(0, 0);
        let to = BattleHexCoord::new(5, 0);
        assert!(map.has_line_of_sight(from, to));
    }

    #[test]
    fn test_line_of_sight_blocked_by_forest() {
        let mut map = BattleMap::new(10, 10);
        map.set_terrain(BattleHexCoord::new(2, 0), BattleTerrain::Forest);

        let from = BattleHexCoord::new(0, 0);
        let to = BattleHexCoord::new(5, 0);
        assert!(!map.has_line_of_sight(from, to));
    }

    #[test]
    fn test_elevation_difference() {
        let mut map = BattleMap::new(10, 10);
        map.set_elevation(BattleHexCoord::new(0, 0), 3);
        map.set_elevation(BattleHexCoord::new(5, 5), 1);

        let diff = map.elevation_difference(BattleHexCoord::new(0, 0), BattleHexCoord::new(5, 5));
        assert_eq!(diff, 2); // 3 - 1
    }

    #[test]
    fn test_total_movement_cost_with_feature() {
        let mut hex = BattleHex::new(BattleHexCoord::new(0, 0), BattleTerrain::Open);
        let base_cost = hex.total_movement_cost();

        hex.features.push(TerrainFeature::Hill);
        let with_hill = hex.total_movement_cost();

        assert!(with_hill > base_cost);
    }
}
```

**Step 4: Update mod.rs**

```rust
//! Battle system - tactical combat with fog of war and courier delays
//!
//! NOT Total War: terrain is dense, vision is scarce, control is delegated.

pub mod constants;
pub mod hex;
pub mod terrain;
pub mod battle_map;
pub mod planning;
pub mod execution;
pub mod courier;
pub mod resolution;

pub use hex::{BattleHexCoord, HexDirection};
pub use terrain::{BattleTerrain, TerrainFeature};
pub use battle_map::{BattleHex, BattleMap, VisibilityState, Objective};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib battle::battle_map`
Expected: PASS (8 tests)

**Step 6: Commit**

```bash
git add src/battle/battle_map.rs src/battle/mod.rs
git commit -m "feat(battle): add BattleHex and BattleMap with LOS"
```

---

## Task 5: Unit Types and Properties

**Files:**
- Create: `src/battle/unit_type.rs`
- Modify: `src/battle/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heavy_infantry_slow() {
        let props = UnitType::HeavyInfantry.default_properties();
        assert!(props.movement_speed < 1.0);
    }

    #[test]
    fn test_light_cavalry_fast() {
        let props = UnitType::LightCavalry.default_properties();
        assert!(props.movement_speed > 1.5);
    }

    #[test]
    fn test_scouts_good_vision() {
        let props = UnitType::Scouts.default_properties();
        assert!(props.vision_range > 8);
    }

    #[test]
    fn test_cavalry_is_mounted() {
        assert!(UnitType::LightCavalry.is_mounted());
        assert!(UnitType::HeavyCavalry.is_mounted());
        assert!(!UnitType::Infantry.is_mounted());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::unit_type`
Expected: FAIL with "cannot find type `UnitType`"

**Step 3: Write minimal implementation**

```rust
//! Unit types and their default properties
//!
//! Unit properties emerge from equipment aggregation.

use serde::{Deserialize, Serialize};
use crate::combat::{WeaponProperties, ArmorProperties, Edge, Mass, Reach, Rigidity, Padding, Coverage};

/// Type of military unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    // Infantry
    Levy,           // Cheap, unreliable
    Infantry,       // Standard foot soldiers
    HeavyInfantry,  // Armored, slow, tough
    Spearmen,       // Anti-cavalry, defensive
    Archers,        // Ranged, vulnerable in melee
    Crossbowmen,    // Slower, more punch

    // Cavalry
    LightCavalry,   // Fast, scout, skirmish
    Cavalry,        // Standard mounted
    HeavyCavalry,   // Shock, armored, expensive
    HorseArchers,   // Mobile ranged

    // Special
    Engineers,      // Siege, construction
    Scouts,         // Reconnaissance
    Command,        // Officers, messengers
}

/// Default properties for a unit type
#[derive(Debug, Clone)]
pub struct UnitProperties {
    pub avg_weapon: WeaponProperties,
    pub avg_armor: ArmorProperties,
    pub movement_speed: f32,       // Relative to baseline
    pub vision_range: u32,         // In hexes
    pub base_stress_threshold: f32,
    pub can_charge: bool,
    pub can_skirmish: bool,
}

impl UnitType {
    /// Get default properties for this unit type
    pub fn default_properties(&self) -> UnitProperties {
        match self {
            UnitType::Levy => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Light,
                    reach: Reach::Short,
                    special: vec![],
                },
                avg_armor: ArmorProperties {
                    rigidity: Rigidity::Cloth,
                    padding: Padding::None,
                    coverage: Coverage::None,
                },
                movement_speed: 1.0,
                vision_range: 6,
                base_stress_threshold: 0.6, // Break easily
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::Infantry => UnitProperties {
                avg_weapon: WeaponProperties::sword(),
                avg_armor: ArmorProperties::leather(),
                movement_speed: 1.0,
                vision_range: 6,
                base_stress_threshold: 1.0,
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::HeavyInfantry => UnitProperties {
                avg_weapon: WeaponProperties::sword(),
                avg_armor: ArmorProperties::plate(),
                movement_speed: 0.7,  // Slow
                vision_range: 5,      // Helmet limits vision
                base_stress_threshold: 1.2, // Harder to break
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::Spearmen => UnitProperties {
                avg_weapon: WeaponProperties::spear(),
                avg_armor: ArmorProperties::mail(),
                movement_speed: 0.9,
                vision_range: 6,
                base_stress_threshold: 1.0,
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::Archers => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Light,
                    reach: Reach::Grapple, // Melee is weak
                    special: vec![],
                },
                avg_armor: ArmorProperties::leather(),
                movement_speed: 1.0,
                vision_range: 10, // Good eyes
                base_stress_threshold: 0.8, // Vulnerable
                can_charge: false,
                can_skirmish: true,
            },

            UnitType::Crossbowmen => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Medium,
                    reach: Reach::Grapple,
                    special: vec![],
                },
                avg_armor: ArmorProperties::mail(),
                movement_speed: 0.9,
                vision_range: 8,
                base_stress_threshold: 0.9,
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::LightCavalry => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Light,
                    reach: Reach::Medium,
                    special: vec![],
                },
                avg_armor: ArmorProperties::leather(),
                movement_speed: 2.0, // Fast
                vision_range: 10,    // Good scouts
                base_stress_threshold: 0.8,
                can_charge: true,
                can_skirmish: true,
            },

            UnitType::Cavalry => UnitProperties {
                avg_weapon: WeaponProperties::sword(),
                avg_armor: ArmorProperties::mail(),
                movement_speed: 1.8,
                vision_range: 8,
                base_stress_threshold: 1.0,
                can_charge: true,
                can_skirmish: false,
            },

            UnitType::HeavyCavalry => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Heavy,
                    reach: Reach::Medium,
                    special: vec![],
                },
                avg_armor: ArmorProperties::plate(),
                movement_speed: 1.5,
                vision_range: 6,
                base_stress_threshold: 1.3, // Elite
                can_charge: true,
                can_skirmish: false,
            },

            UnitType::HorseArchers => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Light,
                    reach: Reach::Grapple,
                    special: vec![],
                },
                avg_armor: ArmorProperties::leather(),
                movement_speed: 2.0,
                vision_range: 10,
                base_stress_threshold: 0.8,
                can_charge: false,
                can_skirmish: true,
            },

            UnitType::Engineers => UnitProperties {
                avg_weapon: WeaponProperties::fists(),
                avg_armor: ArmorProperties::none(),
                movement_speed: 0.8,
                vision_range: 6,
                base_stress_threshold: 0.7,
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::Scouts => UnitProperties {
                avg_weapon: WeaponProperties::dagger(),
                avg_armor: ArmorProperties::leather(),
                movement_speed: 1.5,
                vision_range: 12, // Best vision
                base_stress_threshold: 0.7,
                can_charge: false,
                can_skirmish: true,
            },

            UnitType::Command => UnitProperties {
                avg_weapon: WeaponProperties::sword(),
                avg_armor: ArmorProperties::mail(),
                movement_speed: 1.5, // Mounted
                vision_range: 8,
                base_stress_threshold: 1.2, // Leaders
                can_charge: false,
                can_skirmish: false,
            },
        }
    }

    /// Is this a mounted unit?
    pub fn is_mounted(&self) -> bool {
        matches!(
            self,
            UnitType::LightCavalry
                | UnitType::Cavalry
                | UnitType::HeavyCavalry
                | UnitType::HorseArchers
                | UnitType::Command
        )
    }

    /// Is this a ranged unit?
    pub fn is_ranged(&self) -> bool {
        matches!(
            self,
            UnitType::Archers | UnitType::Crossbowmen | UnitType::HorseArchers
        )
    }

    /// Can this unit receive cavalry charge bonus?
    pub fn can_charge(&self) -> bool {
        self.default_properties().can_charge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heavy_infantry_slow() {
        let props = UnitType::HeavyInfantry.default_properties();
        assert!(props.movement_speed < 1.0);
    }

    #[test]
    fn test_light_cavalry_fast() {
        let props = UnitType::LightCavalry.default_properties();
        assert!(props.movement_speed > 1.5);
    }

    #[test]
    fn test_scouts_good_vision() {
        let props = UnitType::Scouts.default_properties();
        assert!(props.vision_range > 8);
    }

    #[test]
    fn test_cavalry_is_mounted() {
        assert!(UnitType::LightCavalry.is_mounted());
        assert!(UnitType::HeavyCavalry.is_mounted());
        assert!(!UnitType::Infantry.is_mounted());
    }

    #[test]
    fn test_archers_are_ranged() {
        assert!(UnitType::Archers.is_ranged());
        assert!(UnitType::Crossbowmen.is_ranged());
        assert!(!UnitType::Infantry.is_ranged());
    }

    #[test]
    fn test_spearmen_have_reach() {
        let props = UnitType::Spearmen.default_properties();
        assert_eq!(props.avg_weapon.reach, Reach::Long);
    }
}
```

**Step 4: Update mod.rs**

```rust
//! Battle system - tactical combat with fog of war and courier delays
//!
//! NOT Total War: terrain is dense, vision is scarce, control is delegated.

pub mod constants;
pub mod hex;
pub mod terrain;
pub mod battle_map;
pub mod unit_type;
pub mod planning;
pub mod execution;
pub mod courier;
pub mod resolution;

pub use hex::{BattleHexCoord, HexDirection};
pub use terrain::{BattleTerrain, TerrainFeature};
pub use battle_map::{BattleHex, BattleMap, VisibilityState, Objective};
pub use unit_type::{UnitType, UnitProperties};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib battle::unit_type`
Expected: PASS (6 tests)

**Step 6: Commit**

```bash
git add src/battle/unit_type.rs src/battle/mod.rs
git commit -m "feat(battle): add unit types with default properties"
```

---

## Task 6: Unit Hierarchy (Element, Unit, Formation, Army)

**Files:**
- Create: `src/battle/units.rs`
- Modify: `src/battle/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let element = Element::new(vec![EntityId::new(); 5]);
        assert_eq!(element.entities.len(), 5);
    }

    #[test]
    fn test_unit_strength() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 10]));
        assert_eq!(unit.strength(), 10);
    }

    #[test]
    fn test_formation_total_strength() {
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 20]));
        formation.units.push(unit);
        assert_eq!(formation.total_strength(), 20);
    }

    #[test]
    fn test_army_creation() {
        let army = Army::new(ArmyId::new(), EntityId::new());
        assert!(army.formations.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::units`
Expected: FAIL with "cannot find type `Element`"

**Step 3: Write minimal implementation**

```rust
//! Unit hierarchy: Element → Unit → Formation → Army
//!
//! Elements are the smallest tactical grouping (5-10 individuals).
//! Units combine elements into cohesive fighting groups.
//! Formations organize units under a commander.
//! Armies combine formations for a battle.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::battle::hex::{BattleHexCoord, HexDirection};
use crate::battle::unit_type::UnitType;
use crate::core::types::EntityId;

/// Unique identifier for armies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArmyId(pub Uuid);

impl ArmyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ArmyId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for formations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FormationId(pub Uuid);

impl FormationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FormationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitId(pub Uuid);

impl UnitId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UnitId {
    fn default() -> Self {
        Self::new()
    }
}

/// Smallest tactical grouping (5-10 individuals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub entities: Vec<EntityId>,
}

impl Element {
    pub fn new(entities: Vec<EntityId>) -> Self {
        Self { entities }
    }

    pub fn strength(&self) -> usize {
        self.entities.len()
    }
}

/// Combat stance for units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UnitStance {
    #[default]
    Formed,     // In formation, ready
    Moving,     // Moving to position
    Engaged,    // In combat
    Shaken,     // Morale damaged
    Routing,    // Fleeing
    Rallying,   // Reforming after rout
    Patrol,     // Scouting stance
    Alert,      // High awareness
}

/// Formation shape for units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormationShape {
    Line { depth: u8 },
    Column { width: u8 },
    Wedge { angle: f32 },
    Square,
    Skirmish { dispersion: f32 },
}

impl Default for FormationShape {
    fn default() -> Self {
        FormationShape::Line { depth: 2 }
    }
}

/// A military unit (collection of elements)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleUnit {
    pub id: UnitId,
    pub leader: Option<EntityId>,
    pub elements: Vec<Element>,
    pub unit_type: UnitType,

    // Position
    pub position: BattleHexCoord,
    pub facing: HexDirection,

    // State
    pub stance: UnitStance,
    pub formation_shape: FormationShape,
    pub cohesion: f32,   // 0.0 (scattered) to 1.0 (tight)
    pub fatigue: f32,    // 0.0 (fresh) to 1.0 (exhausted)
    pub stress: f32,     // Accumulated stress

    // Casualties
    pub casualties: u32,
}

impl BattleUnit {
    pub fn new(id: UnitId, unit_type: UnitType) -> Self {
        Self {
            id,
            leader: None,
            elements: Vec::new(),
            unit_type,
            position: BattleHexCoord::default(),
            facing: HexDirection::default(),
            stance: UnitStance::default(),
            formation_shape: FormationShape::default(),
            cohesion: 1.0,
            fatigue: 0.0,
            stress: 0.0,
            casualties: 0,
        }
    }

    /// Total strength (number of combatants)
    pub fn strength(&self) -> usize {
        self.elements.iter().map(|e| e.strength()).sum()
    }

    /// Effective strength (accounting for casualties)
    pub fn effective_strength(&self) -> usize {
        self.strength().saturating_sub(self.casualties as usize)
    }

    /// Is this unit broken?
    pub fn is_broken(&self) -> bool {
        matches!(self.stance, UnitStance::Routing)
    }

    /// Can this unit fight?
    pub fn can_fight(&self) -> bool {
        !matches!(self.stance, UnitStance::Routing | UnitStance::Rallying)
            && self.effective_strength() > 0
    }

    /// Is this unit engaged in combat?
    pub fn is_engaged(&self) -> bool {
        matches!(self.stance, UnitStance::Engaged)
    }

    /// Get stress threshold based on unit type and state
    pub fn stress_threshold(&self) -> f32 {
        let base = self.unit_type.default_properties().base_stress_threshold;

        // Additive modifiers
        let mut threshold = base;

        // High cohesion helps
        if self.cohesion > 0.8 {
            threshold += 0.1;
        }

        // Fatigue hurts
        threshold -= self.fatigue * 0.2;

        threshold.max(0.3)
    }
}

/// A formation (collection of units under a commander)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleFormation {
    pub id: FormationId,
    pub commander: EntityId,
    pub units: Vec<BattleUnit>,
    pub name: String,
}

impl BattleFormation {
    pub fn new(id: FormationId, commander: EntityId) -> Self {
        Self {
            id,
            commander,
            units: Vec::new(),
            name: String::new(),
        }
    }

    /// Total strength of all units
    pub fn total_strength(&self) -> usize {
        self.units.iter().map(|u| u.strength()).sum()
    }

    /// Effective strength of all units
    pub fn effective_strength(&self) -> usize {
        self.units.iter().map(|u| u.effective_strength()).sum()
    }

    /// Percentage of formation routing
    pub fn percentage_routing(&self) -> f32 {
        if self.units.is_empty() {
            return 0.0;
        }
        let routing = self.units.iter().filter(|u| u.is_broken()).count();
        routing as f32 / self.units.len() as f32
    }

    /// Is this formation broken?
    pub fn is_broken(&self) -> bool {
        self.percentage_routing() >= 0.5
    }
}

/// An army (collection of formations for a battle)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Army {
    pub id: ArmyId,
    pub commander: EntityId,
    pub formations: Vec<BattleFormation>,
    pub hq_position: BattleHexCoord,
    pub courier_pool: Vec<EntityId>,
}

impl Army {
    pub fn new(id: ArmyId, commander: EntityId) -> Self {
        Self {
            id,
            commander,
            formations: Vec::new(),
            hq_position: BattleHexCoord::default(),
            courier_pool: Vec::new(),
        }
    }

    /// Total strength of the army
    pub fn total_strength(&self) -> usize {
        self.formations.iter().map(|f| f.total_strength()).sum()
    }

    /// Effective strength of the army
    pub fn effective_strength(&self) -> usize {
        self.formations.iter().map(|f| f.effective_strength()).sum()
    }

    /// Percentage of army routing
    pub fn percentage_routing(&self) -> f32 {
        if self.formations.is_empty() {
            return 0.0;
        }

        let total_units: usize = self.formations.iter().map(|f| f.units.len()).sum();
        if total_units == 0 {
            return 0.0;
        }

        let routing_units: usize = self
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .filter(|u| u.is_broken())
            .count();

        routing_units as f32 / total_units as f32
    }

    /// Get a unit by ID
    pub fn get_unit(&self, unit_id: UnitId) -> Option<&BattleUnit> {
        self.formations
            .iter()
            .flat_map(|f| f.units.iter())
            .find(|u| u.id == unit_id)
    }

    /// Get a mutable unit by ID
    pub fn get_unit_mut(&mut self, unit_id: UnitId) -> Option<&mut BattleUnit> {
        self.formations
            .iter_mut()
            .flat_map(|f| f.units.iter_mut())
            .find(|u| u.id == unit_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let element = Element::new(vec![EntityId::new(); 5]);
        assert_eq!(element.entities.len(), 5);
    }

    #[test]
    fn test_unit_strength() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 10]));
        assert_eq!(unit.strength(), 10);
    }

    #[test]
    fn test_formation_total_strength() {
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 20]));
        formation.units.push(unit);
        assert_eq!(formation.total_strength(), 20);
    }

    #[test]
    fn test_army_creation() {
        let army = Army::new(ArmyId::new(), EntityId::new());
        assert!(army.formations.is_empty());
    }

    #[test]
    fn test_unit_effective_strength_with_casualties() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 100]));
        unit.casualties = 30;
        assert_eq!(unit.effective_strength(), 70);
    }

    #[test]
    fn test_unit_broken_when_routing() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        assert!(!unit.is_broken());

        unit.stance = UnitStance::Routing;
        assert!(unit.is_broken());
    }

    #[test]
    fn test_formation_broken_threshold() {
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());

        for _ in 0..10 {
            formation.units.push(BattleUnit::new(UnitId::new(), UnitType::Infantry));
        }

        // Break 4 units (40%) - not broken
        for i in 0..4 {
            formation.units[i].stance = UnitStance::Routing;
        }
        assert!(!formation.is_broken());

        // Break 5th unit (50%) - now broken
        formation.units[4].stance = UnitStance::Routing;
        assert!(formation.is_broken());
    }
}
```

**Step 4: Update mod.rs**

```rust
//! Battle system - tactical combat with fog of war and courier delays
//!
//! NOT Total War: terrain is dense, vision is scarce, control is delegated.

pub mod constants;
pub mod hex;
pub mod terrain;
pub mod battle_map;
pub mod unit_type;
pub mod units;
pub mod planning;
pub mod execution;
pub mod courier;
pub mod resolution;

pub use hex::{BattleHexCoord, HexDirection};
pub use terrain::{BattleTerrain, TerrainFeature};
pub use battle_map::{BattleHex, BattleMap, VisibilityState, Objective};
pub use unit_type::{UnitType, UnitProperties};
pub use units::{
    ArmyId, FormationId, UnitId,
    Element, BattleUnit, BattleFormation, Army,
    UnitStance, FormationShape,
};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib battle::units`
Expected: PASS (7 tests)

**Step 6: Commit**

```bash
git add src/battle/units.rs src/battle/mod.rs
git commit -m "feat(battle): add unit hierarchy (Element, Unit, Formation, Army)"
```

---

## Task 7: Battle Planning Structures

**Files:**
- Replace: `src/battle/planning.rs`
- Modify: `src/battle/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waypoint_creation() {
        let wp = Waypoint::new(BattleHexCoord::new(5, 5), WaypointBehavior::MoveTo);
        assert_eq!(wp.position, BattleHexCoord::new(5, 5));
    }

    #[test]
    fn test_go_code_creation() {
        let go_code = GoCode::new("HAMMER".into(), GoCodeTrigger::Manual);
        assert_eq!(go_code.name, "HAMMER");
    }

    #[test]
    fn test_battle_plan_add_deployment() {
        let mut plan = BattlePlan::new();
        let deployment = UnitDeployment {
            unit_id: UnitId::new(),
            position: BattleHexCoord::new(0, 0),
            facing: HexDirection::East,
            initial_stance: UnitStance::Formed,
        };
        plan.deployments.push(deployment);
        assert_eq!(plan.deployments.len(), 1);
    }

    #[test]
    fn test_engagement_rule_aggressive() {
        let rule = EngagementRule::Aggressive;
        assert!(rule.should_attack_on_sight());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::planning`
Expected: FAIL with "cannot find type `Waypoint`"

**Step 3: Write minimal implementation**

```rust
//! Battle planning structures (waypoints, go-codes, contingencies)
//!
//! Plan like Rainbow Six - waypoints, triggers, and contingencies.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::battle::hex::{BattleHexCoord, HexDirection};
use crate::battle::units::{UnitId, UnitStance, FormationId};
use crate::core::types::Tick;

/// Unique identifier for go-codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoCodeId(pub Uuid);

impl GoCodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for GoCodeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Movement pace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MovementPace {
    Walk,       // Preserve stamina
    #[default]
    Quick,      // Faster, some fatigue
    Run,        // Fast, tiring
    Charge,     // Maximum speed, exhausting, triggers shock
}

impl MovementPace {
    /// Speed multiplier
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            MovementPace::Walk => 0.5,
            MovementPace::Quick => 1.0,
            MovementPace::Run => 1.5,
            MovementPace::Charge => 2.0,
        }
    }

    /// Fatigue rate multiplier
    pub fn fatigue_multiplier(&self) -> f32 {
        match self {
            MovementPace::Walk => 0.5,
            MovementPace::Quick => 1.0,
            MovementPace::Run => 2.0,
            MovementPace::Charge => 4.0,
        }
    }
}

/// Waypoint behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WaypointBehavior {
    #[default]
    MoveTo,     // Just get there
    HoldAt,     // Stop and defend
    AttackFrom, // Assault from this position
    ScanFrom,   // Observe, report
    RallyAt,    // Reform here if broken
}

/// Condition to wait for at waypoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitCondition {
    Duration(u64),              // Wait for N ticks
    GoCode(GoCodeId),           // Wait for go-code
    UnitArrives(UnitId),        // Wait for another unit
    EnemySighted,               // Wait until enemy seen
    Attacked,                   // Wait until attacked
}

/// A waypoint in a movement plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub position: BattleHexCoord,
    pub behavior: WaypointBehavior,
    pub pace: MovementPace,
    pub wait_condition: Option<WaitCondition>,
}

impl Waypoint {
    pub fn new(position: BattleHexCoord, behavior: WaypointBehavior) -> Self {
        Self {
            position,
            behavior,
            pace: MovementPace::default(),
            wait_condition: None,
        }
    }

    pub fn with_pace(mut self, pace: MovementPace) -> Self {
        self.pace = pace;
        self
    }

    pub fn with_wait(mut self, condition: WaitCondition) -> Self {
        self.wait_condition = Some(condition);
        self
    }
}

/// Waypoint plan for a unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaypointPlan {
    pub unit_id: UnitId,
    pub waypoints: Vec<Waypoint>,
    pub current_waypoint: usize,
}

impl WaypointPlan {
    pub fn new(unit_id: UnitId) -> Self {
        Self {
            unit_id,
            waypoints: Vec::new(),
            current_waypoint: 0,
        }
    }

    pub fn add_waypoint(&mut self, waypoint: Waypoint) {
        self.waypoints.push(waypoint);
    }

    pub fn current(&self) -> Option<&Waypoint> {
        self.waypoints.get(self.current_waypoint)
    }

    pub fn advance(&mut self) -> bool {
        if self.current_waypoint < self.waypoints.len().saturating_sub(1) {
            self.current_waypoint += 1;
            true
        } else {
            false
        }
    }
}

/// Engagement rules for a unit
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum EngagementRule {
    #[default]
    Aggressive,     // Attack enemies on sight
    Defensive,      // Only attack if attacked first
    HoldFire,       // No attacking unless directly ordered
    Skirmish,       // Engage then withdraw
}

impl EngagementRule {
    pub fn should_attack_on_sight(&self) -> bool {
        matches!(self, EngagementRule::Aggressive)
    }

    pub fn should_withdraw_after_engagement(&self) -> bool {
        matches!(self, EngagementRule::Skirmish)
    }
}

/// Go-code trigger condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoCodeTrigger {
    Manual,                     // Player activates
    Time(Tick),                 // At specific tick
    UnitPosition {
        unit: UnitId,
        position: BattleHexCoord,
    },
    EnemyInArea {
        area: Vec<BattleHexCoord>,
    },
}

/// A go-code (coordinated trigger)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoCode {
    pub id: GoCodeId,
    pub name: String,
    pub trigger: GoCodeTrigger,
    pub subscribers: Vec<UnitId>,
    pub triggered: bool,
}

impl GoCode {
    pub fn new(name: String, trigger: GoCodeTrigger) -> Self {
        Self {
            id: GoCodeId::new(),
            name,
            trigger,
            subscribers: Vec::new(),
            triggered: false,
        }
    }

    pub fn subscribe(&mut self, unit_id: UnitId) {
        if !self.subscribers.contains(&unit_id) {
            self.subscribers.push(unit_id);
        }
    }
}

/// Contingency trigger condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContingencyTrigger {
    UnitBreaks(UnitId),
    CommanderDies,
    PositionLost(BattleHexCoord),
    EnemyFlanking,
    CasualtiesExceed(f32),      // Percentage
}

/// Contingency response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContingencyResponse {
    ExecutePlan(UnitId),        // Execute unit's backup plan
    Retreat(Vec<BattleHexCoord>), // Retreat route
    Rally(BattleHexCoord),      // Rally point
    Signal(GoCodeId),           // Trigger a go-code
}

/// A contingency (pre-planned response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contingency {
    pub trigger: ContingencyTrigger,
    pub response: ContingencyResponse,
    pub priority: u8,
    pub activated: bool,
}

impl Contingency {
    pub fn new(trigger: ContingencyTrigger, response: ContingencyResponse) -> Self {
        Self {
            trigger,
            response,
            priority: 0,
            activated: false,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Unit deployment in the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitDeployment {
    pub unit_id: UnitId,
    pub position: BattleHexCoord,
    pub facing: HexDirection,
    pub initial_stance: UnitStance,
}

/// Complete battle plan
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BattlePlan {
    pub deployments: Vec<UnitDeployment>,
    pub waypoint_plans: Vec<WaypointPlan>,
    pub engagement_rules: Vec<(UnitId, EngagementRule)>,
    pub go_codes: Vec<GoCode>,
    pub contingencies: Vec<Contingency>,
}

impl BattlePlan {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get waypoint plan for a unit
    pub fn get_waypoint_plan(&self, unit_id: UnitId) -> Option<&WaypointPlan> {
        self.waypoint_plans.iter().find(|p| p.unit_id == unit_id)
    }

    /// Get engagement rule for a unit
    pub fn get_engagement_rule(&self, unit_id: UnitId) -> EngagementRule {
        self.engagement_rules
            .iter()
            .find(|(id, _)| *id == unit_id)
            .map(|(_, rule)| rule.clone())
            .unwrap_or_default()
    }

    /// Get go-code by name
    pub fn get_go_code(&self, name: &str) -> Option<&GoCode> {
        self.go_codes.iter().find(|g| g.name == name)
    }

    /// Get mutable go-code by name
    pub fn get_go_code_mut(&mut self, name: &str) -> Option<&mut GoCode> {
        self.go_codes.iter_mut().find(|g| g.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waypoint_creation() {
        let wp = Waypoint::new(BattleHexCoord::new(5, 5), WaypointBehavior::MoveTo);
        assert_eq!(wp.position, BattleHexCoord::new(5, 5));
    }

    #[test]
    fn test_go_code_creation() {
        let go_code = GoCode::new("HAMMER".into(), GoCodeTrigger::Manual);
        assert_eq!(go_code.name, "HAMMER");
    }

    #[test]
    fn test_battle_plan_add_deployment() {
        let mut plan = BattlePlan::new();
        let deployment = UnitDeployment {
            unit_id: UnitId::new(),
            position: BattleHexCoord::new(0, 0),
            facing: HexDirection::East,
            initial_stance: UnitStance::Formed,
        };
        plan.deployments.push(deployment);
        assert_eq!(plan.deployments.len(), 1);
    }

    #[test]
    fn test_engagement_rule_aggressive() {
        let rule = EngagementRule::Aggressive;
        assert!(rule.should_attack_on_sight());
    }

    #[test]
    fn test_waypoint_plan_advance() {
        let mut plan = WaypointPlan::new(UnitId::new());
        plan.add_waypoint(Waypoint::new(BattleHexCoord::new(0, 0), WaypointBehavior::MoveTo));
        plan.add_waypoint(Waypoint::new(BattleHexCoord::new(5, 5), WaypointBehavior::HoldAt));

        assert_eq!(plan.current_waypoint, 0);
        assert!(plan.advance());
        assert_eq!(plan.current_waypoint, 1);
        assert!(!plan.advance()); // Can't advance past last
    }

    #[test]
    fn test_go_code_subscribe() {
        let mut go_code = GoCode::new("TEST".into(), GoCodeTrigger::Manual);
        let unit_id = UnitId::new();

        go_code.subscribe(unit_id);
        assert_eq!(go_code.subscribers.len(), 1);

        // Subscribing again shouldn't duplicate
        go_code.subscribe(unit_id);
        assert_eq!(go_code.subscribers.len(), 1);
    }

    #[test]
    fn test_movement_pace_speed() {
        assert!(MovementPace::Charge.speed_multiplier() > MovementPace::Run.speed_multiplier());
        assert!(MovementPace::Run.speed_multiplier() > MovementPace::Quick.speed_multiplier());
    }
}
```

**Step 4: Update mod.rs exports**

```rust
//! Battle system - tactical combat with fog of war and courier delays
//!
//! NOT Total War: terrain is dense, vision is scarce, control is delegated.

pub mod constants;
pub mod hex;
pub mod terrain;
pub mod battle_map;
pub mod unit_type;
pub mod units;
pub mod planning;
pub mod execution;
pub mod courier;
pub mod resolution;

pub use hex::{BattleHexCoord, HexDirection};
pub use terrain::{BattleTerrain, TerrainFeature};
pub use battle_map::{BattleHex, BattleMap, VisibilityState, Objective};
pub use unit_type::{UnitType, UnitProperties};
pub use units::{
    ArmyId, FormationId, UnitId,
    Element, BattleUnit, BattleFormation, Army,
    UnitStance, FormationShape,
};
pub use planning::{
    GoCodeId, MovementPace, WaypointBehavior, WaitCondition,
    Waypoint, WaypointPlan, EngagementRule,
    GoCodeTrigger, GoCode, ContingencyTrigger, ContingencyResponse,
    Contingency, UnitDeployment, BattlePlan,
};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib battle::planning`
Expected: PASS (7 tests)

**Step 6: Commit**

```bash
git add src/battle/planning.rs src/battle/mod.rs
git commit -m "feat(battle): add planning structures (waypoints, go-codes, contingencies)"
```

---

## Task 8: Courier System

**Files:**
- Replace: `src/battle/courier.rs`
- Modify: `src/battle/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_courier_creation() {
        let courier = CourierInFlight::new(
            EntityId::new(),
            Order::move_to(UnitId::new(), BattleHexCoord::new(5, 5)),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(10, 10),
        );
        assert_eq!(courier.status, CourierStatus::EnRoute);
    }

    #[test]
    fn test_courier_not_arrived_initially() {
        let courier = CourierInFlight::new(
            EntityId::new(),
            Order::move_to(UnitId::new(), BattleHexCoord::new(5, 5)),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(10, 10),
        );
        assert!(!courier.has_arrived());
    }

    #[test]
    fn test_order_types() {
        let order = Order::retreat(UnitId::new(), vec![BattleHexCoord::new(0, 0)]);
        assert!(matches!(order.order_type, OrderType::Retreat(_)));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::courier`
Expected: FAIL with "cannot find type `CourierInFlight`"

**Step 3: Write minimal implementation**

```rust
//! Courier system for order delivery
//!
//! Orders are NOT instant. Couriers carry commands across the battlefield.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::battle::hex::BattleHexCoord;
use crate::battle::units::{UnitId, FormationId, FormationShape};
use crate::battle::planning::{EngagementRule, GoCodeId};
use crate::core::types::{EntityId, Tick};

/// Unique identifier for couriers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CourierId(pub Uuid);

impl CourierId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CourierId {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of orders that can be sent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    MoveTo(BattleHexCoord),
    Attack(UnitId),
    Defend(BattleHexCoord),
    Retreat(Vec<BattleHexCoord>),    // Retreat route
    ChangeFormation(FormationShape),
    ChangeEngagement(EngagementRule),
    ExecuteGoCode(GoCodeId),
    Rally,
    HoldPosition,
}

/// Target of an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderTarget {
    Unit(UnitId),
    Formation(FormationId),
}

/// An order to be delivered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_type: OrderType,
    pub target: OrderTarget,
    pub issued_at: Tick,
}

impl Order {
    pub fn new(order_type: OrderType, target: OrderTarget, tick: Tick) -> Self {
        Self {
            order_type,
            target,
            issued_at: tick,
        }
    }

    /// Convenience: create a move order
    pub fn move_to(unit_id: UnitId, destination: BattleHexCoord) -> Self {
        Self {
            order_type: OrderType::MoveTo(destination),
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        }
    }

    /// Convenience: create a retreat order
    pub fn retreat(unit_id: UnitId, route: Vec<BattleHexCoord>) -> Self {
        Self {
            order_type: OrderType::Retreat(route),
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        }
    }

    /// Convenience: create an attack order
    pub fn attack(unit_id: UnitId, target: UnitId) -> Self {
        Self {
            order_type: OrderType::Attack(target),
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        }
    }

    /// Convenience: create a hold position order
    pub fn hold(unit_id: UnitId) -> Self {
        Self {
            order_type: OrderType::HoldPosition,
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        }
    }
}

/// Status of a courier in flight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CourierStatus {
    #[default]
    EnRoute,
    Arrived,
    Intercepted,    // Caught by enemy
    Lost,           // Courier killed
}

/// A courier carrying an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierInFlight {
    pub id: CourierId,
    pub courier_entity: EntityId,
    pub order: Order,

    pub source: BattleHexCoord,
    pub destination: BattleHexCoord,
    pub current_position: BattleHexCoord,

    pub progress: f32,          // Progress to next hex (0.0 to 1.0)
    pub path: Vec<BattleHexCoord>,

    pub status: CourierStatus,
}

impl CourierInFlight {
    pub fn new(
        courier_entity: EntityId,
        order: Order,
        source: BattleHexCoord,
        destination: BattleHexCoord,
    ) -> Self {
        // Simple path for now (straight line)
        let path = source.line_to(&destination);

        Self {
            id: CourierId::new(),
            courier_entity,
            order,
            source,
            destination,
            current_position: source,
            progress: 0.0,
            path,
            status: CourierStatus::EnRoute,
        }
    }

    /// Has the courier arrived?
    pub fn has_arrived(&self) -> bool {
        matches!(self.status, CourierStatus::Arrived)
    }

    /// Is the courier still en route?
    pub fn is_en_route(&self) -> bool {
        matches!(self.status, CourierStatus::EnRoute)
    }

    /// Was the courier intercepted?
    pub fn was_intercepted(&self) -> bool {
        matches!(self.status, CourierStatus::Intercepted)
    }

    /// Advance courier position by one step
    pub fn advance(&mut self, speed: f32) {
        if !self.is_en_route() {
            return;
        }

        self.progress += speed;

        // Move to next hex when progress reaches 1.0
        while self.progress >= 1.0 && !self.path.is_empty() {
            self.current_position = self.path.remove(0);
            self.progress -= 1.0;
        }

        // Check if arrived
        if self.path.is_empty() && self.current_position == self.destination {
            self.status = CourierStatus::Arrived;
        }
    }

    /// Mark courier as intercepted
    pub fn intercept(&mut self) {
        self.status = CourierStatus::Intercepted;
    }

    /// Mark courier as lost
    pub fn lose(&mut self) {
        self.status = CourierStatus::Lost;
    }

    /// Estimate remaining travel time in ticks
    pub fn estimate_eta(&self, speed: f32) -> u32 {
        if !self.is_en_route() {
            return 0;
        }

        let remaining_hexes = self.path.len() as f32 + (1.0 - self.progress);
        (remaining_hexes / speed).ceil() as u32
    }
}

/// Courier system managing all couriers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CourierSystem {
    pub in_flight: Vec<CourierInFlight>,
    pub delivered: Vec<Order>,
}

impl CourierSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Dispatch a new courier
    pub fn dispatch(
        &mut self,
        courier_entity: EntityId,
        order: Order,
        source: BattleHexCoord,
        destination: BattleHexCoord,
    ) -> CourierId {
        let courier = CourierInFlight::new(courier_entity, order, source, destination);
        let id = courier.id;
        self.in_flight.push(courier);
        id
    }

    /// Advance all couriers
    pub fn advance_all(&mut self, speed: f32) {
        for courier in &mut self.in_flight {
            courier.advance(speed);
        }
    }

    /// Collect arrived orders
    pub fn collect_arrived(&mut self) -> Vec<Order> {
        let mut arrived = Vec::new();

        self.in_flight.retain(|courier| {
            if courier.has_arrived() {
                arrived.push(courier.order.clone());
                false // Remove from in_flight
            } else {
                true // Keep in in_flight
            }
        });

        self.delivered.extend(arrived.clone());
        arrived
    }

    /// Get courier by ID
    pub fn get_courier(&self, id: CourierId) -> Option<&CourierInFlight> {
        self.in_flight.iter().find(|c| c.id == id)
    }

    /// Get mutable courier by ID
    pub fn get_courier_mut(&mut self, id: CourierId) -> Option<&mut CourierInFlight> {
        self.in_flight.iter_mut().find(|c| c.id == id)
    }

    /// Count couriers en route
    pub fn count_en_route(&self) -> usize {
        self.in_flight.iter().filter(|c| c.is_en_route()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_courier_creation() {
        let courier = CourierInFlight::new(
            EntityId::new(),
            Order::move_to(UnitId::new(), BattleHexCoord::new(5, 5)),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(10, 10),
        );
        assert_eq!(courier.status, CourierStatus::EnRoute);
    }

    #[test]
    fn test_courier_not_arrived_initially() {
        let courier = CourierInFlight::new(
            EntityId::new(),
            Order::move_to(UnitId::new(), BattleHexCoord::new(5, 5)),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(10, 10),
        );
        assert!(!courier.has_arrived());
    }

    #[test]
    fn test_order_types() {
        let order = Order::retreat(UnitId::new(), vec![BattleHexCoord::new(0, 0)]);
        assert!(matches!(order.order_type, OrderType::Retreat(_)));
    }

    #[test]
    fn test_courier_advance() {
        let mut courier = CourierInFlight::new(
            EntityId::new(),
            Order::hold(UnitId::new()),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(3, 0),
        );

        // Advance until arrived
        for _ in 0..20 {
            courier.advance(0.5);
            if courier.has_arrived() {
                break;
            }
        }

        assert!(courier.has_arrived());
    }

    #[test]
    fn test_courier_system_dispatch_and_collect() {
        let mut system = CourierSystem::new();

        system.dispatch(
            EntityId::new(),
            Order::hold(UnitId::new()),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(0, 0), // Same position = instant delivery
        );

        // Advance once
        system.advance_all(1.0);

        // Should have one arrived
        let arrived = system.collect_arrived();
        assert_eq!(arrived.len(), 1);
    }

    #[test]
    fn test_courier_interception() {
        let mut courier = CourierInFlight::new(
            EntityId::new(),
            Order::hold(UnitId::new()),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(10, 10),
        );

        courier.intercept();
        assert!(courier.was_intercepted());
        assert!(!courier.is_en_route());
    }
}
```

**Step 4: Update mod.rs exports**

Add to the exports:
```rust
pub use courier::{
    CourierId, OrderType, OrderTarget, Order,
    CourierStatus, CourierInFlight, CourierSystem,
};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib battle::courier`
Expected: PASS (5 tests)

**Step 6: Commit**

```bash
git add src/battle/courier.rs src/battle/mod.rs
git commit -m "feat(battle): add courier system for delayed order delivery"
```

---

## Task 9: Battle State and Execution Loop

**Files:**
- Replace: `src/battle/execution.rs`
- Modify: `src/battle/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battle_state_creation() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let state = BattleState::new(map, friendly, enemy);

        assert_eq!(state.tick, 0);
        assert!(!state.is_finished());
    }

    #[test]
    fn test_battle_tick_increments() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let mut state = BattleState::new(map, friendly, enemy);

        state.advance_tick();
        assert_eq!(state.tick, 1);
    }

    #[test]
    fn test_battle_phase_planning() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let state = BattleState::new(map, friendly, enemy);

        assert_eq!(state.phase, BattlePhase::Planning);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::execution`
Expected: FAIL with "cannot find type `BattleState`"

**Step 3: Write minimal implementation**

```rust
//! Battle execution loop
//!
//! Each tick: movement → couriers → engagement → combat → morale → rout

use serde::{Deserialize, Serialize};

use crate::battle::battle_map::BattleMap;
use crate::battle::units::{Army, UnitId};
use crate::battle::planning::BattlePlan;
use crate::battle::courier::CourierSystem;
use crate::core::types::{EntityId, Tick};

/// Battle phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BattlePhase {
    #[default]
    Planning,   // Pre-battle planning
    Deployment, // Placing units
    Active,     // Battle in progress
    Finished,   // Battle over
}

/// Battle outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleOutcome {
    Undecided,
    DecisiveVictory,
    Victory,
    PyrrhicVictory,
    Draw,
    Defeat,
    DecisiveDefeat,
}

impl Default for BattleOutcome {
    fn default() -> Self {
        Self::Undecided
    }
}

/// Log entry for battle events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleEvent {
    pub tick: Tick,
    pub event_type: BattleEventType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleEventType {
    BattleStarted,
    UnitEngaged { unit_id: UnitId },
    UnitBroke { unit_id: UnitId },
    UnitRallied { unit_id: UnitId },
    CommanderKilled { entity_id: EntityId },
    ObjectiveCaptured { name: String },
    CourierIntercepted,
    GoCodeTriggered { name: String },
    BattleEnded { outcome: BattleOutcome },
}

/// Routing unit tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingUnit {
    pub unit_id: UnitId,
    pub retreat_progress: f32,
}

/// Active combat between units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCombat {
    pub attacker_unit: UnitId,
    pub defender_unit: UnitId,
    pub ticks_engaged: u32,
}

/// Complete battle state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    // Core state
    pub map: BattleMap,
    pub friendly_army: Army,
    pub enemy_army: Army,

    // Time
    pub tick: Tick,
    pub phase: BattlePhase,
    pub outcome: BattleOutcome,

    // Plans
    pub friendly_plan: BattlePlan,
    pub enemy_plan: BattlePlan,

    // Systems
    pub courier_system: CourierSystem,

    // Combat tracking
    pub active_combats: Vec<ActiveCombat>,
    pub routing_units: Vec<RoutingUnit>,

    // Log
    pub battle_log: Vec<BattleEvent>,
}

impl BattleState {
    pub fn new(map: BattleMap, friendly_army: Army, enemy_army: Army) -> Self {
        Self {
            map,
            friendly_army,
            enemy_army,
            tick: 0,
            phase: BattlePhase::Planning,
            outcome: BattleOutcome::Undecided,
            friendly_plan: BattlePlan::new(),
            enemy_plan: BattlePlan::new(),
            courier_system: CourierSystem::new(),
            active_combats: Vec::new(),
            routing_units: Vec::new(),
            battle_log: Vec::new(),
        }
    }

    /// Is the battle finished?
    pub fn is_finished(&self) -> bool {
        matches!(self.phase, BattlePhase::Finished)
    }

    /// Start the battle (transition from planning to active)
    pub fn start_battle(&mut self) {
        self.phase = BattlePhase::Active;
        self.log_event(BattleEventType::BattleStarted, "Battle has begun!".into());
    }

    /// Advance the battle by one tick
    pub fn advance_tick(&mut self) {
        if self.is_finished() {
            return;
        }

        self.tick += 1;

        // The actual tick resolution is split into sub-tasks
        // This just increments time
    }

    /// Log a battle event
    pub fn log_event(&mut self, event_type: BattleEventType, description: String) {
        self.battle_log.push(BattleEvent {
            tick: self.tick,
            event_type,
            description,
        });
    }

    /// End the battle with an outcome
    pub fn end_battle(&mut self, outcome: BattleOutcome) {
        self.phase = BattlePhase::Finished;
        self.outcome = outcome;
        self.log_event(
            BattleEventType::BattleEnded { outcome },
            format!("Battle ended: {:?}", outcome),
        );
    }

    /// Get a unit from either army
    pub fn get_unit(&self, unit_id: UnitId) -> Option<&crate::battle::units::BattleUnit> {
        self.friendly_army
            .get_unit(unit_id)
            .or_else(|| self.enemy_army.get_unit(unit_id))
    }

    /// Get a mutable unit from either army
    pub fn get_unit_mut(&mut self, unit_id: UnitId) -> Option<&mut crate::battle::units::BattleUnit> {
        if self.friendly_army.get_unit(unit_id).is_some() {
            self.friendly_army.get_unit_mut(unit_id)
        } else {
            self.enemy_army.get_unit_mut(unit_id)
        }
    }
}

/// Check if battle should end
pub fn check_battle_end(state: &BattleState) -> Option<BattleOutcome> {
    let friendly_effective = state.friendly_army.effective_strength();
    let enemy_effective = state.enemy_army.effective_strength();

    // Check for army destruction
    if enemy_effective == 0 {
        return Some(BattleOutcome::DecisiveVictory);
    }

    if friendly_effective == 0 {
        return Some(BattleOutcome::DecisiveDefeat);
    }

    // Check for army rout (>80% routing)
    let enemy_routing = state.enemy_army.percentage_routing();
    let friendly_routing = state.friendly_army.percentage_routing();

    if enemy_routing > 0.8 {
        return Some(BattleOutcome::Victory);
    }

    if friendly_routing > 0.8 {
        return Some(BattleOutcome::Defeat);
    }

    // Check time limit
    if state.tick > crate::battle::constants::MAX_BATTLE_TICKS {
        // Determine by remaining strength
        if friendly_effective > enemy_effective * 2 {
            return Some(BattleOutcome::Victory);
        } else if enemy_effective > friendly_effective * 2 {
            return Some(BattleOutcome::Defeat);
        } else {
            return Some(BattleOutcome::Draw);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::units::ArmyId;
    use crate::core::types::EntityId;

    #[test]
    fn test_battle_state_creation() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let state = BattleState::new(map, friendly, enemy);

        assert_eq!(state.tick, 0);
        assert!(!state.is_finished());
    }

    #[test]
    fn test_battle_tick_increments() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let mut state = BattleState::new(map, friendly, enemy);

        state.advance_tick();
        assert_eq!(state.tick, 1);
    }

    #[test]
    fn test_battle_phase_planning() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let state = BattleState::new(map, friendly, enemy);

        assert_eq!(state.phase, BattlePhase::Planning);
    }

    #[test]
    fn test_battle_start() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let mut state = BattleState::new(map, friendly, enemy);

        state.start_battle();
        assert_eq!(state.phase, BattlePhase::Active);
        assert_eq!(state.battle_log.len(), 1);
    }

    #[test]
    fn test_battle_end() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let mut state = BattleState::new(map, friendly, enemy);

        state.end_battle(BattleOutcome::Victory);
        assert!(state.is_finished());
        assert_eq!(state.outcome, BattleOutcome::Victory);
    }

    #[test]
    fn test_check_battle_end_enemy_destroyed() {
        let map = BattleMap::new(20, 20);
        let mut friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());

        // Give friendly some units
        let mut formation = crate::battle::units::BattleFormation::new(
            crate::battle::units::FormationId::new(),
            EntityId::new(),
        );
        let mut unit = crate::battle::units::BattleUnit::new(
            crate::battle::units::UnitId::new(),
            crate::battle::unit_type::UnitType::Infantry,
        );
        unit.elements.push(crate::battle::units::Element::new(vec![EntityId::new(); 50]));
        formation.units.push(unit);
        friendly.formations.push(formation);

        let state = BattleState::new(map, friendly, enemy);

        // Enemy has no units = decisive victory
        let outcome = check_battle_end(&state);
        assert_eq!(outcome, Some(BattleOutcome::DecisiveVictory));
    }
}
```

**Step 4: Update mod.rs exports**

Add to exports:
```rust
pub use execution::{
    BattlePhase, BattleOutcome, BattleEvent, BattleEventType,
    RoutingUnit, ActiveCombat, BattleState, check_battle_end,
};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib battle::execution`
Expected: PASS (6 tests)

**Step 6: Commit**

```bash
git add src/battle/execution.rs src/battle/mod.rs
git commit -m "feat(battle): add battle state and execution framework"
```

---

## Task 10: Mass Combat Resolution

**Files:**
- Replace: `src/battle/resolution.rs`
- Modify: `src/battle/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_casualty_rate_sharp_vs_cloth() {
        let weapon = WeaponProperties::sword();
        let armor = ArmorProperties::none();
        let rate = calculate_casualty_rate(&weapon, &armor, 0.0);
        assert!(rate > 0.03); // High rate for unarmored
    }

    #[test]
    fn test_casualty_rate_sharp_vs_plate() {
        let weapon = WeaponProperties::sword();
        let armor = ArmorProperties::plate();
        let rate = calculate_casualty_rate(&weapon, &armor, 0.0);
        assert!(rate < 0.01); // Low rate for plate
    }

    #[test]
    fn test_pressure_affects_rate_additively() {
        let weapon = WeaponProperties::sword();
        let armor = ArmorProperties::leather();

        let rate_neutral = calculate_casualty_rate(&weapon, &armor, 0.0);
        let rate_positive = calculate_casualty_rate(&weapon, &armor, 0.5);
        let rate_negative = calculate_casualty_rate(&weapon, &armor, -0.5);

        // Pressure should affect rate additively
        assert!(rate_positive > rate_neutral);
        assert!(rate_negative < rate_neutral);

        // Check it's roughly additive (not multiplicative)
        let delta_pos = rate_positive - rate_neutral;
        let delta_neg = rate_neutral - rate_negative;
        assert!((delta_pos - delta_neg).abs() < 0.005);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::resolution`
Expected: FAIL with "cannot find function `calculate_casualty_rate`"

**Step 3: Write minimal implementation**

```rust
//! Mass combat resolution at different LOD levels
//!
//! Uses categorical property comparisons - NO percentage modifiers.

use crate::combat::{
    WeaponProperties, ArmorProperties,
    Edge, Mass, Rigidity, Padding,
    ShockType,
};
use crate::battle::units::{BattleUnit, UnitStance};
use crate::battle::unit_type::UnitType;

/// Level of detail for combat resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatLOD {
    Individual, // LOD-0: full individual combat
    Element,    // LOD-1: element-level statistical
    Unit,       // LOD-2: unit-level statistical
    Formation,  // LOD-3: formation-level aggregate
}

/// Result of unit-level combat
#[derive(Debug, Clone)]
pub struct UnitCombatResult {
    pub attacker_casualties: u32,
    pub defender_casualties: u32,
    pub attacker_stress_delta: f32,
    pub defender_stress_delta: f32,
    pub attacker_fatigue_delta: f32,
    pub defender_fatigue_delta: f32,
    pub pressure_shift: f32,
}

/// Result of a shock attack (charge, flank, etc.)
#[derive(Debug, Clone)]
pub struct ShockResult {
    pub immediate_casualties: u32,
    pub stress_spike: f32,
    pub triggered_break_check: bool,
}

/// Calculate base casualty rate from weapon vs armor
///
/// Returns casualties per combatant per tick (ADDITIVE, not percentage).
pub fn calculate_casualty_rate(
    weapon: &WeaponProperties,
    armor: &ArmorProperties,
    pressure: f32,
) -> f32 {
    // Base rate from property matchup (categorical, not percentage)
    let base_rate = match (weapon.edge, armor.rigidity) {
        // Sharp vs different armors
        (Edge::Razor, Rigidity::Cloth) => 0.06,
        (Edge::Razor, Rigidity::Leather) => 0.04,
        (Edge::Razor, Rigidity::Mail) => 0.01,
        (Edge::Razor, Rigidity::Plate) => 0.003,

        (Edge::Sharp, Rigidity::Cloth) => 0.05,
        (Edge::Sharp, Rigidity::Leather) => 0.03,
        (Edge::Sharp, Rigidity::Mail) => 0.01,
        (Edge::Sharp, Rigidity::Plate) => 0.005,

        // Blunt weapons care about mass vs padding
        (Edge::Blunt, _) => {
            match (weapon.mass, armor.padding) {
                (Mass::Massive, Padding::None) => 0.08,
                (Mass::Massive, Padding::Light) => 0.05,
                (Mass::Massive, Padding::Heavy) => 0.03,

                (Mass::Heavy, Padding::None) => 0.04,
                (Mass::Heavy, Padding::Light) => 0.02,
                (Mass::Heavy, Padding::Heavy) => 0.01,

                (Mass::Medium, Padding::None) => 0.02,
                (Mass::Medium, Padding::Light) => 0.01,
                (Mass::Medium, Padding::Heavy) => 0.005,

                (Mass::Light, _) => 0.005,
            }
        }
    };

    // Pressure modifier (ADDITIVE, not multiplicative)
    let pressure_modifier = pressure * 0.02; // +/-2% per pressure point

    (base_rate + pressure_modifier).clamp(0.001, 0.15)
}

/// Calculate stress delta from combat
pub fn calculate_stress_delta(
    unit: &BattleUnit,
    casualties: u32,
    is_flanked: bool,
    is_surrounded: bool,
) -> f32 {
    let mut stress = 0.0;

    // Base combat stress
    stress += 0.01;

    // Casualty stress (additive per casualty)
    stress += casualties as f32 * 0.02;

    // Flanked stress
    if is_flanked {
        stress += crate::battle::constants::FLANK_STRESS;
    }

    // Surrounded stress
    if is_surrounded {
        stress += 0.10;
    }

    stress
}

/// Resolve unit-level combat
pub fn resolve_unit_combat(
    attacker: &BattleUnit,
    defender: &BattleUnit,
    pressure: f32,
) -> UnitCombatResult {
    let attacker_props = attacker.unit_type.default_properties();
    let defender_props = defender.unit_type.default_properties();

    // Calculate casualty rates
    let defender_casualty_rate = calculate_casualty_rate(
        &attacker_props.avg_weapon,
        &defender_props.avg_armor,
        pressure,
    );

    let attacker_casualty_rate = calculate_casualty_rate(
        &defender_props.avg_weapon,
        &attacker_props.avg_armor,
        -pressure, // Pressure works against defender
    );

    // Apply casualties
    let defender_casualties =
        (defender_casualty_rate * defender.effective_strength() as f32).ceil() as u32;
    let attacker_casualties =
        (attacker_casualty_rate * attacker.effective_strength() as f32).ceil() as u32;

    // Calculate stress
    let attacker_stress = calculate_stress_delta(attacker, attacker_casualties, false, false);
    let defender_stress = calculate_stress_delta(defender, defender_casualties, false, false);

    // Fatigue from combat
    let fatigue_rate = crate::battle::constants::FATIGUE_RATE_COMBAT;

    // Pressure shifts based on casualties
    let pressure_shift = if defender_casualties > attacker_casualties {
        0.05
    } else if attacker_casualties > defender_casualties {
        -0.05
    } else {
        0.0
    };

    UnitCombatResult {
        attacker_casualties,
        defender_casualties,
        attacker_stress_delta: attacker_stress,
        defender_stress_delta: defender_stress,
        attacker_fatigue_delta: fatigue_rate,
        defender_fatigue_delta: fatigue_rate,
        pressure_shift,
    }
}

/// Calculate shock attack casualties
fn calculate_shock_casualties(
    attacker: &BattleUnit,
    defender: &BattleUnit,
    shock_type: ShockType,
) -> u32 {
    // Cavalry charge = Massive mass hitting front rank
    let front_rank_size = (defender.effective_strength() as f32 * 0.2) as u32;

    let defender_props = defender.unit_type.default_properties();

    // Survival rate based on padding
    let survival_rate = match defender_props.avg_armor.padding {
        Padding::None => 0.3,   // 70% casualties
        Padding::Light => 0.5,  // 50% casualties
        Padding::Heavy => 0.7,  // 30% casualties
    };

    let mut casualties = (front_rank_size as f32 * (1.0 - survival_rate)) as u32;

    // Spearmen reduce charge effectiveness
    if defender.unit_type == UnitType::Spearmen {
        casualties /= 2;
    }

    // Shock type modifiers
    match shock_type {
        ShockType::CavalryCharge => {} // Base calculation
        ShockType::FlankAttack => casualties = casualties * 2 / 3,
        ShockType::RearCharge => casualties = casualties * 3 / 2,
        ShockType::Ambush => casualties = casualties * 5 / 4,
    }

    casualties
}

/// Resolve a shock attack
pub fn resolve_shock_attack(
    attacker: &BattleUnit,
    defender: &BattleUnit,
    shock_type: ShockType,
) -> ShockResult {
    let casualties = calculate_shock_casualties(attacker, defender, shock_type);

    let stress_spike = shock_type.stress_spike()
        + (casualties as f32 / defender.effective_strength() as f32) * 0.20;

    let defender_threshold = defender.stress_threshold();
    let triggered_break_check = defender.stress + stress_spike > defender_threshold * 0.7;

    ShockResult {
        immediate_casualties: casualties,
        stress_spike,
        triggered_break_check,
    }
}

/// Determine LOD for a combat
pub fn determine_combat_lod(
    total_combatants: usize,
    is_player_focused: bool,
    is_near_objective: bool,
) -> CombatLOD {
    if is_player_focused {
        CombatLOD::Individual
    } else if is_near_objective || total_combatants < 50 {
        CombatLOD::Element
    } else if total_combatants < 200 {
        CombatLOD::Unit
    } else {
        CombatLOD::Formation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_casualty_rate_sharp_vs_cloth() {
        let weapon = WeaponProperties::sword();
        let armor = ArmorProperties::none();
        let rate = calculate_casualty_rate(&weapon, &armor, 0.0);
        assert!(rate > 0.03);
    }

    #[test]
    fn test_casualty_rate_sharp_vs_plate() {
        let weapon = WeaponProperties::sword();
        let armor = ArmorProperties::plate();
        let rate = calculate_casualty_rate(&weapon, &armor, 0.0);
        assert!(rate < 0.01);
    }

    #[test]
    fn test_pressure_affects_rate_additively() {
        let weapon = WeaponProperties::sword();
        let armor = ArmorProperties::leather();

        let rate_neutral = calculate_casualty_rate(&weapon, &armor, 0.0);
        let rate_positive = calculate_casualty_rate(&weapon, &armor, 0.5);
        let rate_negative = calculate_casualty_rate(&weapon, &armor, -0.5);

        assert!(rate_positive > rate_neutral);
        assert!(rate_negative < rate_neutral);

        let delta_pos = rate_positive - rate_neutral;
        let delta_neg = rate_neutral - rate_negative;
        assert!((delta_pos - delta_neg).abs() < 0.005);
    }

    #[test]
    fn test_spearmen_reduce_charge_casualties() {
        use crate::battle::units::{UnitId, Element};
        use crate::core::types::EntityId;

        let mut cavalry = BattleUnit::new(UnitId::new(), UnitType::HeavyCavalry);
        cavalry.elements.push(Element::new(vec![EntityId::new(); 50]));

        let mut infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        infantry.elements.push(Element::new(vec![EntityId::new(); 100]));

        let mut spearmen = BattleUnit::new(UnitId::new(), UnitType::Spearmen);
        spearmen.elements.push(Element::new(vec![EntityId::new(); 100]));

        let infantry_result = resolve_shock_attack(&cavalry, &infantry, ShockType::CavalryCharge);
        let spearmen_result = resolve_shock_attack(&cavalry, &spearmen, ShockType::CavalryCharge);

        assert!(spearmen_result.immediate_casualties < infantry_result.immediate_casualties);
    }

    #[test]
    fn test_determine_lod() {
        assert_eq!(determine_combat_lod(30, true, false), CombatLOD::Individual);
        assert_eq!(determine_combat_lod(30, false, true), CombatLOD::Element);
        assert_eq!(determine_combat_lod(30, false, false), CombatLOD::Element);
        assert_eq!(determine_combat_lod(100, false, false), CombatLOD::Unit);
        assert_eq!(determine_combat_lod(300, false, false), CombatLOD::Formation);
    }

    #[test]
    fn test_stress_delta_increases_with_casualties() {
        use crate::battle::units::UnitId;

        let unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);

        let stress_0 = calculate_stress_delta(&unit, 0, false, false);
        let stress_10 = calculate_stress_delta(&unit, 10, false, false);

        assert!(stress_10 > stress_0);
    }
}
```

**Step 4: Update mod.rs exports**

Add to exports:
```rust
pub use resolution::{
    CombatLOD, UnitCombatResult, ShockResult,
    calculate_casualty_rate, calculate_stress_delta,
    resolve_unit_combat, resolve_shock_attack, determine_combat_lod,
};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib battle::resolution`
Expected: PASS (5 tests)

**Step 6: Commit**

```bash
git add src/battle/resolution.rs src/battle/mod.rs
git commit -m "feat(battle): add mass combat resolution with LOD support"
```

---

## Task 11: Final Module Assembly and Integration Test

**Files:**
- Modify: `src/battle/mod.rs` (final version)
- Create: `tests/battle_integration.rs`

**Step 1: Write the integration test**

```rust
//! Battle system integration tests

use arc_citadel::battle::*;
use arc_citadel::combat::*;
use arc_citadel::core::types::EntityId;

#[test]
fn test_full_battle_setup() {
    // Create a battle map
    let mut map = BattleMap::new(30, 30);

    // Add some terrain
    map.set_terrain(BattleHexCoord::new(15, 10), BattleTerrain::Forest);
    map.set_terrain(BattleHexCoord::new(15, 11), BattleTerrain::Forest);
    map.set_terrain(BattleHexCoord::new(15, 12), BattleTerrain::Forest);

    // Forest should block LOS
    assert!(!map.has_line_of_sight(
        BattleHexCoord::new(10, 11),
        BattleHexCoord::new(20, 11)
    ));

    // Create armies
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());

    // Create a friendly formation with infantry
    let mut friendly_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    infantry.elements.push(Element::new(vec![EntityId::new(); 100]));
    infantry.position = BattleHexCoord::new(5, 15);
    friendly_formation.units.push(infantry);
    friendly.formations.push(friendly_formation);

    // Create enemy formation
    let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut enemy_infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    enemy_infantry.elements.push(Element::new(vec![EntityId::new(); 80]));
    enemy_infantry.position = BattleHexCoord::new(25, 15);
    enemy_formation.units.push(enemy_infantry);
    enemy.formations.push(enemy_formation);

    // Create battle state
    let mut state = BattleState::new(map, friendly, enemy);

    assert_eq!(state.phase, BattlePhase::Planning);
    assert_eq!(state.friendly_army.total_strength(), 100);
    assert_eq!(state.enemy_army.total_strength(), 80);

    // Start battle
    state.start_battle();
    assert_eq!(state.phase, BattlePhase::Active);

    // Check victory condition (neither army destroyed yet)
    assert!(check_battle_end(&state).is_none());
}

#[test]
fn test_courier_delivery_flow() {
    let map = BattleMap::new(20, 20);
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    friendly.hq_position = BattleHexCoord::new(0, 0);

    // Add courier to pool
    friendly.courier_pool.push(EntityId::new());

    let enemy = Army::new(ArmyId::new(), EntityId::new());

    let mut state = BattleState::new(map, friendly, enemy);

    // Dispatch a courier
    let unit_id = UnitId::new();
    let order = Order::move_to(unit_id, BattleHexCoord::new(10, 10));
    let destination = BattleHexCoord::new(5, 5);

    state.courier_system.dispatch(
        EntityId::new(),
        order,
        state.friendly_army.hq_position,
        destination,
    );

    assert_eq!(state.courier_system.count_en_route(), 1);

    // Advance couriers until delivered
    for _ in 0..50 {
        state.courier_system.advance_all(COURIER_SPEED);
    }

    let arrived = state.courier_system.collect_arrived();
    assert_eq!(arrived.len(), 1);
}

#[test]
fn test_go_code_planning() {
    let mut plan = BattlePlan::new();

    // Create a go-code for flanking maneuver
    let mut go_code = GoCode::new("HAMMER".into(), GoCodeTrigger::Manual);

    // Subscribe a cavalry unit
    let cavalry_id = UnitId::new();
    go_code.subscribe(cavalry_id);

    plan.go_codes.push(go_code);

    // Create waypoint plan for cavalry
    let mut waypoint_plan = WaypointPlan::new(cavalry_id);
    waypoint_plan.add_waypoint(
        Waypoint::new(BattleHexCoord::new(20, 5), WaypointBehavior::MoveTo)
            .with_pace(MovementPace::Quick)
    );
    waypoint_plan.add_waypoint(
        Waypoint::new(BattleHexCoord::new(25, 10), WaypointBehavior::HoldAt)
            .with_wait(WaitCondition::GoCode(plan.go_codes[0].id))
    );
    waypoint_plan.add_waypoint(
        Waypoint::new(BattleHexCoord::new(25, 15), WaypointBehavior::AttackFrom)
            .with_pace(MovementPace::Charge)
    );

    plan.waypoint_plans.push(waypoint_plan);

    // Verify plan structure
    assert_eq!(plan.go_codes.len(), 1);
    assert_eq!(plan.waypoint_plans.len(), 1);
    assert!(plan.get_go_code("HAMMER").is_some());
}

#[test]
fn test_combat_resolution_no_percentage_modifiers() {
    // This test ensures we're using additive, not multiplicative modifiers

    let weapon = WeaponProperties::sword();
    let armor = ArmorProperties::leather();

    // Get rates at different pressures
    let rates: Vec<f32> = (-5..=5)
        .map(|p| calculate_casualty_rate(&weapon, &armor, p as f32 * 0.1))
        .collect();

    // Calculate deltas between adjacent pressures
    let deltas: Vec<f32> = rates.windows(2).map(|w| w[1] - w[0]).collect();

    // All deltas should be approximately equal (additive behavior)
    let avg_delta: f32 = deltas.iter().sum::<f32>() / deltas.len() as f32;
    for delta in &deltas {
        assert!(
            (delta - avg_delta).abs() < 0.001,
            "Deltas should be consistent for additive behavior"
        );
    }
}
```

**Step 2: Final mod.rs**

```rust
//! Battle system - tactical combat with fog of war and courier delays
//!
//! NOT Total War: terrain is dense, vision is scarce, control is delegated.
//!
//! Key differences from typical RTS:
//! - Dense terrain constrains movement (navigation is a puzzle)
//! - Vision is scarce (information must be gathered)
//! - Orders go through couriers (not instant)
//! - Same simulation at all accessibility levels

pub mod constants;
pub mod hex;
pub mod terrain;
pub mod battle_map;
pub mod unit_type;
pub mod units;
pub mod planning;
pub mod courier;
pub mod execution;
pub mod resolution;

// Re-exports for convenient access
pub use constants::*;
pub use hex::{BattleHexCoord, HexDirection};
pub use terrain::{BattleTerrain, TerrainFeature};
pub use battle_map::{BattleHex, BattleMap, VisibilityState, Objective};
pub use unit_type::{UnitType, UnitProperties};
pub use units::{
    ArmyId, FormationId, UnitId,
    Element, BattleUnit, BattleFormation, Army,
    UnitStance, FormationShape,
};
pub use planning::{
    GoCodeId, MovementPace, WaypointBehavior, WaitCondition,
    Waypoint, WaypointPlan, EngagementRule,
    GoCodeTrigger, GoCode, ContingencyTrigger, ContingencyResponse,
    Contingency, UnitDeployment, BattlePlan,
};
pub use courier::{
    CourierId, OrderType, OrderTarget, Order,
    CourierStatus, CourierInFlight, CourierSystem,
};
pub use execution::{
    BattlePhase, BattleOutcome, BattleEvent, BattleEventType,
    RoutingUnit, ActiveCombat, BattleState, check_battle_end,
};
pub use resolution::{
    CombatLOD, UnitCombatResult, ShockResult,
    calculate_casualty_rate, calculate_stress_delta,
    resolve_unit_combat, resolve_shock_attack, determine_combat_lod,
};
```

**Step 3: Run all tests**

Run: `cargo test --lib battle`
Expected: PASS (all ~50 tests)

Run: `cargo test --test battle_integration`
Expected: PASS (4 tests)

**Step 4: Commit**

```bash
git add src/battle/mod.rs tests/battle_integration.rs
git commit -m "feat(battle): complete battle system with integration tests"
```

---

## Summary

This implementation plan creates a complete battle system:

1. **Constants** - All tunable values in one place (additive, never multiplicative)
2. **Hex system** - Axial coordinates with distance, neighbors, line drawing
3. **Terrain** - Dense terrain types that constrain movement
4. **Battle map** - Hex grid with LOS, elevation, visibility
5. **Unit types** - Infantry, cavalry, archers with default properties
6. **Unit hierarchy** - Element → Unit → Formation → Army
7. **Planning** - Waypoints, go-codes, contingencies (Rainbow Six style)
8. **Couriers** - Delayed order delivery (not instant)
9. **Execution** - Battle state, phases, events
10. **Resolution** - Multi-LOD combat with categorical property comparisons

**Total: ~2000 lines of Rust across 10 files, ~60 tests**

**Verification commands:**
- `cargo test --lib battle` - All battle unit tests
- `cargo test --test battle_integration` - Integration tests
- `cargo build` - Verify compilation
- `cargo clippy` - Check for common issues

---

**Plan complete and saved to `docs/plans/2026-01-03-battle-system.md`. Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
