//! Housing assignment system
//!
//! Assigns homeless entities to available houses.

use crate::city::building::BuildingState;
use crate::ecs::world::World;

/// Assign homeless humans to available housing
pub fn assign_housing(world: &mut World) {
    // Calculate available capacity per completed house
    let mut available: Vec<(crate::city::building::BuildingId, u32)> = Vec::new();

    for idx in 0..world.buildings.count() {
        if world.buildings.states[idx] != BuildingState::Complete {
            continue;
        }

        let capacity = world.buildings.building_types[idx].housing_capacity();
        if capacity == 0 {
            continue;
        }

        let building_id = world.buildings.ids[idx];

        // Count current occupants
        let current_occupants = world
            .humans
            .assigned_houses
            .iter()
            .filter(|&&h| h == Some(building_id))
            .count() as u32;

        let remaining = capacity.saturating_sub(current_occupants);
        if remaining > 0 {
            available.push((building_id, remaining));
        }
    }

    // Assign homeless to available housing
    let mut available_iter = available.into_iter().peekable();
    let mut current_house = available_iter.peek().map(|(id, _)| *id);
    let mut remaining_capacity = available_iter.peek().map(|(_, cap)| *cap).unwrap_or(0);

    for idx in world.humans.iter_living().collect::<Vec<_>>() {
        if world.humans.assigned_houses[idx].is_some() {
            continue; // Already housed
        }

        if current_house.is_none() {
            break; // No more housing available
        }

        world.humans.assigned_houses[idx] = current_house;
        remaining_capacity -= 1;

        if remaining_capacity == 0 {
            available_iter.next();
            current_house = available_iter.peek().map(|(id, _)| *id);
            remaining_capacity = available_iter.peek().map(|(_, cap)| *cap).unwrap_or(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::building::BuildingType;
    use crate::core::types::Vec2;

    #[test]
    fn test_assign_housing_basic() {
        let mut world = World::new();

        // Spawn a house (capacity 4)
        let house_id = world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
        // Complete construction
        world.buildings.states[0] = BuildingState::Complete;

        // Spawn 2 homeless humans
        world.spawn_human("Alice".into());
        world.spawn_human("Bob".into());

        // Both should be homeless initially
        assert!(world.humans.assigned_houses[0].is_none());
        assert!(world.humans.assigned_houses[1].is_none());

        // Run housing assignment
        assign_housing(&mut world);

        // Both should now be housed
        assert_eq!(world.humans.assigned_houses[0], Some(house_id));
        assert_eq!(world.humans.assigned_houses[1], Some(house_id));
    }

    #[test]
    fn test_assign_housing_respects_capacity() {
        let mut world = World::new();

        // Spawn a house (capacity 4)
        world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
        world.buildings.states[0] = BuildingState::Complete;

        // Spawn 6 humans (exceeds capacity)
        for i in 0..6 {
            world.spawn_human(format!("Human {}", i));
        }

        assign_housing(&mut world);

        // Count housed vs homeless
        let housed = world
            .humans
            .assigned_houses
            .iter()
            .filter(|h| h.is_some())
            .count();
        let homeless = world
            .humans
            .assigned_houses
            .iter()
            .filter(|h| h.is_none())
            .count();

        assert_eq!(housed, 4, "Should house exactly 4 (capacity)");
        assert_eq!(homeless, 2, "Should have 2 homeless");
    }

    #[test]
    fn test_assign_housing_ignores_incomplete_buildings() {
        let mut world = World::new();

        // Spawn a house still under construction
        world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
        // Don't complete it - stays UnderConstruction

        world.spawn_human("Alice".into());

        assign_housing(&mut world);

        // Should still be homeless (building not complete)
        assert!(world.humans.assigned_houses[0].is_none());
    }
}
