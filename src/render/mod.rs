//! 2D Rendering system for Arc Citadel
//!
//! Provides visual representation of simulation state.
//! This module is READ-ONLY - it never modifies simulation state.

pub mod camera;
pub mod colors;

use crate::core::types::{Species, Vec2};
use crate::ecs::world::World;

/// Lightweight snapshot of an entity for rendering
#[derive(Debug, Clone)]
pub struct RenderEntity {
    pub position: Vec2,
    pub species: Species,
    pub health: f32,
    pub fatigue: f32,
}

/// Collects all renderable entities from the world into a reusable buffer.
/// Call this once per frame, passing the same buffer to avoid allocations.
pub fn collect_render_entities(world: &World, buffer: &mut Vec<RenderEntity>) {
    buffer.clear();

    // Collect living humans
    for i in 0..world.humans.ids.len() {
        if world.humans.alive[i] {
            let body = &world.humans.body_states[i];
            buffer.push(RenderEntity {
                position: world.humans.positions[i],
                species: Species::Human,
                health: body.overall_health,
                fatigue: body.fatigue,
            });
        }
    }

    // Collect living orcs
    for i in 0..world.orcs.ids.len() {
        if world.orcs.alive[i] {
            let body = &world.orcs.body_states[i];
            buffer.push(RenderEntity {
                position: world.orcs.positions[i],
                species: Species::Orc,
                health: body.overall_health,
                fatigue: body.fatigue,
            });
        }
    }
}
