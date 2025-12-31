# Data Module

> Asset loading and game data definitions.

## Module Structure

```
data/
└── mod.rs    # Module exports (stub)
```

## Data Files

```
data/
└── species/
    └── human.json    # Human species definition
```

## Current Data: Human Species

```json
{
  "species": "Human",
  "value_vocabulary": [
    {"name": "honor", "description": "Social standing, keeping word"},
    {"name": "beauty", "description": "Aesthetic appreciation"},
    {"name": "comfort", "description": "Physical ease"},
    {"name": "ambition", "description": "Desire for advancement"},
    {"name": "loyalty", "description": "Attachment to group"},
    {"name": "love", "description": "Attachment to individuals"},
    {"name": "justice", "description": "Fairness"},
    {"name": "curiosity", "description": "Desire to know"},
    {"name": "safety", "description": "Self-preservation"},
    {"name": "piety", "description": "Religious devotion"}
  ],
  "idle_behaviors": ["socialize", "observe", "wander"],
  "perception_filters": {
    "high_beauty": ["aesthetic", "quality", "appearance"],
    "high_honor": ["social_status", "reputation"],
    "high_piety": ["sacred", "ritual"]
  }
}
```

## Planned Design

### Asset Loader

```rust
pub struct AssetLoader {
    base_path: PathBuf,
}

impl AssetLoader {
    pub fn new(base_path: impl Into<PathBuf>) -> Self;

    pub fn load_species(&self, name: &str) -> Result<SpeciesDefinition>;
    pub fn load_items(&self) -> Result<ItemDatabase>;
    pub fn load_names(&self) -> Result<NameGenerator>;
}
```

### Species Definition

```rust
#[derive(Deserialize)]
pub struct SpeciesDefinition {
    pub species: String,
    pub value_vocabulary: Vec<ValueDefinition>,
    pub idle_behaviors: Vec<String>,
    pub perception_filters: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
pub struct ValueDefinition {
    pub name: String,
    pub description: String,
}
```

### Item Database

```rust
pub struct ItemDatabase {
    pub weapons: HashMap<String, WeaponDef>,
    pub armor: HashMap<String, ArmorDef>,
    pub tools: HashMap<String, ToolDef>,
    pub resources: HashMap<String, ResourceDef>,
}
```

### Name Generator

```rust
pub struct NameGenerator {
    pub first_names: Vec<String>,
    pub surnames: Vec<String>,
    pub nicknames: Vec<String>,
}

impl NameGenerator {
    pub fn generate(&self, rng: &mut impl Rng) -> String {
        let first = self.first_names.choose(rng).unwrap();
        let surname = self.surnames.choose(rng).unwrap();
        format!("{} {}", first, surname)
    }
}
```

## Data File Formats

| Data Type | Format | Location |
|-----------|--------|----------|
| Species | JSON | `data/species/*.json` |
| Items | JSON | `data/items/*.json` |
| Names | JSON | `data/names/*.json` |
| Maps | Custom | `data/maps/*.map` |

## Integration Points

### With `entity/species/`
- Load species definitions at startup
- Apply value vocabularies to archetypes

### With `combat/`
- Load weapon and armor definitions

### With `campaign/`
- Load map data

## Future Implementation

1. **Implement AssetLoader** with JSON parsing
2. **Load species** definitions at startup
3. **Add item database** for equipment
4. **Add name generator** for entity spawning
5. **Add map loading** for campaign
