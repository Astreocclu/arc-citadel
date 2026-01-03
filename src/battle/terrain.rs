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
