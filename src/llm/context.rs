//! Gather game context for LLM prompts
//!
//! This module builds world state summaries that help the LLM parser
//! understand the current game situation for better command disambiguation.
//! The context includes information about entities, resources, threats,
//! and recent events.

use crate::core::types::Species;
use crate::ecs::world::World;

/// Game context for LLM prompts
///
/// Contains a summary of the current game state that helps the LLM
/// parser understand context and disambiguate commands.
pub struct GameContext {
    /// Name of the current location
    pub location_name: String,
    /// Total number of entities in the world
    pub entity_count: usize,
    /// Resources available at the current location
    pub available_resources: Vec<String>,
    /// Recent significant events
    pub recent_events: Vec<String>,
    /// Named entities the player might reference
    pub named_entities: Vec<NamedEntity>,
    /// Current threats or dangers
    pub threats: Vec<String>,
    /// Current game tick
    pub current_tick: u64,
}

/// A named entity that can be referenced in commands
pub struct NamedEntity {
    /// The entity's name
    pub name: String,
    /// The entity's species
    pub species: Species,
    /// Current role or occupation
    pub role: String,
    /// Current status (healthy, injured, etc.)
    pub status: String,
}

impl GameContext {
    /// Build a game context from the current world state
    ///
    /// # Arguments
    /// * `world` - The game world to extract context from
    ///
    /// # Returns
    /// A GameContext suitable for LLM prompt construction
    pub fn from_world(world: &World) -> Self {
        // Extract named entities (limit to 10 for prompt size)
        let named_entities: Vec<_> = world
            .humans
            .iter_living()
            .take(10)
            .map(|i| {
                let body = &world.humans.body_states[i];
                let needs = &world.humans.needs[i];
                let task_queue = &world.humans.task_queues[i];

                // Determine status based on body state and needs
                let status = if !body.can_act() {
                    "incapacitated".to_string()
                } else if body.pain > 0.5 {
                    "injured".to_string()
                } else if needs.rest > 0.8 {
                    "exhausted".to_string()
                } else if needs.food > 0.8 {
                    "hungry".to_string()
                } else {
                    "healthy".to_string()
                };

                // Determine role based on current task
                let role = if let Some(task) = task_queue.current() {
                    format!("{:?}", task.action).to_lowercase()
                } else {
                    "idle".to_string()
                };

                NamedEntity {
                    name: world.humans.names[i].clone(),
                    species: Species::Human,
                    role,
                    status,
                }
            })
            .collect();

        // Detect threats based on entity safety needs
        let threats: Vec<String> = world
            .humans
            .iter_living()
            .filter(|&i| world.humans.needs[i].safety > 0.7)
            .take(3)
            .map(|_| "danger nearby".to_string())
            .collect();

        Self {
            location_name: "Main Camp".into(),
            entity_count: world.entity_count(),
            available_resources: vec!["wood".into(), "stone".into(), "food".into()],
            recent_events: vec![],
            named_entities,
            threats,
            current_tick: world.current_tick,
        }
    }

    /// Generate a text summary of the context for LLM prompts
    ///
    /// # Returns
    /// A human-readable summary of the game context
    pub fn summary(&self) -> String {
        let mut s = String::new();

        // Location and time
        s.push_str(&format!("Location: {}\n", self.location_name));
        s.push_str(&format!("Time: Tick {}\n", self.current_tick));
        s.push_str(&format!("Population: {}\n", self.entity_count));

        // Named entities
        if !self.named_entities.is_empty() {
            s.push_str("\nKey Personnel:\n");
            for entity in &self.named_entities {
                s.push_str(&format!(
                    "- {} ({:?}, {}, {})\n",
                    entity.name, entity.species, entity.role, entity.status
                ));
            }
        }

        // Resources
        if !self.available_resources.is_empty() {
            s.push_str(&format!(
                "\nResources: {}\n",
                self.available_resources.join(", ")
            ));
        }

        // Recent events
        if !self.recent_events.is_empty() {
            s.push_str("\nRecent Events:\n");
            for event in &self.recent_events {
                s.push_str(&format!("- {}\n", event));
            }
        }

        // Threats
        if !self.threats.is_empty() {
            s.push_str(&format!("\nThreats: {}\n", self.threats.join(", ")));
        }

        s
    }

    /// Create an empty context for testing
    pub fn empty() -> Self {
        Self {
            location_name: "Unknown".into(),
            entity_count: 0,
            available_resources: vec![],
            recent_events: vec![],
            named_entities: vec![],
            threats: vec![],
            current_tick: 0,
        }
    }

    /// Add a recent event to the context
    pub fn add_event(&mut self, event: impl Into<String>) {
        self.recent_events.push(event.into());
        // Keep only the last 5 events
        if self.recent_events.len() > 5 {
            self.recent_events.remove(0);
        }
    }

    /// Add a threat to the context
    pub fn add_threat(&mut self, threat: impl Into<String>) {
        self.threats.push(threat.into());
    }

    /// Check if there are any active threats
    pub fn has_threats(&self) -> bool {
        !self.threats.is_empty()
    }

    /// Get entity by name (case-insensitive partial match)
    pub fn find_entity(&self, name: &str) -> Option<&NamedEntity> {
        let name_lower = name.to_lowercase();
        self.named_entities
            .iter()
            .find(|e| e.name.to_lowercase().contains(&name_lower))
    }

    /// Get all entities with a specific role
    pub fn entities_with_role(&self, role: &str) -> Vec<&NamedEntity> {
        let role_lower = role.to_lowercase();
        self.named_entities
            .iter()
            .filter(|e| e.role.to_lowercase().contains(&role_lower))
            .collect()
    }

    /// Get all healthy entities
    pub fn healthy_entities(&self) -> Vec<&NamedEntity> {
        self.named_entities
            .iter()
            .filter(|e| e.status == "healthy")
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::world::World;

    #[test]
    fn test_empty_context() {
        let ctx = GameContext::empty();
        assert_eq!(ctx.entity_count, 0);
        assert!(ctx.named_entities.is_empty());
        assert!(ctx.threats.is_empty());
    }

    #[test]
    fn test_context_from_world() {
        let mut world = World::new();
        world.spawn_human("Alice".into());
        world.spawn_human("Bob".into());

        let ctx = GameContext::from_world(&world);
        assert_eq!(ctx.entity_count, 2);
        assert_eq!(ctx.named_entities.len(), 2);

        // Check that names are preserved
        let names: Vec<_> = ctx.named_entities.iter().map(|e| &e.name).collect();
        assert!(names.contains(&&"Alice".to_string()));
        assert!(names.contains(&&"Bob".to_string()));
    }

    #[test]
    fn test_context_summary() {
        let mut ctx = GameContext::empty();
        ctx.location_name = "Test Camp".into();
        ctx.entity_count = 5;
        ctx.available_resources = vec!["wood".into(), "stone".into()];

        let summary = ctx.summary();
        assert!(summary.contains("Test Camp"));
        assert!(summary.contains("5"));
        assert!(summary.contains("wood"));
        assert!(summary.contains("stone"));
    }

    #[test]
    fn test_add_event() {
        let mut ctx = GameContext::empty();
        ctx.add_event("Enemy spotted");
        ctx.add_event("Wall completed");

        assert_eq!(ctx.recent_events.len(), 2);
        assert!(ctx.recent_events.contains(&"Enemy spotted".to_string()));
    }

    #[test]
    fn test_event_limit() {
        let mut ctx = GameContext::empty();
        for i in 0..10 {
            ctx.add_event(format!("Event {}", i));
        }

        // Should only keep last 5 events
        assert_eq!(ctx.recent_events.len(), 5);
        assert!(ctx.recent_events.contains(&"Event 9".to_string()));
        assert!(!ctx.recent_events.contains(&"Event 0".to_string()));
    }

    #[test]
    fn test_find_entity() {
        let mut ctx = GameContext::empty();
        ctx.named_entities.push(NamedEntity {
            name: "Marcus".into(),
            species: Species::Human,
            role: "guard".into(),
            status: "healthy".into(),
        });

        let found = ctx.find_entity("marc");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Marcus");

        let not_found = ctx.find_entity("Elena");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_entities_with_role() {
        let mut ctx = GameContext::empty();
        ctx.named_entities.push(NamedEntity {
            name: "Marcus".into(),
            species: Species::Human,
            role: "guard".into(),
            status: "healthy".into(),
        });
        ctx.named_entities.push(NamedEntity {
            name: "Elena".into(),
            species: Species::Human,
            role: "builder".into(),
            status: "healthy".into(),
        });
        ctx.named_entities.push(NamedEntity {
            name: "Thomas".into(),
            species: Species::Human,
            role: "guard".into(),
            status: "injured".into(),
        });

        let guards = ctx.entities_with_role("guard");
        assert_eq!(guards.len(), 2);
    }

    #[test]
    fn test_healthy_entities() {
        let mut ctx = GameContext::empty();
        ctx.named_entities.push(NamedEntity {
            name: "Marcus".into(),
            species: Species::Human,
            role: "guard".into(),
            status: "healthy".into(),
        });
        ctx.named_entities.push(NamedEntity {
            name: "Elena".into(),
            species: Species::Human,
            role: "builder".into(),
            status: "injured".into(),
        });

        let healthy = ctx.healthy_entities();
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0].name, "Marcus");
    }

    #[test]
    fn test_has_threats() {
        let mut ctx = GameContext::empty();
        assert!(!ctx.has_threats());

        ctx.add_threat("Enemy nearby");
        assert!(ctx.has_threats());
    }
}
