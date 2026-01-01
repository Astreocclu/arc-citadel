//! Hierarchy operations for polity parent-child relationships
//!
//! Provides queries for traversing the polity hierarchy tree.
//! All polities form a forest (collection of trees) where each tree
//! is rooted at a sovereign polity.

use std::collections::HashMap;
use crate::core::types::PolityId;
use crate::aggregate::polity::Polity;

/// Get the sovereign (root) polity for a given polity.
/// Returns None if the polity doesn't exist.
/// A sovereign polity returns itself.
pub fn get_sovereign(polity_id: PolityId, polities: &HashMap<PolityId, Polity>) -> Option<PolityId> {
    let mut current = polity_id;
    let mut visited = std::collections::HashSet::new();

    loop {
        // Prevent infinite loops from corrupted data
        if !visited.insert(current) {
            return None; // Cycle detected
        }

        let polity = polities.get(&current)?;

        match polity.parent {
            None => return Some(current), // Found sovereign
            Some(parent_id) => current = parent_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{PolityTier, GovernmentType, RulerId};
    use crate::aggregate::polity::{PolityType, CulturalDrift, SpeciesState, HumanState};
    use crate::core::types::Species;

    fn make_polity(id: u32, parent: Option<u32>) -> Polity {
        Polity {
            id: PolityId(id),
            name: format!("Polity {}", id),
            species: Species::Human,
            polity_type: PolityType::Kingdom,
            tier: PolityTier::Kingdom,
            government: GovernmentType::Autocracy,
            parent: parent.map(PolityId),
            rulers: vec![RulerId(id)],
            council_roles: HashMap::new(),
            capital: 0,
            population: 1000,
            military_strength: 10.0,
            economic_strength: 10.0,
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state: SpeciesState::Human(HumanState::default()),
            alive: true,
        }
    }

    fn make_polity_map(polities: Vec<Polity>) -> HashMap<PolityId, Polity> {
        polities.into_iter().map(|p| (p.id, p)).collect()
    }

    #[test]
    fn test_get_sovereign_self() {
        // Sovereign polity returns itself
        let polities = make_polity_map(vec![make_polity(1, None)]);
        assert_eq!(get_sovereign(PolityId(1), &polities), Some(PolityId(1)));
    }

    #[test]
    fn test_get_sovereign_chain() {
        // Empire(1) -> Kingdom(2) -> Duchy(3)
        let polities = make_polity_map(vec![
            make_polity(1, None),      // Empire (sovereign)
            make_polity(2, Some(1)),   // Kingdom under Empire
            make_polity(3, Some(2)),   // Duchy under Kingdom
        ]);

        assert_eq!(get_sovereign(PolityId(1), &polities), Some(PolityId(1)));
        assert_eq!(get_sovereign(PolityId(2), &polities), Some(PolityId(1)));
        assert_eq!(get_sovereign(PolityId(3), &polities), Some(PolityId(1)));
    }

    #[test]
    fn test_get_sovereign_missing() {
        let polities = make_polity_map(vec![make_polity(1, None)]);
        assert_eq!(get_sovereign(PolityId(999), &polities), None);
    }
}
