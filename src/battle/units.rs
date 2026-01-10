//! Unit hierarchy: Element → Unit → Formation → Army
//!
//! Elements are the smallest tactical grouping (5-10 individuals).
//! Units combine elements into cohesive fighting groups.
//! Formations organize units under a commander.
//! Armies combine formations for a battle.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::battle::hex::{BattleHexCoord, HexDirection};
use crate::battle::unit_type::UnitType;
use crate::core::types::EntityId;

/// Unique identifier for armies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArmyId(pub Uuid);

impl ArmyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ArmyId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for formations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FormationId(pub Uuid);

impl FormationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FormationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitId(pub Uuid);

impl UnitId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UnitId {
    fn default() -> Self {
        Self::new()
    }
}

/// Smallest tactical grouping (5-10 individuals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub entities: Vec<EntityId>,
}

impl Element {
    pub fn new(entities: Vec<EntityId>) -> Self {
        Self { entities }
    }

    pub fn strength(&self) -> usize {
        self.entities.len()
    }
}

/// Combat stance for units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UnitStance {
    #[default]
    Formed, // In formation, ready
    Moving,   // Moving to position
    Engaged,  // In combat
    Shaken,   // Morale damaged
    Routing,  // Fleeing
    Rallying, // Reforming after rout
    Patrol,   // Scouting stance
    Alert,    // High awareness
}

/// Formation shape for units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormationShape {
    Line { depth: u8 },
    Column { width: u8 },
    Wedge { angle: f32 },
    Square,
    Skirmish { dispersion: f32 },
}

impl Default for FormationShape {
    fn default() -> Self {
        FormationShape::Line { depth: 2 }
    }
}

/// A military unit (collection of elements)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleUnit {
    pub id: UnitId,
    pub leader: Option<EntityId>,
    pub elements: Vec<Element>,
    pub unit_type: UnitType,

    // Position
    pub position: BattleHexCoord,
    pub facing: HexDirection,

    // State
    pub stance: UnitStance,
    pub formation_shape: FormationShape,
    pub cohesion: f32, // 0.0 (scattered) to 1.0 (tight)
    pub fatigue: f32,  // 0.0 (fresh) to 1.0 (exhausted)
    pub stress: f32,   // Accumulated stress

    // Rally tracking
    pub rallying_since: Option<u64>, // Tick when unit started rallying

    // Casualties
    pub casualties: u32,
}

impl BattleUnit {
    pub fn new(id: UnitId, unit_type: UnitType) -> Self {
        Self {
            id,
            leader: None,
            elements: Vec::new(),
            unit_type,
            position: BattleHexCoord::default(),
            facing: HexDirection::default(),
            stance: UnitStance::default(),
            formation_shape: FormationShape::default(),
            cohesion: 1.0,
            fatigue: 0.0,
            stress: 0.0,
            rallying_since: None,
            casualties: 0,
        }
    }

    /// Total strength (number of combatants)
    pub fn strength(&self) -> usize {
        self.elements.iter().map(|e| e.strength()).sum()
    }

    /// Effective strength (accounting for casualties)
    pub fn effective_strength(&self) -> usize {
        self.strength().saturating_sub(self.casualties as usize)
    }

    /// Is this unit broken?
    pub fn is_broken(&self) -> bool {
        matches!(self.stance, UnitStance::Routing)
    }

    /// Can this unit fight?
    pub fn can_fight(&self) -> bool {
        !matches!(self.stance, UnitStance::Routing | UnitStance::Rallying)
            && self.effective_strength() > 0
    }

    /// Is this unit engaged in combat?
    pub fn is_engaged(&self) -> bool {
        matches!(self.stance, UnitStance::Engaged)
    }

    /// Get stress threshold based on unit type and state
    pub fn stress_threshold(&self) -> f32 {
        let base = self.unit_type.default_properties().base_stress_threshold;

        // Additive modifiers
        let mut threshold = base;

        // High cohesion helps
        if self.cohesion > 0.8 {
            threshold += 0.1;
        }

        // Fatigue hurts
        threshold -= self.fatigue * 0.2;

        threshold.max(0.3)
    }
}

/// A formation (collection of units under a commander)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleFormation {
    pub id: FormationId,
    pub commander: EntityId,
    pub units: Vec<BattleUnit>,
    pub name: String,
}

impl BattleFormation {
    pub fn new(id: FormationId, commander: EntityId) -> Self {
        Self {
            id,
            commander,
            units: Vec::new(),
            name: String::new(),
        }
    }

    /// Total strength of all units
    pub fn total_strength(&self) -> usize {
        self.units.iter().map(|u| u.strength()).sum()
    }

    /// Effective strength of all units
    pub fn effective_strength(&self) -> usize {
        self.units.iter().map(|u| u.effective_strength()).sum()
    }

    /// Percentage of formation routing
    pub fn percentage_routing(&self) -> f32 {
        if self.units.is_empty() {
            return 0.0;
        }
        let routing = self.units.iter().filter(|u| u.is_broken()).count();
        routing as f32 / self.units.len() as f32
    }

    /// Is this formation broken?
    pub fn is_broken(&self) -> bool {
        self.percentage_routing() >= 0.5
    }

    /// Get approximate commander position (center of formation)
    pub fn commander_position(&self) -> Option<BattleHexCoord> {
        if self.units.is_empty() {
            return None;
        }

        let sum_q: i32 = self.units.iter().map(|u| u.position.q).sum();
        let sum_r: i32 = self.units.iter().map(|u| u.position.r).sum();
        let count = self.units.len() as i32;

        Some(BattleHexCoord::new(sum_q / count, sum_r / count))
    }
}

/// An army (collection of formations for a battle)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Army {
    pub id: ArmyId,
    pub commander: EntityId,
    pub formations: Vec<BattleFormation>,
    pub hq_position: BattleHexCoord,
    pub courier_pool: Vec<EntityId>,
}

impl Army {
    pub fn new(id: ArmyId, commander: EntityId) -> Self {
        Self {
            id,
            commander,
            formations: Vec::new(),
            hq_position: BattleHexCoord::default(),
            courier_pool: Vec::new(),
        }
    }

    /// Total strength of the army
    pub fn total_strength(&self) -> usize {
        self.formations.iter().map(|f| f.total_strength()).sum()
    }

    /// Effective strength of the army
    pub fn effective_strength(&self) -> usize {
        self.formations.iter().map(|f| f.effective_strength()).sum()
    }

    /// Percentage of army routing
    pub fn percentage_routing(&self) -> f32 {
        if self.formations.is_empty() {
            return 0.0;
        }

        let total_units: usize = self.formations.iter().map(|f| f.units.len()).sum();
        if total_units == 0 {
            return 0.0;
        }

        let routing_units: usize = self
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .filter(|u| u.is_broken())
            .count();

        routing_units as f32 / total_units as f32
    }

    /// Get a unit by ID
    pub fn get_unit(&self, unit_id: UnitId) -> Option<&BattleUnit> {
        self.formations
            .iter()
            .flat_map(|f| f.units.iter())
            .find(|u| u.id == unit_id)
    }

    /// Get a mutable unit by ID
    pub fn get_unit_mut(&mut self, unit_id: UnitId) -> Option<&mut BattleUnit> {
        self.formations
            .iter_mut()
            .flat_map(|f| f.units.iter_mut())
            .find(|u| u.id == unit_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let element = Element::new(vec![EntityId::new(); 5]);
        assert_eq!(element.entities.len(), 5);
    }

    #[test]
    fn test_unit_strength() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 10]));
        assert_eq!(unit.strength(), 10);
    }

    #[test]
    fn test_formation_total_strength() {
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 20]));
        formation.units.push(unit);
        assert_eq!(formation.total_strength(), 20);
    }

    #[test]
    fn test_army_creation() {
        let army = Army::new(ArmyId::new(), EntityId::new());
        assert!(army.formations.is_empty());
    }

    #[test]
    fn test_unit_effective_strength_with_casualties() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 100]));
        unit.casualties = 30;
        assert_eq!(unit.effective_strength(), 70);
    }

    #[test]
    fn test_unit_broken_when_routing() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        assert!(!unit.is_broken());

        unit.stance = UnitStance::Routing;
        assert!(unit.is_broken());
    }

    #[test]
    fn test_formation_broken_threshold() {
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());

        for _ in 0..10 {
            formation
                .units
                .push(BattleUnit::new(UnitId::new(), UnitType::Infantry));
        }

        // Break 4 units (40%) - not broken
        for i in 0..4 {
            formation.units[i].stance = UnitStance::Routing;
        }
        assert!(!formation.is_broken());

        // Break 5th unit (50%) - now broken
        formation.units[4].stance = UnitStance::Routing;
        assert!(formation.is_broken());
    }
}
