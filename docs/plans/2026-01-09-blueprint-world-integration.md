# Blueprint World Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Integrate the blueprint system into the game world so that natural objects spawn complete, constructed objects track construction progress, and all objects participate in spatial queries/pathfinding.

**Architecture:**
- WorldObjects struct holds Vec<BlueprintInstance> with positions
- SpatialId enum wraps Entity/Object IDs for unified spatial grid
- JSON loader parses Python worldgen output into placed instances
- blocked_cells HashSet tracks impassable positions for pathfinding MVP

**Tech Stack:** Rust, serde_json, glam (Vec2), ahash (AHashMap/AHashSet)

---

## Task 1: Create SpatialId Enum

**Files:**
- Create: `src/world/mod.rs`
- Create: `src/world/spatial_id.rs`
- Modify: `src/lib.rs` (add `pub mod world;`)

**Step 1: Write the failing test**

```rust
// src/world/spatial_id.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::EntityId;
    use crate::blueprints::InstanceId;

    #[test]
    fn test_spatial_id_entity() {
        let entity_id = EntityId::new();
        let spatial = SpatialId::Entity(entity_id);

        assert!(spatial.is_entity());
        assert!(!spatial.is_object());
        assert_eq!(spatial.as_entity(), Some(entity_id));
        assert_eq!(spatial.as_object(), None);
    }

    #[test]
    fn test_spatial_id_object() {
        let instance_id = InstanceId(42);
        let spatial = SpatialId::Object(instance_id);

        assert!(!spatial.is_entity());
        assert!(spatial.is_object());
        assert_eq!(spatial.as_entity(), None);
        assert_eq!(spatial.as_object(), Some(instance_id));
    }

    #[test]
    fn test_spatial_id_hash_eq() {
        use std::collections::HashSet;

        let s1 = SpatialId::Object(InstanceId(1));
        let s2 = SpatialId::Object(InstanceId(1));
        let s3 = SpatialId::Object(InstanceId(2));

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);

        let mut set = HashSet::new();
        set.insert(s1);
        assert!(set.contains(&s2));
        assert!(!set.contains(&s3));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib world::spatial_id -- --nocapture`
Expected: FAIL - module not found

**Step 3: Write minimal implementation**

```rust
// src/world/mod.rs
pub mod spatial_id;

pub use spatial_id::SpatialId;
```

```rust
// src/world/spatial_id.rs
//! Unified spatial ID for entities and world objects

use crate::blueprints::InstanceId;
use crate::core::types::EntityId;

/// Unified ID type for spatial grid storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpatialId {
    /// An entity (human, orc, etc.)
    Entity(EntityId),
    /// A world object (wall, tree, building)
    Object(InstanceId),
}

impl SpatialId {
    /// Check if this is an entity ID
    pub fn is_entity(&self) -> bool {
        matches!(self, Self::Entity(_))
    }

    /// Check if this is an object ID
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Get the entity ID if this is an entity
    pub fn as_entity(&self) -> Option<EntityId> {
        match self {
            Self::Entity(id) => Some(*id),
            Self::Object(_) => None,
        }
    }

    /// Get the instance ID if this is an object
    pub fn as_object(&self) -> Option<InstanceId> {
        match self {
            Self::Object(id) => Some(*id),
            Self::Entity(_) => None,
        }
    }
}

impl From<EntityId> for SpatialId {
    fn from(id: EntityId) -> Self {
        Self::Entity(id)
    }
}

impl From<InstanceId> for SpatialId {
    fn from(id: InstanceId) -> Self {
        Self::Object(id)
    }
}

#[cfg(test)]
mod tests {
    // Tests from above
}
```

Add to `src/lib.rs`:
```rust
pub mod world;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib world::spatial_id -- --nocapture`
Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add src/world/ src/lib.rs
git commit -m "$(cat <<'EOF'
feat(world): add SpatialId enum for unified spatial queries

Wraps EntityId and InstanceId so spatial grid can store both
entities and world objects in a single data structure.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Create WorldObjects Struct

**Files:**
- Create: `src/world/objects.rs`
- Modify: `src/world/mod.rs` (add export)

**Step 1: Write the failing test**

```rust
// src/world/objects.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprints::{BlueprintInstance, InstanceId, PlacedBy};
    use glam::Vec2;

    fn make_test_instance(id: u64, name: &str, pos: Vec2) -> BlueprintInstance {
        BlueprintInstance {
            id: InstanceId(id),
            blueprint_id: crate::blueprints::BlueprintId(1),
            blueprint_name: name.to_string(),
            parameters: std::collections::HashMap::new(),
            position: pos,
            rotation: 0.0,
            geometry: crate::blueprints::EvaluatedGeometry {
                width: 1.0,
                depth: 1.0,
                height: 1.0,
                footprint: vec![],
            },
            current_hp: 100.0,
            max_hp: 100.0,
            damage_state: "intact".to_string(),
            breaches: vec![],
            construction_progress: 1.0,
            construction_stage: "complete".to_string(),
            placed_by: PlacedBy::TerrainGen,
            military: Default::default(),
            civilian: Default::default(),
            anchors: vec![],
            tags: vec![],
        }
    }

    #[test]
    fn test_add_and_get_object() {
        let mut objects = WorldObjects::new();
        let instance = make_test_instance(1, "oak_tree", Vec2::new(10.0, 20.0));

        objects.add(instance.clone());

        let retrieved = objects.get(InstanceId(1));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().blueprint_name, "oak_tree");
    }

    #[test]
    fn test_get_by_position() {
        let mut objects = WorldObjects::new();
        objects.add(make_test_instance(1, "tree", Vec2::new(10.0, 10.0)));
        objects.add(make_test_instance(2, "rock", Vec2::new(50.0, 50.0)));

        let nearby = objects.get_in_radius(Vec2::new(12.0, 12.0), 10.0);
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0].blueprint_name, "tree");
    }

    #[test]
    fn test_remove_object() {
        let mut objects = WorldObjects::new();
        objects.add(make_test_instance(1, "wall", Vec2::new(0.0, 0.0)));

        assert!(objects.get(InstanceId(1)).is_some());
        objects.remove(InstanceId(1));
        assert!(objects.get(InstanceId(1)).is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib world::objects -- --nocapture`
Expected: FAIL - module not found

**Step 3: Write minimal implementation**

```rust
// src/world/objects.rs
//! World objects storage and queries

use crate::blueprints::{BlueprintInstance, InstanceId};
use ahash::AHashMap;
use glam::Vec2;

/// Storage for all world objects (walls, trees, buildings, etc.)
pub struct WorldObjects {
    /// All instances by ID
    instances: AHashMap<InstanceId, BlueprintInstance>,
}

impl WorldObjects {
    pub fn new() -> Self {
        Self {
            instances: AHashMap::new(),
        }
    }

    /// Add a world object
    pub fn add(&mut self, instance: BlueprintInstance) {
        self.instances.insert(instance.id, instance);
    }

    /// Get an object by ID
    pub fn get(&self, id: InstanceId) -> Option<&BlueprintInstance> {
        self.instances.get(&id)
    }

    /// Get mutable reference to an object
    pub fn get_mut(&mut self, id: InstanceId) -> Option<&mut BlueprintInstance> {
        self.instances.get_mut(&id)
    }

    /// Remove an object
    pub fn remove(&mut self, id: InstanceId) -> Option<BlueprintInstance> {
        self.instances.remove(&id)
    }

    /// Get all objects within radius of a point
    pub fn get_in_radius(&self, center: Vec2, radius: f32) -> Vec<&BlueprintInstance> {
        let radius_sq = radius * radius;
        self.instances
            .values()
            .filter(|obj| obj.position.distance_squared(center) <= radius_sq)
            .collect()
    }

    /// Iterate over all objects
    pub fn iter(&self) -> impl Iterator<Item = &BlueprintInstance> {
        self.instances.values()
    }

    /// Number of objects
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

impl Default for WorldObjects {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // Tests from above
}
```

Update `src/world/mod.rs`:
```rust
pub mod spatial_id;
pub mod objects;

pub use spatial_id::SpatialId;
pub use objects::WorldObjects;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib world::objects -- --nocapture`
Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add src/world/objects.rs src/world/mod.rs
git commit -m "$(cat <<'EOF'
feat(world): add WorldObjects struct for object storage

Provides add/get/remove/get_in_radius operations for
BlueprintInstance objects separate from entity archetypes.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Create JSON Placement Schema

**Files:**
- Create: `src/world/placement.rs` (JSON schema types)
- Modify: `src/world/mod.rs` (add export)

**Step 1: Write the failing test**

```rust
// src/world/placement.rs
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_JSON: &str = r#"{
        "version": 1,
        "placements": [
            {
                "id": "castle_001",
                "template": "castle_keep",
                "position": [100.0, 200.0],
                "rotation_deg": 45.0,
                "placed_by": { "HistorySim": { "polity_id": 42, "year": -500 } },
                "state": "complete",
                "damage_state": "damaged",
                "current_hp_ratio": 0.6,
                "parameters": { "length": 30.0, "height": 25.0 },
                "tags": ["fortification", "stone"]
            },
            {
                "id": "tree_001",
                "template": "oak_tree",
                "position": [50.0, 60.0],
                "placed_by": "TerrainGen",
                "parameters": { "height": 15.0 }
            }
        ]
    }"#;

    #[test]
    fn test_deserialize_placement_file() {
        let file: PlacementFile = serde_json::from_str(SAMPLE_JSON).unwrap();

        assert_eq!(file.version, 1);
        assert_eq!(file.placements.len(), 2);
    }

    #[test]
    fn test_placement_with_history_sim() {
        let file: PlacementFile = serde_json::from_str(SAMPLE_JSON).unwrap();
        let castle = &file.placements[0];

        assert_eq!(castle.id, "castle_001");
        assert_eq!(castle.template, "castle_keep");
        assert_eq!(castle.position, [100.0, 200.0]);
        assert_eq!(castle.rotation_deg, Some(45.0));
        assert!(matches!(castle.placed_by, PlacedByJson::HistorySim { .. }));
        assert_eq!(castle.state, Some(ObjectState::Complete));
        assert_eq!(castle.damage_state, Some("damaged".to_string()));
    }

    #[test]
    fn test_placement_natural_defaults() {
        let file: PlacementFile = serde_json::from_str(SAMPLE_JSON).unwrap();
        let tree = &file.placements[1];

        assert_eq!(tree.template, "oak_tree");
        assert!(matches!(tree.placed_by, PlacedByJson::TerrainGen));
        assert!(tree.state.is_none()); // Natural objects don't need state
        assert!(tree.rotation_deg.is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib world::placement -- --nocapture`
Expected: FAIL - module not found

**Step 3: Write minimal implementation**

```rust
// src/world/placement.rs
//! JSON schema for world object placements from worldgen

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root of a placement JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementFile {
    /// Schema version for compatibility checking
    pub version: u32,
    /// Optional metadata
    #[serde(default)]
    pub metadata: Option<PlacementMetadata>,
    /// The actual placements
    pub placements: Vec<Placement>,
}

/// Optional metadata about the placement file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<String>,
}

/// A single object placement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placement {
    /// Unique identifier for this placement (for updates)
    pub id: String,
    /// Blueprint template name (e.g., "castle_keep", "oak_tree")
    pub template: String,
    /// Position [x, y]
    pub position: [f32; 2],
    /// Rotation in degrees (default 0)
    #[serde(default)]
    pub rotation_deg: Option<f32>,
    /// Who placed this object
    pub placed_by: PlacedByJson,
    /// Construction state (only for constructed objects)
    #[serde(default)]
    pub state: Option<ObjectState>,
    /// Damage state name (if damaged)
    #[serde(default)]
    pub damage_state: Option<String>,
    /// Current HP as ratio of max (0.0-1.0)
    #[serde(default)]
    pub current_hp_ratio: Option<f32>,
    /// Construction progress (0.0-1.0) for under_construction state
    #[serde(default)]
    pub construction_progress: Option<f32>,
    /// Parameter values for blueprint instantiation
    #[serde(default)]
    pub parameters: HashMap<String, f32>,
    /// Optional tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Who placed this object (JSON-friendly version of PlacedBy)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlacedByJson {
    /// Simple terrain generation
    TerrainGen,
    /// History simulation with metadata
    HistorySim { polity_id: u32, year: i32 },
    /// Gameplay with tick
    Gameplay { tick: u64 },
}

// Handle "TerrainGen" as a string
impl PlacedByJson {
    pub fn terrain_gen() -> Self {
        Self::TerrainGen
    }
}

// Custom deserializer for PlacedByJson to handle string "TerrainGen"
use serde::de::{self, Visitor};
use std::fmt;

struct PlacedByJsonVisitor;

impl<'de> Visitor<'de> for PlacedByJsonVisitor {
    type Value = PlacedByJson;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("'TerrainGen' or an object with HistorySim/Gameplay")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "TerrainGen" => Ok(PlacedByJson::TerrainGen),
            _ => Err(de::Error::unknown_variant(value, &["TerrainGen"])),
        }
    }

    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        // Use default map handling
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper {
            HistorySim {
                #[serde(rename = "HistorySim")]
                inner: HistorySimInner,
            },
            Gameplay {
                #[serde(rename = "Gameplay")]
                inner: GameplayInner,
            },
        }

        #[derive(Deserialize)]
        struct HistorySimInner {
            polity_id: u32,
            year: i32,
        }

        #[derive(Deserialize)]
        struct GameplayInner {
            tick: u64,
        }

        let helper = Helper::deserialize(de::value::MapAccessDeserializer::new(map))?;
        match helper {
            Helper::HistorySim { inner } => Ok(PlacedByJson::HistorySim {
                polity_id: inner.polity_id,
                year: inner.year,
            }),
            Helper::Gameplay { inner } => Ok(PlacedByJson::Gameplay { tick: inner.tick }),
        }
    }
}

impl<'de> Deserialize<'de> for PlacedByJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(PlacedByJsonVisitor)
    }
}

/// Construction state for placed objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectState {
    /// Fully constructed
    Complete,
    /// Under construction
    UnderConstruction,
}

#[cfg(test)]
mod tests {
    // Tests from above
}
```

Update `src/world/mod.rs`:
```rust
pub mod spatial_id;
pub mod objects;
pub mod placement;

pub use spatial_id::SpatialId;
pub use objects::WorldObjects;
pub use placement::{PlacementFile, Placement, PlacedByJson, ObjectState};
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib world::placement -- --nocapture`
Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add src/world/placement.rs src/world/mod.rs
git commit -m "$(cat <<'EOF'
feat(world): add JSON placement schema for worldgen output

Defines PlacementFile, Placement, PlacedByJson types for
deserializing world object placements from Python worldgen.

Supports TerrainGen (string), HistorySim, Gameplay placed_by variants.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Create Placement Loader

**Files:**
- Create: `src/world/loader.rs`
- Modify: `src/world/mod.rs` (add export)

**Step 1: Write the failing test**

```rust
// src/world/loader.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprints::BlueprintRegistry;

    fn setup_registry() -> BlueprintRegistry {
        let mut registry = BlueprintRegistry::new();
        // Load test blueprints
        registry.load_directory("data/blueprints").ok();
        registry
    }

    #[test]
    fn test_load_placement_file() {
        let registry = setup_registry();
        let json = r#"{
            "version": 1,
            "placements": [
                {
                    "id": "tree_001",
                    "template": "oak_tree",
                    "position": [50.0, 60.0],
                    "placed_by": "TerrainGen",
                    "parameters": { "height": 15.0, "canopy_radius": 6.0, "trunk_radius": 0.5 }
                }
            ]
        }"#;

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json).unwrap();

        assert_eq!(objects.len(), 1);
        let obj = objects.get(InstanceId(1)).unwrap();
        assert_eq!(obj.blueprint_name, "oak_tree");
        assert_eq!(obj.construction_progress, 1.0); // Natural = complete
    }

    #[test]
    fn test_load_constructed_complete() {
        let registry = setup_registry();
        let json = r#"{
            "version": 1,
            "placements": [
                {
                    "id": "wall_001",
                    "template": "stone_wall",
                    "position": [100.0, 100.0],
                    "placed_by": { "HistorySim": { "polity_id": 1, "year": -100 } },
                    "state": "complete",
                    "parameters": { "length": 10.0, "height": 3.0, "thickness": 0.6 }
                }
            ]
        }"#;

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json).unwrap();

        let wall = objects.get(InstanceId(1)).unwrap();
        assert_eq!(wall.construction_progress, 1.0);
        assert_eq!(wall.construction_stage, "complete");
    }

    #[test]
    fn test_load_under_construction() {
        let registry = setup_registry();
        let json = r#"{
            "version": 1,
            "placements": [
                {
                    "id": "wall_002",
                    "template": "stone_wall",
                    "position": [200.0, 100.0],
                    "placed_by": { "Gameplay": { "tick": 500 } },
                    "state": "under_construction",
                    "construction_progress": 0.3,
                    "parameters": { "length": 8.0, "height": 2.5, "thickness": 0.5 }
                }
            ]
        }"#;

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json).unwrap();

        let wall = objects.get(InstanceId(1)).unwrap();
        assert!((wall.construction_progress - 0.3).abs() < 0.001);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib world::loader -- --nocapture`
Expected: FAIL - module not found

**Step 3: Write minimal implementation**

```rust
// src/world/loader.rs
//! Load world objects from JSON placement files

use crate::blueprints::{BlueprintInstance, BlueprintRegistry, InstanceId, PlacedBy};
use crate::world::objects::WorldObjects;
use crate::world::placement::{ObjectState, PlacedByJson, PlacementFile};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

/// Errors during placement loading
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Blueprint not found: {0}")]
    BlueprintNotFound(String),
    #[error("Blueprint instantiation failed: {0}")]
    InstantiationFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Loads placements and instantiates blueprints
pub struct PlacementLoader<'a> {
    registry: &'a BlueprintRegistry,
    next_id: AtomicU64,
}

impl<'a> PlacementLoader<'a> {
    pub fn new(registry: &'a BlueprintRegistry) -> Self {
        Self {
            registry,
            next_id: AtomicU64::new(1),
        }
    }

    /// Load placements from JSON string
    pub fn load_from_json(&self, json: &str) -> Result<WorldObjects, LoadError> {
        let file: PlacementFile = serde_json::from_str(json)?;
        self.load_from_placements(&file)
    }

    /// Load placements from file path
    pub fn load_from_file(&self, path: &str) -> Result<WorldObjects, LoadError> {
        let content = std::fs::read_to_string(path)?;
        self.load_from_json(&content)
    }

    /// Load from parsed placement file
    fn load_from_placements(&self, file: &PlacementFile) -> Result<WorldObjects, LoadError> {
        let mut objects = WorldObjects::new();

        for placement in &file.placements {
            // Get blueprint ID
            let blueprint_id = self
                .registry
                .id_by_name(&placement.template)
                .ok_or_else(|| LoadError::BlueprintNotFound(placement.template.clone()))?;

            // Instantiate with parameters
            let instance_id = InstanceId(self.next_id.fetch_add(1, Ordering::SeqCst));
            let placed_by = convert_placed_by(&placement.placed_by);

            let mut instance = self
                .registry
                .instantiate(blueprint_id, &placement.parameters, placed_by)
                .map_err(|e| LoadError::InstantiationFailed(e.to_string()))?;

            // Override instance ID (registry generates its own)
            instance.id = instance_id;

            // Set position and rotation
            instance.position = glam::Vec2::new(placement.position[0], placement.position[1]);
            instance.rotation = placement.rotation_deg.unwrap_or(0.0).to_radians();

            // Handle construction state
            match placement.state {
                Some(ObjectState::Complete) => {
                    instance.construction_progress = 1.0;
                    instance.construction_stage = "complete".to_string();
                }
                Some(ObjectState::UnderConstruction) => {
                    instance.construction_progress = placement.construction_progress.unwrap_or(0.0);
                    // Find appropriate stage for progress
                    // (actual stage lookup would need blueprint data)
                }
                None => {
                    // Natural objects are always complete
                    instance.construction_progress = 1.0;
                    instance.construction_stage = "complete".to_string();
                }
            }

            // Handle damage state
            if let Some(ref damage_state) = placement.damage_state {
                instance.damage_state = damage_state.clone();
            }
            if let Some(hp_ratio) = placement.current_hp_ratio {
                instance.current_hp = instance.max_hp * hp_ratio;
            }

            // Add tags
            instance.tags = placement.tags.clone();

            objects.add(instance);
        }

        Ok(objects)
    }
}

/// Convert JSON placed_by to internal PlacedBy
fn convert_placed_by(json: &PlacedByJson) -> PlacedBy {
    match json {
        PlacedByJson::TerrainGen => PlacedBy::TerrainGen,
        PlacedByJson::HistorySim { polity_id, year } => PlacedBy::HistorySim {
            polity_id: *polity_id,
            year: *year,
        },
        PlacedByJson::Gameplay { tick } => PlacedBy::Gameplay { tick: *tick },
    }
}

#[cfg(test)]
mod tests {
    // Tests from above
}
```

Update `src/world/mod.rs`:
```rust
pub mod spatial_id;
pub mod objects;
pub mod placement;
pub mod loader;

pub use spatial_id::SpatialId;
pub use objects::WorldObjects;
pub use placement::{PlacementFile, Placement, PlacedByJson, ObjectState};
pub use loader::{PlacementLoader, LoadError};
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib world::loader -- --nocapture`
Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add src/world/loader.rs src/world/mod.rs
git commit -m "$(cat <<'EOF'
feat(world): add PlacementLoader for JSON → WorldObjects

Loads placement JSON files and instantiates blueprints with
correct PlacedBy provenance, construction state, and damage.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Add BlockingState and Blocked Cells Tracking

**Files:**
- Create: `src/world/blocking.rs`
- Modify: `src/world/mod.rs` (add export)

**Step 1: Write the failing test**

```rust
// src/world/blocking.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_cells_insert_remove() {
        let mut blocked = BlockedCells::new();

        blocked.block(5, 10);
        blocked.block(5, 11);

        assert!(blocked.is_blocked(5, 10));
        assert!(blocked.is_blocked(5, 11));
        assert!(!blocked.is_blocked(5, 12));

        blocked.unblock(5, 10);
        assert!(!blocked.is_blocked(5, 10));
    }

    #[test]
    fn test_blocked_cells_from_footprint() {
        let mut blocked = BlockedCells::new();

        // Rectangular footprint from (0,0) to (3,2)
        let footprint = vec![
            glam::Vec2::new(0.0, 0.0),
            glam::Vec2::new(3.0, 0.0),
            glam::Vec2::new(3.0, 2.0),
            glam::Vec2::new(0.0, 2.0),
        ];

        blocked.block_footprint(&footprint, 1.0);

        // Should block cells (0,0), (1,0), (2,0), (0,1), (1,1), (2,1)
        assert!(blocked.is_blocked(0, 0));
        assert!(blocked.is_blocked(1, 0));
        assert!(blocked.is_blocked(2, 0));
        assert!(blocked.is_blocked(0, 1));
        assert!(blocked.is_blocked(1, 1));
        assert!(blocked.is_blocked(2, 1));
        assert!(!blocked.is_blocked(3, 0)); // Outside
    }

    #[test]
    fn test_blocking_state_solid() {
        let state = BlockingState::Solid;
        assert!(!state.can_pass());
    }

    #[test]
    fn test_blocking_state_breached() {
        let state = BlockingState::Breached {
            gaps: vec![glam::Vec2::new(5.0, 5.0)],
        };
        assert!(state.can_pass()); // Has gaps
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib world::blocking -- --nocapture`
Expected: FAIL - module not found

**Step 3: Write minimal implementation**

```rust
// src/world/blocking.rs
//! Blocking state and pathfinding obstacle tracking

use ahash::AHashSet;
use glam::Vec2;

/// How an object blocks movement
#[derive(Debug, Clone)]
pub enum BlockingState {
    /// Completely blocks movement
    Solid,
    /// Has passable gaps at specific positions
    Breached { gaps: Vec<Vec2> },
    /// Slows movement but doesn't block (cost multiplier)
    Permeable(f32),
    /// Doesn't block at all
    None,
}

impl BlockingState {
    /// Check if ANY passage is possible
    pub fn can_pass(&self) -> bool {
        match self {
            Self::Solid => false,
            Self::Breached { gaps } => !gaps.is_empty(),
            Self::Permeable(_) => true,
            Self::None => true,
        }
    }

    /// Get movement cost multiplier (1.0 = normal)
    pub fn movement_cost(&self) -> f32 {
        match self {
            Self::Solid => f32::INFINITY,
            Self::Breached { .. } => 1.5, // Slightly slower through breach
            Self::Permeable(cost) => *cost,
            Self::None => 1.0,
        }
    }
}

impl Default for BlockingState {
    fn default() -> Self {
        Self::None
    }
}

/// Simple blocked cells tracking for pathfinding MVP
pub struct BlockedCells {
    cells: AHashSet<(i32, i32)>,
    cell_size: f32,
}

impl BlockedCells {
    pub fn new() -> Self {
        Self::with_cell_size(1.0)
    }

    pub fn with_cell_size(cell_size: f32) -> Self {
        Self {
            cells: AHashSet::new(),
            cell_size,
        }
    }

    /// Block a cell
    pub fn block(&mut self, x: i32, y: i32) {
        self.cells.insert((x, y));
    }

    /// Unblock a cell
    pub fn unblock(&mut self, x: i32, y: i32) {
        self.cells.remove(&(x, y));
    }

    /// Check if a cell is blocked
    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.cells.contains(&(x, y))
    }

    /// Check if a world position is blocked
    pub fn is_position_blocked(&self, pos: Vec2) -> bool {
        let (x, y) = self.world_to_cell(pos);
        self.is_blocked(x, y)
    }

    /// Convert world position to cell coordinates
    pub fn world_to_cell(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    /// Block all cells covered by a polygon footprint
    pub fn block_footprint(&mut self, footprint: &[Vec2], _cell_size: f32) {
        if footprint.is_empty() {
            return;
        }

        // Find bounding box
        let min_x = footprint.iter().map(|v| v.x).fold(f32::INFINITY, f32::min);
        let max_x = footprint.iter().map(|v| v.x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = footprint.iter().map(|v| v.y).fold(f32::INFINITY, f32::min);
        let max_y = footprint.iter().map(|v| v.y).fold(f32::NEG_INFINITY, f32::max);

        // Iterate over cells in bounding box
        let start_x = (min_x / self.cell_size).floor() as i32;
        let end_x = (max_x / self.cell_size).ceil() as i32;
        let start_y = (min_y / self.cell_size).floor() as i32;
        let end_y = (max_y / self.cell_size).ceil() as i32;

        for cx in start_x..end_x {
            for cy in start_y..end_y {
                // Cell center
                let center = Vec2::new(
                    (cx as f32 + 0.5) * self.cell_size,
                    (cy as f32 + 0.5) * self.cell_size,
                );

                // Simple point-in-polygon test
                if point_in_polygon(center, footprint) {
                    self.block(cx, cy);
                }
            }
        }
    }

    /// Clear all blocked cells
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Number of blocked cells
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Check if no cells are blocked
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl Default for BlockedCells {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple point-in-polygon test using ray casting
fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut inside = false;
    let n = polygon.len();

    for i in 0..n {
        let j = (i + 1) % n;
        let vi = polygon[i];
        let vj = polygon[j];

        if ((vi.y > point.y) != (vj.y > point.y))
            && (point.x < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y) + vi.x)
        {
            inside = !inside;
        }
    }

    inside
}

#[cfg(test)]
mod tests {
    // Tests from above
}
```

Update `src/world/mod.rs`:
```rust
pub mod spatial_id;
pub mod objects;
pub mod placement;
pub mod loader;
pub mod blocking;

pub use spatial_id::SpatialId;
pub use objects::WorldObjects;
pub use placement::{PlacementFile, Placement, PlacedByJson, ObjectState};
pub use loader::{PlacementLoader, LoadError};
pub use blocking::{BlockedCells, BlockingState};
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib world::blocking -- --nocapture`
Expected: PASS (4 tests)

**Step 5: Commit**

```bash
git add src/world/blocking.rs src/world/mod.rs
git commit -m "$(cat <<'EOF'
feat(world): add BlockedCells for pathfinding MVP

Simple HashSet-based cell blocking with footprint support.
BlockingState enum for future breach/permeable handling.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Integrate WorldObjects into World Struct

**Files:**
- Modify: `src/ecs/world.rs`

**Step 1: Write the failing test**

```rust
// Add to src/ecs/world.rs tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_has_objects() {
        let world = World::new();
        assert!(world.world_objects.is_empty());
    }

    #[test]
    fn test_world_has_blocked_cells() {
        let world = World::new();
        assert!(world.blocked_cells.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib ecs::world::tests -- --nocapture`
Expected: FAIL - world_objects not found

**Step 3: Modify World struct**

Add imports at top of `src/ecs/world.rs`:
```rust
use crate::world::{WorldObjects, BlockedCells};
```

Add fields to World struct (around line 63):
```rust
pub struct World {
    pub current_tick: u64,
    entity_registry: AHashMap<EntityId, (Species, usize)>,
    pub humans: HumanArchetype,
    pub orcs: OrcArchetype,
    next_indices: AHashMap<Species, usize>,
    pub food_zones: Vec<FoodZone>,
    next_food_zone_id: u32,
    pub resource_zones: Vec<ResourceZone>,
    pub astronomy: AstronomicalState,
    pub species_rules: SpeciesRules,
    pub buildings: BuildingArchetype,
    pub stockpile: Stockpile,
    /// World objects (walls, trees, etc.)
    pub world_objects: WorldObjects,
    /// Blocked cells for pathfinding
    pub blocked_cells: BlockedCells,
}
```

Update `World::new()` (around line 96):
```rust
Self {
    current_tick: 0,
    entity_registry: AHashMap::new(),
    humans: HumanArchetype::new(),
    orcs: OrcArchetype::new(),
    next_indices,
    food_zones: Vec::new(),
    next_food_zone_id: 0,
    resource_zones: Vec::new(),
    astronomy: AstronomicalState::default(),
    species_rules,
    buildings: BuildingArchetype::new(),
    stockpile: Stockpile::new(),
    world_objects: WorldObjects::new(),
    blocked_cells: BlockedCells::new(),
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib ecs::world -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ecs/world.rs
git commit -m "$(cat <<'EOF'
feat(ecs): add world_objects and blocked_cells to World

Integrates WorldObjects and BlockedCells into the main
World struct for unified game state management.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Add World Object Loading Method

**Files:**
- Modify: `src/ecs/world.rs`

**Step 1: Write the failing test**

```rust
// Add to src/ecs/world.rs tests
#[test]
fn test_load_world_objects() {
    let mut world = World::new();

    // Create a test JSON (assuming blueprints are loaded)
    let json = r#"{
        "version": 1,
        "placements": []
    }"#;

    // This should not panic even with empty placements
    let result = world.load_world_objects_json(json);
    assert!(result.is_ok());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib ecs::world::tests::test_load_world_objects -- --nocapture`
Expected: FAIL - method not found

**Step 3: Add method to World**

Add to `src/ecs/world.rs`:
```rust
use crate::world::{WorldObjects, BlockedCells, PlacementLoader, LoadError};
use crate::blueprints::BlueprintRegistry;

impl World {
    /// Load world objects from JSON placement data
    pub fn load_world_objects_json(&mut self, json: &str) -> Result<usize, LoadError> {
        // Create a temporary registry for loading
        let mut registry = BlueprintRegistry::new();
        if let Err(e) = registry.load_directory("data/blueprints") {
            eprintln!("Warning: Failed to load blueprints: {}", e);
        }

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json)?;

        let count = objects.len();

        // Update blocked cells for objects that block movement
        for obj in objects.iter() {
            if obj.military.blocks_movement && !obj.geometry.footprint.is_empty() {
                // Transform footprint to world space
                let world_footprint: Vec<glam::Vec2> = obj.geometry.footprint
                    .iter()
                    .map(|v| {
                        // Rotate and translate
                        let rotated = glam::Vec2::new(
                            v.x * obj.rotation.cos() - v.y * obj.rotation.sin(),
                            v.x * obj.rotation.sin() + v.y * obj.rotation.cos(),
                        );
                        rotated + obj.position
                    })
                    .collect();

                self.blocked_cells.block_footprint(&world_footprint, 1.0);
            }
        }

        self.world_objects = objects;
        Ok(count)
    }

    /// Load world objects from a file path
    pub fn load_world_objects_file(&mut self, path: &str) -> Result<usize, LoadError> {
        let content = std::fs::read_to_string(path)
            .map_err(LoadError::IoError)?;
        self.load_world_objects_json(&content)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib ecs::world::tests::test_load_world_objects -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ecs/world.rs
git commit -m "$(cat <<'EOF'
feat(ecs): add world object loading methods to World

load_world_objects_json() and load_world_objects_file() parse
placement data and populate world_objects and blocked_cells.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Create Integration Test

**Files:**
- Create: `tests/world_objects_integration.rs`

**Step 1: Write the test**

```rust
// tests/world_objects_integration.rs
//! Integration tests for world object loading and spatial queries

use arc_citadel::blueprints::BlueprintRegistry;
use arc_citadel::world::{PlacementLoader, WorldObjects, BlockedCells};
use glam::Vec2;

fn setup_registry() -> BlueprintRegistry {
    let mut registry = BlueprintRegistry::new();
    registry.load_directory("data/blueprints").expect("Failed to load blueprints");
    registry
}

#[test]
fn test_full_world_object_lifecycle() {
    let registry = setup_registry();

    // 1. Create placement JSON (simulating worldgen output)
    let placement_json = r#"{
        "version": 1,
        "metadata": {
            "name": "Test Region",
            "created_by": "Integration Test"
        },
        "placements": [
            {
                "id": "tree_001",
                "template": "oak_tree",
                "position": [10.0, 10.0],
                "placed_by": "TerrainGen",
                "parameters": {
                    "height": 12.0,
                    "canopy_radius": 5.0,
                    "trunk_radius": 0.4
                }
            },
            {
                "id": "wall_001",
                "template": "stone_wall",
                "position": [50.0, 50.0],
                "rotation_deg": 90.0,
                "placed_by": { "HistorySim": { "polity_id": 1, "year": -200 } },
                "state": "complete",
                "damage_state": "damaged",
                "current_hp_ratio": 0.7,
                "parameters": {
                    "length": 15.0,
                    "height": 4.0,
                    "thickness": 0.8
                },
                "tags": ["fortification", "border"]
            },
            {
                "id": "wall_002",
                "template": "stone_wall",
                "position": [100.0, 50.0],
                "placed_by": { "Gameplay": { "tick": 1000 } },
                "state": "under_construction",
                "construction_progress": 0.4,
                "parameters": {
                    "length": 10.0,
                    "height": 3.0,
                    "thickness": 0.6
                }
            }
        ]
    }"#;

    // 2. Load placements
    let loader = PlacementLoader::new(&registry);
    let objects = loader.load_from_json(placement_json).expect("Failed to load placements");

    // 3. Verify object count
    assert_eq!(objects.len(), 3);

    // 4. Verify natural object (tree)
    let tree = objects.iter().find(|o| o.blueprint_name == "oak_tree").unwrap();
    assert_eq!(tree.construction_progress, 1.0); // Natural = complete
    assert!(tree.is_complete());

    // 5. Verify complete constructed object (old wall)
    let old_wall = objects.iter()
        .find(|o| o.blueprint_name == "stone_wall" && o.construction_progress == 1.0)
        .unwrap();
    assert_eq!(old_wall.damage_state, "damaged");
    assert!((old_wall.hp_ratio() - 0.7).abs() < 0.01);
    assert!(old_wall.tags.contains(&"fortification".to_string()));

    // 6. Verify under-construction object
    let new_wall = objects.iter()
        .find(|o| o.blueprint_name == "stone_wall" && o.construction_progress < 1.0)
        .unwrap();
    assert!((new_wall.construction_progress - 0.4).abs() < 0.01);
    assert!(!new_wall.is_complete());

    // 7. Test spatial query
    let nearby_tree = objects.get_in_radius(Vec2::new(12.0, 12.0), 5.0);
    assert_eq!(nearby_tree.len(), 1);
    assert_eq!(nearby_tree[0].blueprint_name, "oak_tree");

    let nearby_walls = objects.get_in_radius(Vec2::new(75.0, 50.0), 30.0);
    assert_eq!(nearby_walls.len(), 2); // Both walls
}

#[test]
fn test_blocked_cells_from_objects() {
    let registry = setup_registry();

    let placement_json = r#"{
        "version": 1,
        "placements": [
            {
                "id": "wall_blocking",
                "template": "stone_wall",
                "position": [0.0, 0.0],
                "placed_by": "TerrainGen",
                "state": "complete",
                "parameters": {
                    "length": 5.0,
                    "height": 3.0,
                    "thickness": 1.0
                }
            }
        ]
    }"#;

    let loader = PlacementLoader::new(&registry);
    let objects = loader.load_from_json(placement_json).unwrap();

    // Build blocked cells from objects
    let mut blocked = BlockedCells::new();
    for obj in objects.iter() {
        if obj.military.blocks_movement && !obj.geometry.footprint.is_empty() {
            blocked.block_footprint(&obj.geometry.footprint, 1.0);
        }
    }

    // Wall should block some cells
    // (exact cells depend on footprint generation)
    assert!(blocked.len() > 0 || objects.iter().next().unwrap().geometry.footprint.is_empty());
}

#[test]
fn test_provenance_tracking() {
    let registry = setup_registry();

    let placement_json = r#"{
        "version": 1,
        "placements": [
            {
                "id": "terrain_rock",
                "template": "rock_outcrop",
                "position": [0.0, 0.0],
                "placed_by": "TerrainGen",
                "parameters": { "width": 3.0, "depth": 2.0, "height": 1.5 }
            },
            {
                "id": "history_wall",
                "template": "stone_wall",
                "position": [10.0, 0.0],
                "placed_by": { "HistorySim": { "polity_id": 42, "year": -500 } },
                "state": "complete",
                "parameters": { "length": 5.0, "height": 2.0, "thickness": 0.5 }
            },
            {
                "id": "player_wall",
                "template": "stone_wall",
                "position": [20.0, 0.0],
                "placed_by": { "Gameplay": { "tick": 12345 } },
                "state": "complete",
                "parameters": { "length": 5.0, "height": 2.0, "thickness": 0.5 }
            }
        ]
    }"#;

    let loader = PlacementLoader::new(&registry);
    let objects = loader.load_from_json(placement_json).unwrap();

    use arc_citadel::blueprints::PlacedBy;

    // Verify each provenance type
    for obj in objects.iter() {
        match &obj.placed_by {
            PlacedBy::TerrainGen => {
                assert_eq!(obj.blueprint_name, "rock_outcrop");
            }
            PlacedBy::HistorySim { polity_id, year } => {
                assert_eq!(*polity_id, 42);
                assert_eq!(*year, -500);
            }
            PlacedBy::Gameplay { tick } => {
                assert_eq!(*tick, 12345);
            }
        }
    }
}
```

**Step 2: Run test**

Run: `cargo test --test world_objects_integration -- --nocapture`
Expected: PASS (3 tests)

**Step 3: Commit**

```bash
git add tests/world_objects_integration.rs
git commit -m "$(cat <<'EOF'
test: add world objects integration tests

Tests full lifecycle: JSON loading, natural vs constructed,
spatial queries, blocked cells, and provenance tracking.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Final Verification

**Step 1: Run all tests**

```bash
cargo test
```

Expected: All tests pass

**Step 2: Run clippy**

```bash
cargo clippy --all-targets -- -D warnings
```

Expected: No errors

**Step 3: Build release**

```bash
cargo build --release
```

Expected: Successful build

**Step 4: Verify module exports**

```bash
cargo doc --no-deps --open
```

Expected: Documentation for `arc_citadel::world` module visible

**Step 5: Final commit (if any cleanup needed)**

```bash
git add -A
git commit -m "$(cat <<'EOF'
chore: final cleanup for blueprint world integration

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Summary

| Task | Files | Tests |
|------|-------|-------|
| 1. SpatialId enum | `src/world/mod.rs`, `src/world/spatial_id.rs` | 3 |
| 2. WorldObjects struct | `src/world/objects.rs` | 3 |
| 3. JSON placement schema | `src/world/placement.rs` | 3 |
| 4. PlacementLoader | `src/world/loader.rs` | 3 |
| 5. BlockedCells | `src/world/blocking.rs` | 4 |
| 6. World integration | `src/ecs/world.rs` | 2 |
| 7. World loading methods | `src/ecs/world.rs` | 1 |
| 8. Integration test | `tests/world_objects_integration.rs` | 3 |
| 9. Final verification | - | - |

**Total: ~22 unit tests + 3 integration tests**
