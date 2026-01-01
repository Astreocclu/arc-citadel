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

/// Get direct vassals of a polity (immediate children only)
pub fn get_vassals(polity_id: PolityId, polities: &HashMap<PolityId, Polity>) -> Vec<PolityId> {
    polities
        .values()
        .filter(|p| p.parent == Some(polity_id))
        .map(|p| p.id)
        .collect()
}

/// Get all vassals recursively (all descendants)
pub fn get_all_vassals(polity_id: PolityId, polities: &HashMap<PolityId, Polity>) -> Vec<PolityId> {
    let mut result = Vec::new();
    let mut stack = get_vassals(polity_id, polities);

    while let Some(vassal_id) = stack.pop() {
        result.push(vassal_id);
        stack.extend(get_vassals(vassal_id, polities));
    }

    result
}

/// Check if subject is a vassal of lord (at any level)
pub fn is_vassal_of(subject: PolityId, lord: PolityId, polities: &HashMap<PolityId, Polity>) -> bool {
    if subject == lord {
        return false; // Not a vassal of yourself
    }

    let mut current = subject;
    let mut visited = std::collections::HashSet::new();

    while let Some(polity) = polities.get(&current) {
        if !visited.insert(current) {
            return false; // Cycle detected
        }

        match polity.parent {
            Some(parent_id) if parent_id == lord => return true,
            Some(parent_id) => current = parent_id,
            None => return false, // Reached sovereign without finding lord
        }
    }

    false
}

/// Check if two polities are in the same realm (share a sovereign)
pub fn same_realm(a: PolityId, b: PolityId, polities: &HashMap<PolityId, Polity>) -> bool {
    match (get_sovereign(a, polities), get_sovereign(b, polities)) {
        (Some(sov_a), Some(sov_b)) => sov_a == sov_b,
        _ => false,
    }
}

/// Get the liege (immediate parent) of a polity
pub fn get_liege(polity_id: PolityId, polities: &HashMap<PolityId, Polity>) -> Option<PolityId> {
    polities.get(&polity_id).and_then(|p| p.parent)
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

    #[test]
    fn test_get_vassals_direct() {
        // Empire(1) has vassals Kingdom(2) and Kingdom(3)
        let polities = make_polity_map(vec![
            make_polity(1, None),
            make_polity(2, Some(1)),
            make_polity(3, Some(1)),
            make_polity(4, Some(2)), // Vassal of 2, not 1
        ]);

        let vassals = get_vassals(PolityId(1), &polities);
        assert_eq!(vassals.len(), 2);
        assert!(vassals.contains(&PolityId(2)));
        assert!(vassals.contains(&PolityId(3)));
        assert!(!vassals.contains(&PolityId(4))); // Not direct vassal
    }

    #[test]
    fn test_get_all_vassals_recursive() {
        // Empire(1) -> Kingdom(2) -> Duchy(4)
        //           -> Kingdom(3)
        let polities = make_polity_map(vec![
            make_polity(1, None),
            make_polity(2, Some(1)),
            make_polity(3, Some(1)),
            make_polity(4, Some(2)),
        ]);

        let all_vassals = get_all_vassals(PolityId(1), &polities);
        assert_eq!(all_vassals.len(), 3);
        assert!(all_vassals.contains(&PolityId(2)));
        assert!(all_vassals.contains(&PolityId(3)));
        assert!(all_vassals.contains(&PolityId(4)));
    }

    #[test]
    fn test_get_vassals_none() {
        let polities = make_polity_map(vec![make_polity(1, None)]);
        let vassals = get_vassals(PolityId(1), &polities);
        assert!(vassals.is_empty());
    }

    #[test]
    fn test_is_vassal_of_direct() {
        let polities = make_polity_map(vec![
            make_polity(1, None),
            make_polity(2, Some(1)),
        ]);

        assert!(is_vassal_of(PolityId(2), PolityId(1), &polities));
        assert!(!is_vassal_of(PolityId(1), PolityId(2), &polities));
    }

    #[test]
    fn test_is_vassal_of_indirect() {
        // Empire(1) -> Kingdom(2) -> Duchy(3)
        let polities = make_polity_map(vec![
            make_polity(1, None),
            make_polity(2, Some(1)),
            make_polity(3, Some(2)),
        ]);

        // Duchy is vassal of both Kingdom and Empire
        assert!(is_vassal_of(PolityId(3), PolityId(2), &polities));
        assert!(is_vassal_of(PolityId(3), PolityId(1), &polities));
    }

    #[test]
    fn test_same_realm() {
        // Empire(1) -> Kingdom(2) -> Duchy(3)
        //           -> Kingdom(4)
        let polities = make_polity_map(vec![
            make_polity(1, None),
            make_polity(2, Some(1)),
            make_polity(3, Some(2)),
            make_polity(4, Some(1)),
            make_polity(5, None), // Different sovereign
        ]);

        assert!(same_realm(PolityId(2), PolityId(3), &polities));
        assert!(same_realm(PolityId(2), PolityId(4), &polities));
        assert!(same_realm(PolityId(3), PolityId(4), &polities));
        assert!(!same_realm(PolityId(2), PolityId(5), &polities));
    }

    #[test]
    fn test_get_liege() {
        let polities = make_polity_map(vec![
            make_polity(1, None),
            make_polity(2, Some(1)),
            make_polity(3, Some(2)),
        ]);

        assert_eq!(get_liege(PolityId(1), &polities), None);
        assert_eq!(get_liege(PolityId(2), &polities), Some(PolityId(1)));
        assert_eq!(get_liege(PolityId(3), &polities), Some(PolityId(2)));
    }
}
