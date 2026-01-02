# Species Definition Schema

## Required Fields

```toml
[metadata]
name = "Human"              # PascalCase, used for struct names
module_name = "human"       # snake_case, used for file and module names

[entity_values]
# Arbitrary f32 fields - species-specific value concepts
honor = 0.5                 # Default value (0.0-1.0)
beauty = 0.5
# ... add as many as needed

[polity_state]
# Fields for aggregate-level state
expansion_pressure = { type = "f32", default = 0.5 }
internal_cohesion = { type = "f32", default = 0.7 }
factions = { type = "Vec<Faction>", default = "Vec::new()" }
# Supported types: f32, u32, String, Vec<T>, HashMap<K,V>, HashSet<T>

[terrain_fitness]
# Terrain type -> fitness value (0.0-1.0)
plains = 0.9
forest = 0.7
mountains = 0.3
hills = 0.7
desert = 0.3
tundra = 0.2
swamp = 0.3
coastal = 0.8

[growth]
rate = 1.01                 # Population multiplier per year (e.g., 1.01 = 1% growth)

[expansion]
threshold = 0.3             # Minimum terrain fitness for expansion

[naming]
prefixes = ["Alden", "Bran", "Cael"]
suffixes = ["mark", "ford", "heim"]

[polity_types]
# Map territory size ranges to polity types
small = "Tribe"             # 0-5 regions
medium = "CityState"        # 6-15 regions
large = "Kingdom"           # 16+ regions
```

## Validation Rules

1. `name` must be PascalCase and a valid Rust identifier
2. `module_name` must be snake_case and a valid Rust identifier
3. All `entity_values` must be f32 between 0.0 and 1.0
4. `polity_state` fields must have valid `type` strings
5. `terrain_fitness` values must be f32 between 0.0 and 1.0
6. `growth.rate` should typically be between 1.0 and 1.1
7. `polity_types` values must match existing `PolityType` enum variants
