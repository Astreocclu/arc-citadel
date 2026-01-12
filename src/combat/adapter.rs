//! Combat adapter - bridges tick.rs entity data to combat resolution
//!
//! This module provides an adapter layer that converts entity state
//! (from HumanArchetype) into the Combatant structs used by the
//! combat resolution system.

use crate::combat::resolution::{resolve_exchange, Combatant, ExchangeResult};
use crate::combat::{ArmorProperties, CombatSkill, CombatStance, WeaponProperties};
use crate::core::types::EntityId;
use crate::ecs::world::World;

/// Adapter to resolve combat between entities using their actual stats
pub struct CombatAdapter<'a> {
    world: &'a World,
}

impl<'a> CombatAdapter<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }

    /// Resolve an attack between two human entities
    ///
    /// # Arguments
    /// * `attacker_idx` - Index of the attacker in HumanArchetype
    /// * `defender_id` - EntityId of the defender
    /// * `_skill_modifier` - Modifier from skill check (reserved for future use)
    ///
    /// # Returns
    /// Combat result if defender was found, None otherwise
    pub fn resolve_attack(
        &self,
        attacker_idx: usize,
        defender_id: EntityId,
        _skill_modifier: f32,
    ) -> Option<CombatResult> {
        // Find defender
        let defender_idx = self.world.humans.index_of(defender_id)?;

        // Build combatants from entity data
        let attacker = self.build_combatant(attacker_idx, CombatStance::Pressing);
        let defender = self.build_combatant(defender_idx, CombatStance::Neutral);

        let exchange = resolve_exchange(&attacker, &defender);

        Some(CombatResult {
            attacker_idx,
            defender_idx,
            exchange,
        })
    }

    fn build_combatant(&self, idx: usize, stance: CombatStance) -> Combatant {
        // Derive combat skill from chunk library
        let skill = CombatSkill::from_chunk_library(&self.world.humans.chunk_libraries[idx]);

        Combatant {
            weapon: WeaponProperties::fists(), // Default: unarmed
            armor: ArmorProperties::none(),    // Default: unarmored
            stance,
            skill,
        }
    }
}

/// Result of combat resolution
pub struct CombatResult {
    pub attacker_idx: usize,
    pub defender_idx: usize,
    pub exchange: ExchangeResult,
}

impl CombatResult {
    /// Did the attacker successfully hit the defender?
    pub fn attacker_success(&self) -> bool {
        self.exchange.defender_hit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::world::World;

    #[test]
    fn test_combat_adapter_creation() {
        let world = World::new();
        let _adapter = CombatAdapter::new(&world);
    }

    #[test]
    fn test_resolve_attack_no_defender() {
        let world = World::new();
        let adapter = CombatAdapter::new(&world);

        // Try to attack a non-existent entity
        let result = adapter.resolve_attack(0, EntityId::new(), 1.0);
        assert!(result.is_none());
    }
}
