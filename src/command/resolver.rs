//! Intent resolution - converts ParsedIntent subjects/locations to concrete entities/positions

use crate::core::types::{EntityId, Vec2};
use crate::ecs::world::World;
use crate::llm::parser::ParsedIntent;

/// Result of resolving an intent's subjects and location
#[derive(Debug, Clone)]
pub struct IntentResolution {
    /// Entities that matched the subject criteria
    pub subjects: Vec<SubjectMatch>,
    /// Resolved location (if specified)
    pub location: Option<Vec2>,
    /// Target entity (for "attack that orc" style commands)
    pub target_entity: Option<EntityId>,
    /// Any ambiguity or issues encountered
    pub notes: Vec<String>,
}

/// A matched subject with confidence
#[derive(Debug, Clone)]
pub struct SubjectMatch {
    pub entity_id: EntityId,
    pub name: String,
    pub match_reason: MatchReason,
}

#[derive(Debug, Clone)]
pub enum MatchReason {
    ExactName,
    PartialName,
    Qualification { skill: String, level: f32 },
    Everyone,
}

/// Resolves ParsedIntent subjects and locations to concrete entities/positions
pub struct IntentResolver<'a> {
    world: &'a World,
}

impl<'a> IntentResolver<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }

    /// Resolve a parsed intent to concrete entities and positions
    pub fn resolve(&self, intent: &ParsedIntent) -> IntentResolution {
        let subjects = self.resolve_subjects(&intent.subjects);
        let location = self.resolve_location(&intent.location);
        let target_entity = self.resolve_target(&intent.target);

        IntentResolution {
            subjects,
            location,
            target_entity,
            notes: Vec::new(),
        }
    }

    fn resolve_subjects(&self, subjects: &Option<Vec<String>>) -> Vec<SubjectMatch> {
        let Some(subject_specs) = subjects else {
            return Vec::new();
        };

        let mut matches = Vec::new();

        for spec in subject_specs {
            let spec_lower = spec.to_lowercase();

            // Check for "everyone" / "all"
            if spec_lower == "everyone" || spec_lower == "all" {
                for (i, name) in self.world.humans.names.iter().enumerate() {
                    if self.world.humans.alive[i] {
                        matches.push(SubjectMatch {
                            entity_id: self.world.humans.ids[i],
                            name: name.clone(),
                            match_reason: MatchReason::Everyone,
                        });
                    }
                }
                continue;
            }

            // Check for qualification patterns
            if let Some(qual_match) = self.resolve_qualification(&spec_lower) {
                matches.extend(qual_match);
                continue;
            }

            // Try exact name match
            if let Some(m) = self.find_by_name(&spec) {
                matches.push(m);
            }
        }

        matches
    }

    fn find_by_name(&self, name: &str) -> Option<SubjectMatch> {
        let name_lower = name.to_lowercase();

        for (i, entity_name) in self.world.humans.names.iter().enumerate() {
            if !self.world.humans.alive[i] {
                continue;
            }

            if entity_name.to_lowercase() == name_lower {
                return Some(SubjectMatch {
                    entity_id: self.world.humans.ids[i],
                    name: entity_name.clone(),
                    match_reason: MatchReason::ExactName,
                });
            }

            // Partial match (first name)
            if entity_name.to_lowercase().starts_with(&name_lower) {
                return Some(SubjectMatch {
                    entity_id: self.world.humans.ids[i],
                    name: entity_name.clone(),
                    match_reason: MatchReason::PartialName,
                });
            }
        }

        None
    }

    fn resolve_qualification(&self, spec: &str) -> Option<Vec<SubjectMatch>> {
        // Pattern: "a qualified builder", "the best builder", "a skilled fighter"
        let patterns = [
            ("builder", "building_skills"),
            ("fighter", "combat"),
            ("soldier", "combat"),
        ];

        for (keyword, skill_type) in patterns {
            if spec.contains(keyword) {
                return Some(self.find_by_skill(skill_type, 0.5)); // min skill 0.5
            }
        }

        None
    }

    fn find_by_skill(&self, skill_type: &str, min_level: f32) -> Vec<SubjectMatch> {
        let mut matches = Vec::new();

        for i in 0..self.world.humans.ids.len() {
            if !self.world.humans.alive[i] {
                continue;
            }

            let skill_level = match skill_type {
                "building_skills" => self.world.humans.building_skills[i],
                // Add more skill types as needed
                _ => 0.0,
            };

            if skill_level >= min_level {
                matches.push(SubjectMatch {
                    entity_id: self.world.humans.ids[i],
                    name: self.world.humans.names[i].clone(),
                    match_reason: MatchReason::Qualification {
                        skill: skill_type.to_string(),
                        level: skill_level,
                    },
                });
            }
        }

        // Sort by skill level descending
        matches.sort_by(|a, b| {
            let a_level = match &a.match_reason {
                MatchReason::Qualification { level, .. } => *level,
                _ => 0.0,
            };
            let b_level = match &b.match_reason {
                MatchReason::Qualification { level, .. } => *level,
                _ => 0.0,
            };
            b_level
                .partial_cmp(&a_level)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    fn resolve_location(&self, location: &Option<String>) -> Option<Vec2> {
        let loc = location.as_ref()?;
        let loc_lower = loc.to_lowercase();

        // Named locations (expand as needed)
        if loc_lower.contains("center") || loc_lower.contains("middle") {
            return Some(Vec2::new(100.0, 100.0));
        }
        if loc_lower.contains("east") {
            return Some(Vec2::new(180.0, 100.0));
        }
        if loc_lower.contains("west") {
            return Some(Vec2::new(20.0, 100.0));
        }
        if loc_lower.contains("north") {
            return Some(Vec2::new(100.0, 180.0));
        }
        if loc_lower.contains("south") {
            return Some(Vec2::new(100.0, 20.0));
        }

        // TODO: Parse coordinates like "50, 100"

        None
    }

    fn resolve_target(&self, _target: &Option<String>) -> Option<EntityId> {
        // For now, no entity targeting by description
        // Would need spatial queries for "that orc" or "nearest enemy"
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_by_name() {
        let mut world = World::new();
        let marcus_id = world.spawn_human("Marcus".into());

        let resolver = IntentResolver::new(&world);
        let matches = resolver.resolve_subjects(&Some(vec!["Marcus".to_string()]));

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].entity_id, marcus_id);
    }

    #[test]
    fn test_resolve_everyone() {
        let mut world = World::new();
        world.spawn_human("Alice".into());
        world.spawn_human("Bob".into());

        let resolver = IntentResolver::new(&world);
        let matches = resolver.resolve_subjects(&Some(vec!["everyone".to_string()]));

        assert_eq!(matches.len(), 2);
    }
}
