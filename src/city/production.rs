//! Production system - processes building production each tick
//!
//! The production tick system iterates over all buildings with active recipes,
//! advances production progress based on worker count, and on completion:
//! - Consumes input resources from the stockpile
//! - Adds output resources to the stockpile
//! - Resets progress for the next cycle

use crate::city::building::BuildingArchetype;
use crate::city::recipe::RecipeCatalog;
use crate::city::stockpile::Stockpile;

/// Result of a single production cycle completion
#[derive(Debug, Clone, PartialEq)]
pub struct ProductionResult {
    /// Index of the building that completed production
    pub building_idx: usize,
    /// ID of the recipe that completed
    pub recipe_id: String,
    /// Number of cycles completed (always 1 per tick, but kept for future batch processing)
    pub cycles_completed: u32,
}

/// Process production for all active buildings
///
/// This function:
/// 1. Iterates over all buildings with active recipes (using iter_producing)
/// 2. Advances production progress based on worker count and recipe rate
/// 3. On completion:
///    - Checks if stockpile has required inputs (skips if not)
///    - Consumes inputs from stockpile
///    - Adds outputs to stockpile
///    - Resets progress for next cycle
///
/// Returns a list of completed production cycles for tracking/events.
pub fn tick_production(
    buildings: &mut BuildingArchetype,
    recipes: &RecipeCatalog,
    stockpile: &mut Stockpile,
) -> Vec<ProductionResult> {
    let mut results = Vec::new();

    // Collect indices to avoid borrowing issues during iteration
    let producing_indices: Vec<usize> = buildings.iter_producing().collect();

    for i in producing_indices {
        // Get recipe ID (we know it exists because iter_producing filters for Some)
        let recipe_id = match &buildings.active_recipes[i] {
            Some(id) => id.clone(),
            None => continue, // Safety check, should not happen
        };

        // Look up the recipe
        let recipe = match recipes.get(&recipe_id) {
            Some(r) => r,
            None => continue, // Unknown recipe, skip
        };

        // Check if we have required inputs before proceeding
        // This prevents wasting production progress on recipes we can't complete
        if !stockpile.has_materials(&recipe.inputs) {
            continue; // Can't produce without materials
        }

        // Calculate progress this tick based on workers
        let workers = buildings.production_workers[i];
        let rate = recipe.production_rate(workers);
        // Progress is rate / work_required (normalized to 0.0-1.0 range)
        let progress = if recipe.work_required > 0 {
            rate / recipe.work_required as f32
        } else {
            1.0 // Instant completion if no work required
        };

        // Advance production and check for completion
        if buildings.advance_production(i, progress) {
            // Production cycle complete!

            // Consume inputs from stockpile
            stockpile.consume_materials(&recipe.inputs);

            // Produce outputs - add to stockpile
            for (resource, amount) in &recipe.outputs {
                stockpile.add(*resource, *amount);
            }

            results.push(ProductionResult {
                building_idx: i,
                recipe_id,
                cycles_completed: 1,
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::building::{BuildingArchetype, BuildingId, BuildingState, BuildingType};
    use crate::city::recipe::RecipeCatalog;
    use crate::city::stockpile::Stockpile;
    use crate::core::types::Vec2;
    use crate::simulation::resource_zone::ResourceType;

    #[test]
    fn test_tick_production_basic() {
        // Setup: Create a completed farm with farm_food recipe
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new();
        buildings.spawn(id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);
        buildings.states[0] = BuildingState::Complete;
        buildings.start_production(0, "farm_food".into());
        buildings.production_workers[0] = 2; // Full worker capacity

        let recipes = RecipeCatalog::with_defaults();
        let mut stockpile = Stockpile::new();

        // Set capacity for food
        stockpile.set_capacity(ResourceType::Food, 1000);

        // Farm recipe: work_required=100, workers_needed=2
        // With 2 workers: rate = 1.0, progress per tick = 1.0/100 = 0.01
        // Run 101 ticks to ensure completion despite floating point errors

        // Run enough ticks to complete one cycle
        let mut total_cycles = 0;
        for _ in 0..101 {
            let results = tick_production(&mut buildings, &recipes, &mut stockpile);
            total_cycles += results.len();
        }

        // Should have completed exactly 1 cycle
        assert_eq!(
            total_cycles, 1,
            "Should complete exactly 1 production cycle"
        );

        // Should have produced food (farm_food outputs 5 food)
        assert_eq!(stockpile.get(ResourceType::Food), 5);
    }

    #[test]
    fn test_tick_production_no_workers() {
        // Setup: Farm with no workers
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new();
        buildings.spawn(id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);
        buildings.states[0] = BuildingState::Complete;
        buildings.start_production(0, "farm_food".into());
        buildings.production_workers[0] = 0; // No workers!

        let recipes = RecipeCatalog::with_defaults();
        let mut stockpile = Stockpile::new();
        stockpile.set_capacity(ResourceType::Food, 1000);

        // Run 100 ticks
        let mut total_cycles = 0;
        for _ in 0..100 {
            let results = tick_production(&mut buildings, &recipes, &mut stockpile);
            total_cycles += results.len();
        }

        // With 0 workers, production_rate = 0, so no progress
        assert_eq!(
            total_cycles, 0,
            "Should not complete any cycles with 0 workers"
        );
        assert_eq!(
            stockpile.get(ResourceType::Food),
            0,
            "Should not have produced any food"
        );
    }

    #[test]
    fn test_tick_production_requires_inputs() {
        // Setup: Workshop with smelt_iron recipe (needs Ore)
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new();
        buildings.spawn(id, BuildingType::Workshop, Vec2::new(0.0, 0.0), 0);
        buildings.states[0] = BuildingState::Complete;
        buildings.start_production(0, "smelt_iron".into());
        buildings.production_workers[0] = 1;

        let recipes = RecipeCatalog::with_defaults();
        let mut stockpile = Stockpile::new();
        stockpile.set_capacity(ResourceType::Iron, 100);
        stockpile.set_capacity(ResourceType::Ore, 100);

        // Don't add any ore - production should not proceed

        // Run 100 ticks
        let mut total_cycles = 0;
        for _ in 0..100 {
            let results = tick_production(&mut buildings, &recipes, &mut stockpile);
            total_cycles += results.len();
        }

        // Should not complete because no ore available
        assert_eq!(
            total_cycles, 0,
            "Should not complete cycles without required inputs"
        );
        assert_eq!(
            stockpile.get(ResourceType::Iron),
            0,
            "Should not produce iron without ore"
        );
    }

    #[test]
    fn test_tick_production_consumes_inputs() {
        // Setup: Workshop with smelt_iron recipe (needs 3 Ore, produces 1 Iron)
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new();
        buildings.spawn(id, BuildingType::Workshop, Vec2::new(0.0, 0.0), 0);
        buildings.states[0] = BuildingState::Complete;
        buildings.start_production(0, "smelt_iron".into());
        buildings.production_workers[0] = 1;

        let recipes = RecipeCatalog::with_defaults();
        let mut stockpile = Stockpile::new();
        stockpile.set_capacity(ResourceType::Iron, 100);
        stockpile.set_capacity(ResourceType::Ore, 100);

        // Add enough ore for exactly 2 cycles (6 ore, 3 per cycle)
        stockpile.add(ResourceType::Ore, 6);

        // smelt_iron: work_required=50, workers_needed=1
        // With 1 worker: rate=1.0, progress = 1.0/50 = 0.02 per tick
        // 50 ticks per cycle, run 102 ticks for 2 cycles (extra for floating point)

        let mut total_cycles = 0;
        for _ in 0..102 {
            let results = tick_production(&mut buildings, &recipes, &mut stockpile);
            total_cycles += results.len();
        }

        // Should complete 2 cycles
        assert_eq!(
            total_cycles, 2,
            "Should complete exactly 2 production cycles"
        );

        // Should have consumed all ore (6) and produced 2 iron
        assert_eq!(
            stockpile.get(ResourceType::Ore),
            0,
            "Should have consumed all ore"
        );
        assert_eq!(
            stockpile.get(ResourceType::Iron),
            2,
            "Should have produced 2 iron"
        );
    }

    #[test]
    fn test_tick_production_multiple_buildings() {
        // Setup: Multiple buildings producing simultaneously
        let mut buildings = BuildingArchetype::new();

        // Farm producing food
        let farm_id = BuildingId::new();
        buildings.spawn(farm_id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);
        buildings.states[0] = BuildingState::Complete;
        buildings.start_production(0, "farm_food".into());
        buildings.production_workers[0] = 2;

        // Workshop producing cloth
        let workshop_id = BuildingId::new();
        buildings.spawn(workshop_id, BuildingType::Workshop, Vec2::new(10.0, 0.0), 0);
        buildings.states[1] = BuildingState::Complete;
        buildings.start_production(1, "weave_cloth".into());
        buildings.production_workers[1] = 1;

        let recipes = RecipeCatalog::with_defaults();
        let mut stockpile = Stockpile::new();
        stockpile.set_capacity(ResourceType::Food, 1000);
        stockpile.set_capacity(ResourceType::Cloth, 1000);
        stockpile.set_capacity(ResourceType::Wood, 1000);

        // Add wood for cloth production (needs 2 wood per cycle)
        stockpile.add(ResourceType::Wood, 20);

        // Run 210 ticks to ensure both buildings complete cycles (extra for floating point)
        // farm_food: 100 ticks per cycle (100 work, rate 1.0)
        // weave_cloth: 40 ticks per cycle (40 work, rate 1.0)

        let mut farm_cycles = 0;
        let mut cloth_cycles = 0;
        for _ in 0..210 {
            let results = tick_production(&mut buildings, &recipes, &mut stockpile);
            for r in results {
                if r.recipe_id == "farm_food" {
                    farm_cycles += 1;
                } else if r.recipe_id == "weave_cloth" {
                    cloth_cycles += 1;
                }
            }
        }

        // Farm: ~2 cycles (210/100 with floating point)
        assert!(
            farm_cycles >= 2,
            "Farm should complete at least 2 cycles, got {}",
            farm_cycles
        );

        // Cloth: ~5 cycles (210/40 = 5.25, limited by 20 wood / 2 = 10 possible)
        assert!(
            cloth_cycles >= 4,
            "Workshop should complete at least 4 cloth cycles, got {}",
            cloth_cycles
        );

        // Check outputs
        assert!(
            stockpile.get(ResourceType::Food) >= 10,
            "Should have at least 10 food"
        );
        assert!(
            stockpile.get(ResourceType::Cloth) >= 4,
            "Should have at least 4 cloth"
        );
    }

    #[test]
    fn test_tick_production_returns_results() {
        // Setup: Farm ready to complete production
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new();
        buildings.spawn(id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);
        buildings.states[0] = BuildingState::Complete;
        buildings.start_production(0, "farm_food".into());
        buildings.production_workers[0] = 2;

        // Set progress to just before completion
        buildings.production_progress[0] = 0.99;

        let recipes = RecipeCatalog::with_defaults();
        let mut stockpile = Stockpile::new();
        stockpile.set_capacity(ResourceType::Food, 1000);

        // Run one tick - should complete
        let results = tick_production(&mut buildings, &recipes, &mut stockpile);

        assert_eq!(results.len(), 1, "Should return 1 result");
        assert_eq!(results[0].building_idx, 0);
        assert_eq!(results[0].recipe_id, "farm_food");
        assert_eq!(results[0].cycles_completed, 1);
    }

    #[test]
    fn test_tick_production_under_construction_excluded() {
        // Setup: Building still under construction should not produce
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new();
        buildings.spawn(id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);
        // Note: states[0] is UnderConstruction by default
        buildings.active_recipes[0] = Some("farm_food".into()); // Force recipe even though not complete
        buildings.production_workers[0] = 2;

        let recipes = RecipeCatalog::with_defaults();
        let mut stockpile = Stockpile::new();
        stockpile.set_capacity(ResourceType::Food, 1000);

        // Run ticks
        let mut total_cycles = 0;
        for _ in 0..100 {
            let results = tick_production(&mut buildings, &recipes, &mut stockpile);
            total_cycles += results.len();
        }

        // iter_producing filters out non-complete buildings
        assert_eq!(
            total_cycles, 0,
            "Under-construction buildings should not produce"
        );
    }

    #[test]
    fn test_tick_production_bonus_workers() {
        // Setup: Workshop with extra workers (should get 1.2x bonus)
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new();
        buildings.spawn(id, BuildingType::Workshop, Vec2::new(0.0, 0.0), 0);
        buildings.states[0] = BuildingState::Complete;
        buildings.start_production(0, "smelt_iron".into());
        buildings.production_workers[0] = 5; // 5 workers for recipe that needs 1 = 1.2x rate (capped)

        let recipes = RecipeCatalog::with_defaults();
        let mut stockpile = Stockpile::new();
        stockpile.set_capacity(ResourceType::Iron, 1000);
        stockpile.set_capacity(ResourceType::Ore, 1000);
        stockpile.add(ResourceType::Ore, 100); // Plenty of ore

        // smelt_iron: work_required=50, with 1.2x rate = 41.67 ticks per cycle
        // In 50 ticks with 1.2x rate: should complete 1+ cycles

        let mut total_cycles = 0;
        for _ in 0..50 {
            let results = tick_production(&mut buildings, &recipes, &mut stockpile);
            total_cycles += results.len();
        }

        // With bonus rate (1.2), should complete more than with base rate (1.0)
        // 50 * (1.2/50) = 1.2 progress, so 1 cycle complete
        assert_eq!(
            total_cycles, 1,
            "Should complete 1 cycle with bonus workers in 50 ticks"
        );
    }

    #[test]
    fn test_tick_production_progress_resets() {
        // Verify that progress resets after completion
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new();
        buildings.spawn(id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);
        buildings.states[0] = BuildingState::Complete;
        buildings.start_production(0, "farm_food".into());
        buildings.production_workers[0] = 2;
        buildings.production_progress[0] = 0.99;

        let recipes = RecipeCatalog::with_defaults();
        let mut stockpile = Stockpile::new();
        stockpile.set_capacity(ResourceType::Food, 1000);

        // Complete one cycle
        tick_production(&mut buildings, &recipes, &mut stockpile);

        // Progress should have reset to near 0 (might have small progress from current tick)
        assert!(
            buildings.production_progress[0] < 0.02,
            "Progress should reset after completion, got {}",
            buildings.production_progress[0]
        );
    }
}
