//! AI's filtered view of the battle state
//!
//! Respects fog of war unless ignores_fog_of_war is true.

use crate::battle::hex::BattleHexCoord;
use crate::battle::units::{Army, BattleUnit, UnitId};
use crate::battle::visibility::ArmyVisibility;
use crate::core::types::Tick;

/// AI's decision-making context
///
/// Provides a filtered view of the battle respecting fog of war.
pub struct DecisionContext<'a> {
    pub own_army: &'a Army,
    pub enemy_army: &'a Army,
    pub own_visibility: &'a ArmyVisibility,
    pub current_tick: Tick,
    ignores_fog: bool,
}

impl<'a> DecisionContext<'a> {
    pub fn new(
        own_army: &'a Army,
        enemy_army: &'a Army,
        own_visibility: &'a ArmyVisibility,
        current_tick: Tick,
        ignores_fog: bool,
    ) -> Self {
        Self {
            own_army,
            enemy_army,
            own_visibility,
            current_tick,
            ignores_fog,
        }
    }

    /// Get all own units
    pub fn own_units(&self) -> Vec<&BattleUnit> {
        self.own_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .collect()
    }

    /// Get enemy units that are visible (or all if ignoring fog)
    pub fn visible_enemy_units(&self) -> Vec<&BattleUnit> {
        self.enemy_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .filter(|u| self.ignores_fog || self.own_visibility.is_visible(u.position))
            .collect()
    }

    /// Get a specific own unit by ID
    pub fn get_own_unit(&self, unit_id: UnitId) -> Option<&BattleUnit> {
        self.own_army.get_unit(unit_id)
    }

    /// Check if a position is visible
    pub fn is_visible(&self, pos: BattleHexCoord) -> bool {
        self.ignores_fog || self.own_visibility.is_visible(pos)
    }

    /// Calculate own army effective strength
    pub fn own_effective_strength(&self) -> usize {
        self.own_army.effective_strength()
    }

    /// Calculate visible enemy effective strength
    pub fn visible_enemy_strength(&self) -> usize {
        self.visible_enemy_units()
            .iter()
            .map(|u| u.effective_strength())
            .sum()
    }

    /// Calculate strength ratio (own / enemy)
    /// Returns f32::MAX if no visible enemies
    pub fn strength_ratio(&self) -> f32 {
        let enemy_strength = self.visible_enemy_strength();
        if enemy_strength == 0 {
            return f32::MAX;
        }
        self.own_effective_strength() as f32 / enemy_strength as f32
    }

    /// Find weakest visible enemy unit
    pub fn weakest_enemy(&self) -> Option<&BattleUnit> {
        self.visible_enemy_units()
            .into_iter()
            .min_by_key(|u| u.effective_strength())
    }

    /// Find closest enemy to a position
    pub fn closest_enemy_to(&self, pos: BattleHexCoord) -> Option<&BattleUnit> {
        self.visible_enemy_units()
            .into_iter()
            .min_by_key(|u| u.position.distance(&pos) as u32)
    }

    /// Get all units that are routing
    pub fn routing_own_units(&self) -> Vec<&BattleUnit> {
        self.own_units()
            .into_iter()
            .filter(|u| u.is_broken())
            .collect()
    }

    /// Calculate own casualty percentage
    pub fn own_casualty_percentage(&self) -> f32 {
        let total = self.own_army.total_strength();
        if total == 0 {
            return 0.0;
        }
        let effective = self.own_army.effective_strength();
        1.0 - (effective as f32 / total as f32)
    }

    /// Get own HQ position
    pub fn hq_position(&self) -> BattleHexCoord {
        self.own_army.hq_position
    }

    /// Get enemy HQ position (known from pre-battle intel)
    /// Even with fog of war, commanders know roughly where the enemy came from.
    pub fn enemy_hq_position(&self) -> BattleHexCoord {
        self.enemy_army.hq_position
    }

    /// Get available couriers
    pub fn available_couriers(&self) -> usize {
        self.own_army.courier_pool.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{
        Army, ArmyId, BattleFormation, BattleUnit, Element, FormationId, UnitId, UnitStance,
    };
    use crate::battle::visibility::ArmyVisibility;
    use crate::core::types::EntityId;
    use std::collections::HashSet;

    fn create_test_unit(pos: BattleHexCoord) -> BattleUnit {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = pos;
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit
    }

    #[test]
    fn test_visible_enemy_units_respects_fog() {
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_army = Army::new(ArmyId::new(), EntityId::new());

        // Own unit at (5,5)
        let mut own_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        own_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(5, 5)));
        own_army.formations.push(own_formation);

        // Enemy units at (6,5) visible and (20,20) not visible
        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        enemy_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(6, 5)));
        enemy_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(20, 20)));
        enemy_army.formations.push(enemy_formation);

        // Visibility only includes (6,5)
        let mut visibility = ArmyVisibility::new();
        let mut visible = HashSet::new();
        visible.insert(BattleHexCoord::new(5, 5));
        visible.insert(BattleHexCoord::new(6, 5));
        visibility.update(visible);

        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);
        let visible_enemies = context.visible_enemy_units();

        assert_eq!(visible_enemies.len(), 1);
        assert_eq!(visible_enemies[0].position, BattleHexCoord::new(6, 5));
    }

    #[test]
    fn test_ignores_fog_sees_all() {
        let own_army = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_army = Army::new(ArmyId::new(), EntityId::new());

        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        enemy_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(6, 5)));
        enemy_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(20, 20)));
        enemy_army.formations.push(enemy_formation);

        // Empty visibility but ignores_fog = true
        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, true);
        let visible_enemies = context.visible_enemy_units();

        assert_eq!(visible_enemies.len(), 2);
    }

    #[test]
    fn test_strength_ratio_calculation() {
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_army = Army::new(ArmyId::new(), EntityId::new());

        let mut own_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        own_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(5, 5)));
        own_army.formations.push(own_formation);

        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        enemy_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(6, 5)));
        enemy_army.formations.push(enemy_formation);

        let mut visibility = ArmyVisibility::new();
        let mut visible = HashSet::new();
        visible.insert(BattleHexCoord::new(6, 5));
        visibility.update(visible);

        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);

        // Both have 50 strength, ratio should be 1.0
        assert!((context.strength_ratio() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_strength_ratio_no_visible_enemies() {
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_army = Army::new(ArmyId::new(), EntityId::new());

        let mut own_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        own_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(5, 5)));
        own_army.formations.push(own_formation);

        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        enemy_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(20, 20)));
        enemy_army.formations.push(enemy_formation);

        // No visibility - enemy not visible
        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);

        // Should return f32::MAX when no visible enemies
        assert_eq!(context.strength_ratio(), f32::MAX);
    }

    #[test]
    fn test_weakest_enemy() {
        let own_army = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_army = Army::new(ArmyId::new(), EntityId::new());

        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());

        // Strong unit
        let mut strong = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        strong.position = BattleHexCoord::new(6, 5);
        strong
            .elements
            .push(Element::new(vec![EntityId::new(); 100]));
        enemy_formation.units.push(strong);

        // Weak unit
        let mut weak = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        weak.position = BattleHexCoord::new(7, 5);
        weak.elements.push(Element::new(vec![EntityId::new(); 20]));
        enemy_formation.units.push(weak);

        enemy_army.formations.push(enemy_formation);

        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, true);
        let weakest = context.weakest_enemy().expect("Should find weakest");

        assert_eq!(weakest.effective_strength(), 20);
    }

    #[test]
    fn test_closest_enemy_to() {
        let own_army = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_army = Army::new(ArmyId::new(), EntityId::new());

        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());

        // Far enemy at (20, 20)
        enemy_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(20, 20)));

        // Close enemy at (3, 3)
        enemy_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(3, 3)));

        enemy_army.formations.push(enemy_formation);

        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, true);

        // Find closest to (5, 5)
        let closest = context
            .closest_enemy_to(BattleHexCoord::new(5, 5))
            .expect("Should find closest");
        assert_eq!(closest.position, BattleHexCoord::new(3, 3));
    }

    #[test]
    fn test_routing_own_units() {
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());

        let mut own_formation = BattleFormation::new(FormationId::new(), EntityId::new());

        // Normal unit
        own_formation
            .units
            .push(create_test_unit(BattleHexCoord::new(5, 5)));

        // Routing unit
        let mut routing = create_test_unit(BattleHexCoord::new(6, 5));
        routing.stance = UnitStance::Routing;
        own_formation.units.push(routing);

        own_army.formations.push(own_formation);

        let enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);

        let routing_units = context.routing_own_units();
        assert_eq!(routing_units.len(), 1);
        assert!(routing_units[0].is_broken());
    }

    #[test]
    fn test_own_casualty_percentage() {
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());

        let mut own_formation = BattleFormation::new(FormationId::new(), EntityId::new());

        // Unit with 100 strength and 30 casualties = 30% casualties
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 100]));
        unit.casualties = 30;
        own_formation.units.push(unit);

        own_army.formations.push(own_formation);

        let enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);

        let casualty_pct = context.own_casualty_percentage();
        assert!((casualty_pct - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_own_casualty_percentage_no_units() {
        let own_army = Army::new(ArmyId::new(), EntityId::new());
        let enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);

        // Empty army should return 0.0 casualty percentage
        assert_eq!(context.own_casualty_percentage(), 0.0);
    }

    #[test]
    fn test_hq_position() {
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());
        own_army.hq_position = BattleHexCoord::new(10, 15);

        let enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);

        assert_eq!(context.hq_position(), BattleHexCoord::new(10, 15));
    }

    #[test]
    fn test_available_couriers() {
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());
        own_army.courier_pool = vec![EntityId::new(); 5];

        let enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);

        assert_eq!(context.available_couriers(), 5);
    }

    #[test]
    fn test_own_units() {
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());

        let mut formation1 = BattleFormation::new(FormationId::new(), EntityId::new());
        formation1
            .units
            .push(create_test_unit(BattleHexCoord::new(5, 5)));
        formation1
            .units
            .push(create_test_unit(BattleHexCoord::new(6, 5)));
        own_army.formations.push(formation1);

        let mut formation2 = BattleFormation::new(FormationId::new(), EntityId::new());
        formation2
            .units
            .push(create_test_unit(BattleHexCoord::new(7, 5)));
        own_army.formations.push(formation2);

        let enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);

        assert_eq!(context.own_units().len(), 3);
    }

    #[test]
    fn test_is_visible() {
        let own_army = Army::new(ArmyId::new(), EntityId::new());
        let enemy_army = Army::new(ArmyId::new(), EntityId::new());

        let mut visibility = ArmyVisibility::new();
        let mut visible = HashSet::new();
        visible.insert(BattleHexCoord::new(5, 5));
        visibility.update(visible);

        // Without ignores_fog
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);
        assert!(context.is_visible(BattleHexCoord::new(5, 5)));
        assert!(!context.is_visible(BattleHexCoord::new(10, 10)));

        // With ignores_fog
        let context_fog = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, true);
        assert!(context_fog.is_visible(BattleHexCoord::new(5, 5)));
        assert!(context_fog.is_visible(BattleHexCoord::new(10, 10)));
    }

    #[test]
    fn test_get_own_unit() {
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());

        let unit_id = UnitId::new();
        let mut unit = BattleUnit::new(unit_id, UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 5);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));

        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        formation.units.push(unit);
        own_army.formations.push(formation);

        let enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);

        let found_unit = context.get_own_unit(unit_id);
        assert!(found_unit.is_some());
        assert_eq!(found_unit.unwrap().id, unit_id);

        // Non-existent unit
        let not_found = context.get_own_unit(UnitId::new());
        assert!(not_found.is_none());
    }
}
