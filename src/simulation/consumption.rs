//! Food consumption system
//!
//! Living entities consume food from the stockpile daily.

use crate::ecs::world::World;
use crate::simulation::resource_zone::ResourceType;

/// Consume food for all living entities
/// Returns number of entities that went hungry
pub fn consume_food(world: &mut World) -> u32 {
    let living_count = world.humans.iter_living().count() as u32;

    if living_count == 0 {
        return 0;
    }

    let food_needed = living_count; // 1 food per entity
    let food_consumed = world.stockpile.remove(ResourceType::Food, food_needed);

    // Return number who went hungry
    living_count - food_consumed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_food_basic() {
        let mut world = World::new();

        // Add food to stockpile
        world.stockpile.add(ResourceType::Food, 100);

        // Spawn 3 humans
        world.spawn_human("Alice".into());
        world.spawn_human("Bob".into());
        world.spawn_human("Charlie".into());

        let hungry = consume_food(&mut world);

        assert_eq!(hungry, 0, "No one should be hungry");
        assert_eq!(
            world.stockpile.get(ResourceType::Food),
            97,
            "Should consume 3 food"
        );
    }

    #[test]
    fn test_consume_food_not_enough() {
        let mut world = World::new();

        // Only 1 food, 3 humans
        world.stockpile.add(ResourceType::Food, 1);

        world.spawn_human("Alice".into());
        world.spawn_human("Bob".into());
        world.spawn_human("Charlie".into());

        let hungry = consume_food(&mut world);

        assert_eq!(hungry, 2, "2 should go hungry");
        assert_eq!(
            world.stockpile.get(ResourceType::Food),
            0,
            "Should consume all food"
        );
    }

    #[test]
    fn test_consume_food_empty_stockpile() {
        let mut world = World::new();

        world.spawn_human("Alice".into());

        let hungry = consume_food(&mut world);

        assert_eq!(hungry, 1, "Should be hungry with no food");
    }
}
