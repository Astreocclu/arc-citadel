//! Diplomacy and relations system

use crate::aggregate::world::AggregateWorld;

/// Decay relations over time
pub fn decay_relations(world: &mut AggregateWorld) {
    for polity in &mut world.polities {
        if !polity.alive {
            continue;
        }

        for relation in polity.relations.values_mut() {
            // Opinion decays toward neutral
            if relation.opinion > 0 {
                relation.opinion = (relation.opinion - 1).max(0);
            } else if relation.opinion < 0 {
                relation.opinion = (relation.opinion + 1).min(0);
            }

            // Trust decays slower
            if relation.trust > 0 && world.year % 5 == 0 {
                relation.trust = (relation.trust - 1).max(0);
            }
        }
    }
}
