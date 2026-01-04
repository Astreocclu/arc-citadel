//! Per-army visibility (fog of war)
//!
//! Each army has its own visibility map based on unit positions.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::battle::battle_map::BattleMap;
use crate::battle::constants::{BASE_VISION_RANGE, ELEVATION_VISION_BONUS, SCOUT_VISION_BONUS};
use crate::battle::hex::BattleHexCoord;
use crate::battle::unit_type::UnitType;
use crate::battle::units::{Army, BattleUnit};

/// Visibility state for an army
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArmyVisibility {
    /// Currently visible hexes
    pub visible: HashSet<BattleHexCoord>,
    /// Previously seen hexes (remembered)
    pub remembered: HashSet<BattleHexCoord>,
}

impl ArmyVisibility {
    pub fn new() -> Self {
        Self::default()
    }

    /// Is this hex currently visible?
    pub fn is_visible(&self, coord: BattleHexCoord) -> bool {
        self.visible.contains(&coord)
    }

    /// Has this hex been seen before?
    pub fn is_remembered(&self, coord: BattleHexCoord) -> bool {
        self.remembered.contains(&coord)
    }

    /// Update: move current visible to remembered, set new visible
    pub fn update(&mut self, new_visible: HashSet<BattleHexCoord>) {
        // Move old visible to remembered
        self.remembered.extend(self.visible.drain());
        // Set new visible
        self.visible = new_visible;
        // Remove from remembered what is now visible
        for coord in &self.visible {
            self.remembered.remove(coord);
        }
    }
}

/// Calculate vision range for a unit
pub fn unit_vision_range(unit: &BattleUnit, map: &BattleMap) -> u32 {
    let mut range = BASE_VISION_RANGE;

    // Scout bonus (LightCavalry acts as scouts)
    if matches!(unit.unit_type, UnitType::LightCavalry) {
        range += SCOUT_VISION_BONUS;
    }

    // Elevation bonus
    if let Some(hex) = map.get_hex(unit.position) {
        if hex.elevation > 0 {
            range += ELEVATION_VISION_BONUS * hex.elevation as u32;
        }
    }

    range
}

/// Calculate visibility for an entire army
pub fn calculate_army_visibility(map: &BattleMap, army: &Army) -> ArmyVisibility {
    let mut visible = HashSet::new();

    for formation in &army.formations {
        for unit in &formation.units {
            if unit.effective_strength() == 0 {
                continue;
            }

            let range = unit_vision_range(unit, map);
            let unit_visible = map.visible_hexes(unit.position, range);
            visible.extend(unit_visible);
        }
    }

    let mut visibility = ArmyVisibility::new();
    visibility.visible = visible;
    visibility
}

/// Update army visibility in place
pub fn update_army_visibility(visibility: &mut ArmyVisibility, map: &BattleMap, army: &Army) {
    let new_visible = calculate_army_visibility(map, army).visible;
    visibility.update(new_visible);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::battle_map::BattleMap;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{Army, ArmyId, BattleFormation, BattleUnit, Element, FormationId, UnitId};
    use crate::core::types::EntityId;

    #[test]
    fn test_visibility_near_unit() {
        let map = BattleMap::new(20, 20);
        let mut army = Army::new(ArmyId::new(), EntityId::new());

        // Add a formation with one unit at (10, 10)
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(10, 10);
        unit.elements.push(Element::new(vec![EntityId::new(); 10]));
        formation.units.push(unit);
        army.formations.push(formation);

        let visibility = calculate_army_visibility(&map, &army);

        // Unit position should be visible
        assert!(visibility.is_visible(BattleHexCoord::new(10, 10)));
        // Nearby hexes should be visible
        assert!(visibility.is_visible(BattleHexCoord::new(11, 10)));
        // Far hexes should not be visible
        assert!(!visibility.is_visible(BattleHexCoord::new(0, 0)));
    }

    #[test]
    fn test_remembered_hexes() {
        let _map = BattleMap::new(20, 20);
        let mut visibility = ArmyVisibility::new();

        // First update: see hex (5,5)
        let mut visible1 = HashSet::new();
        visible1.insert(BattleHexCoord::new(5, 5));
        visibility.update(visible1);

        assert!(visibility.is_visible(BattleHexCoord::new(5, 5)));

        // Second update: see different hex
        let mut visible2 = HashSet::new();
        visible2.insert(BattleHexCoord::new(10, 10));
        visibility.update(visible2);

        // Old hex should be remembered, not visible
        assert!(!visibility.is_visible(BattleHexCoord::new(5, 5)));
        assert!(visibility.is_remembered(BattleHexCoord::new(5, 5)));
        assert!(visibility.is_visible(BattleHexCoord::new(10, 10)));
    }

    #[test]
    fn test_scout_bonus_vision() {
        let map = BattleMap::new(20, 20);

        let mut infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        infantry.position = BattleHexCoord::new(10, 10);

        let mut scout = BattleUnit::new(UnitId::new(), UnitType::LightCavalry);
        scout.position = BattleHexCoord::new(10, 10);

        let infantry_range = unit_vision_range(&infantry, &map);
        let scout_range = unit_vision_range(&scout, &map);

        assert!(scout_range > infantry_range);
        assert_eq!(scout_range - infantry_range, SCOUT_VISION_BONUS);
    }

    #[test]
    fn test_elevation_bonus_vision() {
        let mut map = BattleMap::new(20, 20);
        map.set_elevation(BattleHexCoord::new(10, 10), 2);

        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(10, 10);

        let range_on_hill = unit_vision_range(&unit, &map);

        // Move unit to flat ground
        unit.position = BattleHexCoord::new(5, 5);
        let range_on_flat = unit_vision_range(&unit, &map);

        assert!(range_on_hill > range_on_flat);
        assert_eq!(range_on_hill - range_on_flat, ELEVATION_VISION_BONUS * 2);
    }

    #[test]
    fn test_dead_unit_no_vision() {
        let map = BattleMap::new(20, 20);
        let mut army = Army::new(ArmyId::new(), EntityId::new());

        // Add a unit with no elements (effectively dead)
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        // No elements means effective_strength() == 0
        formation.units.push(unit);
        army.formations.push(formation);

        let visibility = calculate_army_visibility(&map, &army);

        // Should have no visible hexes since unit has no strength
        assert!(visibility.visible.is_empty());
    }

    #[test]
    fn test_multiple_units_combine_vision() {
        let map = BattleMap::new(30, 30);
        let mut army = Army::new(ArmyId::new(), EntityId::new());

        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());

        // Unit 1 at (5, 5)
        let mut unit1 = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit1.position = BattleHexCoord::new(5, 5);
        unit1.elements.push(Element::new(vec![EntityId::new(); 10]));
        formation.units.push(unit1);

        // Unit 2 at (25, 25)
        let mut unit2 = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit2.position = BattleHexCoord::new(25, 25);
        unit2.elements.push(Element::new(vec![EntityId::new(); 10]));
        formation.units.push(unit2);

        army.formations.push(formation);

        let visibility = calculate_army_visibility(&map, &army);

        // Both unit positions should be visible
        assert!(visibility.is_visible(BattleHexCoord::new(5, 5)));
        assert!(visibility.is_visible(BattleHexCoord::new(25, 25)));

        // Nearby hexes to each unit should be visible
        assert!(visibility.is_visible(BattleHexCoord::new(6, 5)));
        assert!(visibility.is_visible(BattleHexCoord::new(24, 25)));
    }

    #[test]
    fn test_update_army_visibility() {
        let map = BattleMap::new(20, 20);
        let mut army = Army::new(ArmyId::new(), EntityId::new());

        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 5);
        unit.elements.push(Element::new(vec![EntityId::new(); 10]));
        formation.units.push(unit);
        army.formations.push(formation);

        let mut visibility = ArmyVisibility::new();

        // First update
        update_army_visibility(&mut visibility, &map, &army);
        assert!(visibility.is_visible(BattleHexCoord::new(5, 5)));

        // Move the unit
        army.formations[0].units[0].position = BattleHexCoord::new(15, 15);

        // Second update
        update_army_visibility(&mut visibility, &map, &army);

        // New position visible
        assert!(visibility.is_visible(BattleHexCoord::new(15, 15)));
        // Old position remembered (if it wasn't in new range)
        assert!(visibility.is_remembered(BattleHexCoord::new(5, 5)));
    }

    #[test]
    fn test_visibility_respects_los() {
        use crate::battle::terrain::BattleTerrain;

        let mut map = BattleMap::new(20, 20);
        // Place a forest blocking line of sight
        map.set_terrain(BattleHexCoord::new(7, 5), BattleTerrain::Forest);

        let mut army = Army::new(ArmyId::new(), EntityId::new());
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 5);
        unit.elements.push(Element::new(vec![EntityId::new(); 10]));
        formation.units.push(unit);
        army.formations.push(formation);

        let visibility = calculate_army_visibility(&map, &army);

        // Position just before forest should be visible
        assert!(visibility.is_visible(BattleHexCoord::new(6, 5)));

        // Position behind forest should NOT be visible (blocked by LOS)
        assert!(!visibility.is_visible(BattleHexCoord::new(10, 5)));
    }
}
