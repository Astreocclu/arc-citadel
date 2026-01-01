//! Core type definitions used throughout the codebase

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

/// Game tick counter (simulation time unit)
pub type Tick = u64;

/// Location identifier for campaign map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocationId(pub u32);

/// Flow field identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlowFieldId(pub u32);

/// Species enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Species {
    Human,
    Dwarf,
    Elf,
}

/// 2D position
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Self { x: self.x / len, y: self.y / len }
        } else {
            Self::default()
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

/// Unique identifier for polities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolityId(pub u32);

impl PolityId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polity_id_equality() {
        let a = PolityId(1);
        let b = PolityId(1);
        let c = PolityId(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_polity_id_hash() {
        use std::collections::HashMap;
        let mut map: HashMap<PolityId, &str> = HashMap::new();
        map.insert(PolityId(1), "empire");
        assert_eq!(map.get(&PolityId(1)), Some(&"empire"));
    }
}
