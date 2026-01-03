//! Color definitions for species and visual states

use crate::core::types::Species;

/// RGBA color (0.0 to 1.0 per channel)
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Darken color by a factor (0.0 = black, 1.0 = unchanged)
    pub fn darken(&self, factor: f32) -> Self {
        Self {
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
            a: self.a,
        }
    }
}

/// Background color for the renderer
pub const BACKGROUND: Color = Color::new(0.1, 0.1, 0.12, 1.0);

/// Get the base color for a species
pub fn species_color(species: Species) -> Color {
    match species {
        // Playable races
        Species::Human => Color::new(0.2, 0.6, 0.9, 1.0),      // Blue
        Species::Dwarf => Color::new(0.8, 0.5, 0.2, 1.0),      // Brown/orange
        Species::Elf => Color::new(0.3, 0.9, 0.5, 1.0),        // Light green
        Species::Orc => Color::new(0.1, 0.7, 0.2, 1.0),        // Dark green

        // Humanoid monsters
        Species::Kobold => Color::new(0.6, 0.4, 0.2, 1.0),     // Tan
        Species::Gnoll => Color::new(0.7, 0.5, 0.3, 1.0),      // Yellowish brown
        Species::Lizardfolk => Color::new(0.2, 0.5, 0.3, 1.0), // Swamp green
        Species::Hobgoblin => Color::new(0.8, 0.4, 0.1, 1.0),  // Orange-red
        Species::Ogre => Color::new(0.5, 0.4, 0.3, 1.0),       // Muddy brown
        Species::Goblin => Color::new(0.4, 0.6, 0.2, 1.0),     // Yellow-green

        // Mythical humanoids
        Species::Harpy => Color::new(0.8, 0.7, 0.5, 1.0),      // Feather tan
        Species::Centaur => Color::new(0.6, 0.4, 0.3, 1.0),    // Chestnut
        Species::Minotaur => Color::new(0.5, 0.3, 0.2, 1.0),   // Dark brown
        Species::Satyr => Color::new(0.7, 0.5, 0.4, 1.0),      // Russet

        // Nature spirits
        Species::Dryad => Color::new(0.4, 0.7, 0.3, 1.0),      // Forest green
        Species::Fey => Color::new(0.7, 0.5, 0.9, 1.0),        // Purple/violet

        // Large monsters
        Species::Troll => Color::new(0.3, 0.5, 0.3, 1.0),      // Moss green
        Species::StoneGiants => Color::new(0.5, 0.5, 0.5, 1.0),// Gray

        // Magical/elemental
        Species::AbyssalDemons => Color::new(0.8, 0.1, 0.1, 1.0), // Blood red
        Species::Elemental => Color::new(0.9, 0.6, 0.2, 1.0),  // Fiery orange
        Species::Golem => Color::new(0.4, 0.4, 0.5, 1.0),      // Stone gray

        // Aquatic
        Species::Merfolk => Color::new(0.2, 0.6, 0.8, 1.0),    // Ocean blue
        Species::Naga => Color::new(0.3, 0.7, 0.6, 1.0),       // Teal

        // Undead
        Species::Revenant => Color::new(0.4, 0.4, 0.5, 1.0),   // Ashen gray
        Species::Vampire => Color::new(0.6, 0.1, 0.2, 1.0),    // Dark crimson

        // Lycanthropes
        Species::Lupine => Color::new(0.5, 0.4, 0.3, 1.0),     // Fur brown
    }
}

/// Modulate color based on entity health (lower health = more red tint)
pub fn health_tint(base: Color, health: f32) -> Color {
    let health_clamped = health.clamp(0.0, 1.0);
    // Interpolate toward red as health decreases
    Color {
        r: base.r + (1.0 - base.r) * (1.0 - health_clamped) * 0.5,
        g: base.g * health_clamped,
        b: base.b * health_clamped,
        a: base.a,
    }
}

/// Modulate color based on fatigue (higher fatigue = darker)
pub fn fatigue_tint(base: Color, fatigue: f32) -> Color {
    let fatigue_clamped = fatigue.clamp(0.0, 1.0);
    let brightness = 1.0 - fatigue_clamped * 0.4; // Max 40% darkening
    base.darken(brightness)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_species_colors_unique() {
        // Ensure at least a few key species have distinct colors
        let human = species_color(Species::Human);
        let orc = species_color(Species::Orc);
        let dwarf = species_color(Species::Dwarf);

        // Colors should be different (not exactly equal)
        assert!(human.r != orc.r || human.g != orc.g || human.b != orc.b);
        assert!(human.r != dwarf.r || human.g != dwarf.g || human.b != dwarf.b);
    }

    #[test]
    fn test_health_tint_full_health() {
        let base = Color::new(0.5, 0.5, 0.5, 1.0);
        let tinted = health_tint(base, 1.0);
        // Full health should be close to original
        assert!((tinted.r - base.r).abs() < 0.01);
    }

    #[test]
    fn test_fatigue_darkens() {
        let base = Color::new(1.0, 1.0, 1.0, 1.0);
        let tired = fatigue_tint(base, 1.0);
        // Max fatigue should darken by 40%
        assert!((tired.r - 0.6).abs() < 0.01);
    }
}
