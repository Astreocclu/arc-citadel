//! Species-specific chunk formation and decay modifiers
//!
//! Different species learn different domains at different rates.

use crate::skills::ChunkDomain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Modifiers for chunk formation in a specific domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainModifier {
    /// Multiplier for chunk formation rate (1.0 = normal)
    pub formation_rate: f32,
    /// Multiplier for rust/decay rate (1.0 = normal, 0.0 = no decay)
    pub decay_rate: f32,
    /// Maximum encoding depth achievable (0.95-0.995)
    pub max_encoding: f32,
}

impl Default for DomainModifier {
    fn default() -> Self {
        Self {
            formation_rate: 1.0,
            decay_rate: 1.0,
            max_encoding: 0.95,
        }
    }
}

/// Species-specific skill modifiers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpeciesChunkModifiers {
    /// Per-domain modifiers
    pub domains: HashMap<ChunkDomain, DomainModifier>,
    /// Global learning rate multiplier
    pub base_learning_rate: f32,
    /// Cross-species social penalty (0.0-1.0, where 1.0 = no penalty)
    pub cross_species_social: f32,
}

impl SpeciesChunkModifiers {
    /// Create default human modifiers (baseline)
    pub fn human() -> Self {
        Self {
            domains: HashMap::new(), // All defaults
            base_learning_rate: 1.0,
            cross_species_social: 0.8, // 20% penalty with non-humans
        }
    }

    /// Create dwarf modifiers (craft-focused)
    pub fn dwarf() -> Self {
        let mut domains = HashMap::new();

        // Dwarves excel at crafting
        domains.insert(
            ChunkDomain::Craft,
            DomainModifier {
                formation_rate: 1.5, // 50% faster craft learning
                decay_rate: 0.3,     // Craft skills barely rust
                max_encoding: 0.99,  // Higher ceiling
            },
        );

        // Good at physical labor
        domains.insert(
            ChunkDomain::Physical,
            DomainModifier {
                formation_rate: 1.2,
                decay_rate: 0.5,
                max_encoding: 0.95,
            },
        );

        // Slower socially
        domains.insert(
            ChunkDomain::Social,
            DomainModifier {
                formation_rate: 0.8,
                decay_rate: 1.0,
                max_encoding: 0.9,
            },
        );

        Self {
            domains,
            base_learning_rate: 1.0,
            cross_species_social: 0.7, // 30% penalty with non-dwarves
        }
    }

    /// Create elf modifiers (long-lived, slow but deep)
    pub fn elf() -> Self {
        let mut domains = HashMap::new();

        // Elves learn slowly but deeply in all domains
        for domain in ChunkDomain::all() {
            domains.insert(
                *domain,
                DomainModifier {
                    formation_rate: 0.6,  // Slower learning
                    decay_rate: 0.05,     // Almost no rust
                    max_encoding: 0.995,  // Higher ceiling
                },
            );
        }

        Self {
            domains,
            base_learning_rate: 0.8,
            cross_species_social: 0.6, // 40% penalty with non-elves
        }
    }

    /// Create orc modifiers (combat-focused)
    pub fn orc() -> Self {
        let mut domains = HashMap::new();

        // Orcs excel at combat
        domains.insert(
            ChunkDomain::Combat,
            DomainModifier {
                formation_rate: 1.4,
                decay_rate: 0.6,
                max_encoding: 0.95,
            },
        );

        // Physical strength
        domains.insert(
            ChunkDomain::Physical,
            DomainModifier {
                formation_rate: 1.3,
                decay_rate: 0.7,
                max_encoding: 0.95,
            },
        );

        // Weaker at scholarly pursuits
        domains.insert(
            ChunkDomain::Knowledge,
            DomainModifier {
                formation_rate: 0.6,
                decay_rate: 1.5,
                max_encoding: 0.8,
            },
        );

        Self {
            domains,
            base_learning_rate: 1.0,
            cross_species_social: 0.6,
        }
    }

    /// Get modifier for a specific domain (defaults if not set)
    pub fn get_domain(&self, domain: ChunkDomain) -> DomainModifier {
        self.domains.get(&domain).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_baseline() {
        let mods = SpeciesChunkModifiers::human();
        let craft = mods.get_domain(ChunkDomain::Craft);

        assert_eq!(craft.formation_rate, 1.0);
        assert_eq!(craft.decay_rate, 1.0);
    }

    #[test]
    fn test_dwarf_craft_bonus() {
        let mods = SpeciesChunkModifiers::dwarf();
        let craft = mods.get_domain(ChunkDomain::Craft);

        assert_eq!(craft.formation_rate, 1.5);
        assert_eq!(craft.decay_rate, 0.3);
    }

    #[test]
    fn test_elf_slow_learning() {
        let mods = SpeciesChunkModifiers::elf();
        let combat = mods.get_domain(ChunkDomain::Combat);

        assert_eq!(combat.formation_rate, 0.6);
        assert_eq!(combat.decay_rate, 0.05);
        assert_eq!(combat.max_encoding, 0.995);
    }
}
