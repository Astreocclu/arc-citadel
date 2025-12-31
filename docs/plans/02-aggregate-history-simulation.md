# Aggregate History Simulation - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a world-generation aggregate simulation that produces 250 years of polity-level history in under 60 seconds. Species (Human, Dwarf, Elf) must feel authentically different through emergent behavioral rules.

**Architecture:** Separate module from entity-level gameplay. Pseudo-node graph (1200 regions) abstracts over hex detail. Priority-sorted event queue. Species-specific state enums with static dispatch. ChaCha8Rng for deterministic replay.

**Priority:** Species authenticity > feature completeness > performance tuning

---

## Critical Design Constraints

1. **PSEUDO-NODES NOT HEXES** - Regions are abstract strategic areas (~100+ hexes each when expanded to gameplay)
2. **SPECIES AUTHENTICITY** - Dwarves fight grudge wars they can't win. Elves deliberate for decades. Humans expand and fragment.
3. **STATIC DISPATCH** - Use enum variants for SpeciesState, not trait objects
4. **DETERMINISTIC** - Same seed produces identical history

---

## Task 1: Module Scaffolding

**Files:**
- Edit: `src/lib.rs` (add aggregate module)
- Create: `src/aggregate/mod.rs`
- Create: `src/aggregate/world.rs`
- Create: `src/aggregate/region.rs`
- Create: `src/aggregate/polity.rs`
- Create: `src/aggregate/simulation.rs`
- Create: `src/aggregate/events.rs`
- Create: `src/aggregate/output.rs`
- Create: `src/aggregate/systems/mod.rs`
- Create: `src/aggregate/species/mod.rs`

**Step 1: Create directory structure**

```bash
cd /home/astre/arc-citadel
mkdir -p src/aggregate/systems
mkdir -p src/aggregate/species
```

**Step 2: Add aggregate module to lib.rs**

Edit `src/lib.rs` - add after line 14 (`pub mod ui;`):
```rust
pub mod aggregate;
```

**Step 3: Create src/aggregate/mod.rs**

```rust
//! Aggregate History Simulation
//!
//! World-generation module that simulates polity-level history.
//! Operates on ~1200 pseudo-node regions, not individual hexes.
//! Produces emergent history with species-authentic behavior.

pub mod world;
pub mod region;
pub mod polity;
pub mod simulation;
pub mod events;
pub mod output;
pub mod systems;
pub mod species;

pub use world::AggregateWorld;
pub use region::{Region, Terrain, ResourceType};
pub use polity::{Polity, PolityType, SpeciesState};
pub use simulation::{simulate, SimulationConfig};
pub use events::{Event, EventType, HistoryLog};
pub use output::SimulationOutput;
```

**Step 4: Create placeholder files**

Create `src/aggregate/world.rs`:
```rust
//! AggregateWorld - the main world state container

use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

use crate::aggregate::region::Region;
use crate::aggregate::polity::Polity;
use crate::aggregate::events::Event;

/// The aggregate world state for history simulation
pub struct AggregateWorld {
    /// All regions (pseudo-nodes) in the world
    pub regions: Vec<Region>,
    /// All polities (nations/tribes/holds/groves)
    pub polities: Vec<Polity>,
    /// Currently active wars
    pub active_wars: Vec<War>,
    /// Current simulation year
    pub year: u32,
    /// Random number generator (deterministic)
    pub rng: ChaCha8Rng,
}

/// Active war state machine
pub struct War {
    pub id: u32,
    pub aggressor: u32,
    pub defender: u32,
    pub cause: WarCause,
    pub start_year: u32,
    pub state: WarState,
    pub contested_regions: Vec<u32>,
}

#[derive(Clone, Debug)]
pub enum WarCause {
    Expansion,
    Grudge(Vec<u32>), // Grudge IDs
    Honor,
    Religion,
    Grief,
    Defense,
}

#[derive(Clone, Debug)]
pub enum WarState {
    Declared,
    Active,
    Stalemate { years: u32 },
    Concluded { victor: Option<u32> },
}

impl AggregateWorld {
    pub fn new(regions: Vec<Region>, polities: Vec<Polity>, rng: ChaCha8Rng) -> Self {
        Self {
            regions,
            polities,
            active_wars: Vec::new(),
            year: 0,
            rng,
        }
    }

    pub fn get_region(&self, id: u32) -> Option<&Region> {
        self.regions.get(id as usize)
    }

    pub fn get_polity(&self, id: u32) -> Option<&Polity> {
        self.polities.iter().find(|p| p.id == id)
    }

    pub fn get_polity_mut(&mut self, id: u32) -> Option<&mut Polity> {
        self.polities.iter_mut().find(|p| p.id == id)
    }

    pub fn next_war_id(&self) -> u32 {
        self.active_wars.iter().map(|w| w.id).max().unwrap_or(0) + 1
    }
}
```

Create `src/aggregate/region.rs`:
```rust
//! Region - pseudo-node representing strategic territory

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::types::Species;

/// A pseudo-node representing a strategic region (~100+ hexes when expanded)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Region {
    pub id: u32,
    pub name: String,

    // Geography
    pub terrain: Terrain,
    pub resources: ResourceType,
    pub neighbors: Vec<u32>,

    // Species fitness (0.0 to 1.0) - how suitable for each species
    pub fitness: HashMap<Species, f32>,

    // Ownership
    pub controller: Option<u32>,
    pub contested_by: Vec<u32>,

    // Population capacity
    pub max_population: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Mountain,
    Forest,
    Plains,
    Marsh,
    Coast,
    Desert,
    Hills,
    River,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    None,
    Iron,
    Gold,
    Timber,
    Grain,
    Stone,
    Fish,
    Gems,
}

impl Region {
    /// Calculate species fitness based on terrain
    pub fn calculate_fitness(terrain: Terrain) -> HashMap<Species, f32> {
        let mut fitness = HashMap::new();

        match terrain {
            Terrain::Mountain => {
                fitness.insert(Species::Dwarf, 1.0);
                fitness.insert(Species::Elf, 0.1);
                fitness.insert(Species::Human, 0.2);
            }
            Terrain::Hills => {
                fitness.insert(Species::Dwarf, 0.8);
                fitness.insert(Species::Elf, 0.5);
                fitness.insert(Species::Human, 0.7);
            }
            Terrain::Forest => {
                fitness.insert(Species::Dwarf, 0.2);
                fitness.insert(Species::Elf, 1.0);
                fitness.insert(Species::Human, 0.5);
            }
            Terrain::Plains => {
                fitness.insert(Species::Dwarf, 0.1);
                fitness.insert(Species::Elf, 0.3);
                fitness.insert(Species::Human, 0.9);
            }
            Terrain::Marsh => {
                fitness.insert(Species::Dwarf, 0.0);
                fitness.insert(Species::Elf, 0.2);
                fitness.insert(Species::Human, 0.3);
            }
            Terrain::Coast => {
                fitness.insert(Species::Dwarf, 0.3);
                fitness.insert(Species::Elf, 0.4);
                fitness.insert(Species::Human, 0.9);
            }
            Terrain::Desert => {
                fitness.insert(Species::Dwarf, 0.2);
                fitness.insert(Species::Elf, 0.1);
                fitness.insert(Species::Human, 0.4);
            }
            Terrain::River => {
                fitness.insert(Species::Dwarf, 0.4);
                fitness.insert(Species::Elf, 0.6);
                fitness.insert(Species::Human, 1.0);
            }
        }

        fitness
    }
}
```

Create `src/aggregate/polity.rs`:
```rust
//! Polity - nation/tribe/hold/grove and species-specific state

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::core::types::Species;

/// A polity (nation, tribe, hold, grove, etc.)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Polity {
    pub id: u32,
    pub name: String,
    pub species: Species,
    pub polity_type: PolityType,

    // Physical state
    pub population: u32,
    pub territory: HashSet<u32>,
    pub capital: u32,
    pub military_strength: f32,
    pub economic_strength: f32,

    // Cultural drift from species baseline
    pub cultural_drift: CulturalDrift,

    // Relations with other polities
    pub relations: HashMap<u32, Relation>,

    // Species-specific state
    pub species_state: SpeciesState,

    // Alive status
    pub alive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolityType {
    // Human
    Kingdom,
    Tribe,
    CityState,
    // Dwarf
    Clan,
    Hold,
    // Elf
    Grove,
    Court,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CulturalDrift {
    pub primary_drift: Option<(String, f32)>,
    pub secondary_drift: Option<(String, f32)>,
    pub traditions: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Relation {
    pub opinion: i32,  // -100 to +100
    pub trust: i32,    // -100 to +100
    pub at_war: bool,
    pub alliance: bool,
    pub grudges: Vec<Grudge>,
    pub treaties: Vec<Treaty>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grudge {
    pub id: u32,
    pub against: u32,
    pub reason: GrudgeReason,
    pub severity: f32,
    pub year_incurred: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GrudgeReason {
    Betrayal,
    TerritoryLost(u32),
    HoldsAncestralSite(u32),
    OathBroken,
    KinSlain { count: u32 },
    InsultGiven,
    DebtUnpaid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Treaty {
    pub id: u32,
    pub parties: Vec<u32>,
    pub terms: TreatyTerms,
    pub year_signed: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TreatyTerms {
    Peace,
    Trade,
    MilitaryAccess,
    Tribute { from: u32, to: u32, amount: u32 },
    Vassalage { vassal: u32, lord: u32 },
}

/// Species-specific state - enum variants for static dispatch
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpeciesState {
    Human(HumanState),
    Dwarf(DwarfState),
    Elf(ElfState),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HumanState {
    pub expansion_pressure: f32,
    pub internal_cohesion: f32,
    pub reputation: f32,
    pub piety: f32,
    pub factions: Vec<Faction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Faction {
    pub id: u32,
    pub name: String,
    pub power: f32,
    pub ideology: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DwarfState {
    pub grudge_ledger: HashMap<u32, Vec<Grudge>>,
    pub oaths: Vec<Oath>,
    pub ancestral_sites: Vec<u32>,
    pub craft_focus: CraftType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Oath {
    pub id: u32,
    pub sworn_to: Option<u32>,
    pub oath_type: OathType,
    pub year_sworn: u32,
    pub fulfilled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OathType {
    MutualDefense,
    Vengeance { target: u32 },
    Service { duration_years: u32 },
    Silence,
    Crafting { item: String },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CraftType {
    #[default]
    Stone,
    Metal,
    Gems,
    Weapons,
    Armor,
    Architecture,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ElfState {
    pub memory: Vec<HistoricalMemory>,
    pub grief_level: f32,
    pub pending_decisions: Vec<PendingDecision>,
    pub core_territory: HashSet<u32>,
    pub pattern_assessment: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoricalMemory {
    pub event_id: u32,
    pub year: u32,
    pub emotional_weight: f32,
    pub lesson_learned: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingDecision {
    pub trigger_event: u32,
    pub deliberation_started: u32,
    pub deliberation_required: u32,
    pub decision_type: DecisionType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecisionType {
    War { target: u32 },
    Alliance { with: u32 },
    Isolation,
    PatternIntervention { situation: u32 },
    Migration,
}

impl Polity {
    pub fn human_state(&self) -> Option<&HumanState> {
        match &self.species_state {
            SpeciesState::Human(s) => Some(s),
            _ => None,
        }
    }

    pub fn human_state_mut(&mut self) -> Option<&mut HumanState> {
        match &mut self.species_state {
            SpeciesState::Human(s) => Some(s),
            _ => None,
        }
    }

    pub fn dwarf_state(&self) -> Option<&DwarfState> {
        match &self.species_state {
            SpeciesState::Dwarf(s) => Some(s),
            _ => None,
        }
    }

    pub fn dwarf_state_mut(&mut self) -> Option<&mut DwarfState> {
        match &mut self.species_state {
            SpeciesState::Dwarf(s) => Some(s),
            _ => None,
        }
    }

    pub fn elf_state(&self) -> Option<&ElfState> {
        match &self.species_state {
            SpeciesState::Elf(s) => Some(s),
            _ => None,
        }
    }

    pub fn elf_state_mut(&mut self) -> Option<&mut ElfState> {
        match &mut self.species_state {
            SpeciesState::Elf(s) => Some(s),
            _ => None,
        }
    }
}
```

Create `src/aggregate/events.rs`:
```rust
//! Events and history logging

use serde::{Deserialize, Serialize};

use crate::core::types::Species;
use crate::aggregate::polity::{GrudgeReason, TreatyTerms, DecisionType};
use crate::aggregate::world::WarCause;

/// A historical event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: u32,
    pub year: u32,
    pub event_type: EventType,
    pub participants: Vec<u32>,
    pub location: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventType {
    // Wars
    WarDeclared { aggressor: u32, defender: u32, cause: WarCause },
    Battle { war_id: u32, location: u32, winner: u32, casualties: (u32, u32) },
    Siege { war_id: u32, target: u32, successful: bool },
    WarEnded { war_id: u32, victor: Option<u32> },

    // Diplomacy
    AllianceFormed { members: Vec<u32> },
    AllianceBroken { breaker: u32 },
    Treaty { parties: Vec<u32>, terms: TreatyTerms },
    Betrayal { betrayer: u32, victim: u32 },

    // Territory
    Expansion { polity: u32, region: u32 },
    RegionLost { loser: u32, winner: u32, region: u32 },
    Settlement { polity: u32, region: u32, name: String },

    // Internal
    CivilWar { polity: u32, faction_ids: Vec<u32> },
    PolityCollapsed { polity: u32, successor_states: Vec<u32> },
    PolityMerged { absorbed: u32, absorber: u32 },

    // Cultural
    TraditionAdopted { polity: u32, tradition: String },
    CulturalDrift { polity: u32, value: String, direction: f32 },

    // Disasters
    Plague { affected: Vec<u32>, severity: f32 },
    Famine { affected: Vec<u32> },

    // Dwarf-specific
    GrudgeDeclared { polity: u32, against: u32, reason: GrudgeReason },
    GrudgeSettled { polity: u32, against: u32 },
    OathSworn { polity: u32, oath_id: u32 },
    OathBroken { polity: u32, oath_id: u32 },

    // Elf-specific
    GriefEvent { polity: u32, intensity: f32 },
    DeliberationComplete { polity: u32, decision: DecisionType },
    Isolation { polity: u32 },
}

/// The complete history log
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HistoryLog {
    pub events: Vec<Event>,
    next_event_id: u32,
}

impl HistoryLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event(&mut self, event_type: EventType, year: u32, participants: Vec<u32>, location: Option<u32>) -> u32 {
        let id = self.next_event_id;
        self.next_event_id += 1;

        self.events.push(Event {
            id,
            year,
            event_type,
            participants,
            location,
        });

        id
    }

    pub fn events_for_year(&self, year: u32) -> impl Iterator<Item = &Event> {
        self.events.iter().filter(move |e| e.year == year)
    }

    pub fn events_for_polity(&self, polity_id: u32) -> impl Iterator<Item = &Event> {
        self.events.iter().filter(move |e| e.participants.contains(&polity_id))
    }
}
```

Create `src/aggregate/simulation.rs`:
```rust
//! Main simulation loop

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::HistoryLog;
use crate::aggregate::output::SimulationOutput;
use crate::aggregate::systems;
use crate::aggregate::species;

/// Configuration for the simulation
#[derive(Clone, Debug)]
pub struct SimulationConfig {
    pub map: MapConfig,
    pub polities: PolityConfig,
    pub years: u32,
}

#[derive(Clone, Debug)]
pub struct MapConfig {
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub mountain_frequency: f32,
    pub forest_frequency: f32,
    pub water_frequency: f32,
}

#[derive(Clone, Debug)]
pub struct PolityConfig {
    pub human_count: u32,
    pub dwarf_count: u32,
    pub elf_count: u32,
    pub avg_starting_territory: u32,
    pub misplaced_fraction: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            map: MapConfig {
                width: 40,
                height: 30,
                seed: 12345,
                mountain_frequency: 0.2,
                forest_frequency: 0.3,
                water_frequency: 0.1,
            },
            polities: PolityConfig {
                human_count: 20,
                dwarf_count: 10,
                elf_count: 8,
                avg_starting_territory: 15,
                misplaced_fraction: 0.1,
            },
            years: 250,
        }
    }
}

/// Run the aggregate simulation
pub fn simulate(config: SimulationConfig) -> SimulationOutput {
    let start = std::time::Instant::now();

    let rng = ChaCha8Rng::seed_from_u64(config.map.seed);

    // Generate world
    let regions = systems::generate_map(&config.map, rng.clone());
    let polities = systems::generate_polities(&regions, &config.polities, rng.clone());
    let mut world = AggregateWorld::new(regions, polities, rng);

    // Initialize relations between polities
    systems::initialize_relations(&mut world);

    let mut history = HistoryLog::new();

    // Main simulation loop
    for year in 0..config.years {
        world.year = year;

        // 1. Collect events from all alive polities
        let mut pending_events = Vec::new();

        for polity in world.polities.iter().filter(|p| p.alive) {
            let events = species::tick(polity, &world, year);
            pending_events.extend(events);
        }

        // 2. Sort events by priority
        pending_events.sort_by_key(|e| systems::event_priority(&e.event_type));

        // 3. Resolve events
        for event_type in pending_events {
            systems::resolve_event(&mut world, &mut history, event_type, year);
        }

        // 4. Process active wars
        systems::resolve_active_wars(&mut world, &mut history, year);

        // 5. End-of-year updates
        systems::update_populations(&mut world);
        systems::decay_relations(&mut world);
        systems::check_polity_viability(&mut world, &mut history, year);
        systems::apply_cultural_drift(&mut world, year);
    }

    let elapsed = start.elapsed();

    SimulationOutput::new(world, history, config.years, elapsed)
}
```

Create `src/aggregate/output.rs`:
```rust
//! Simulation output and serialization

use std::time::Duration;
use serde::{Deserialize, Serialize};

use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::HistoryLog;

/// Complete simulation output
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationOutput {
    pub final_world: WorldSnapshot,
    pub history: HistoryLog,
    pub statistics: SimulationStats,
}

/// Serializable snapshot of world state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub year: u32,
    pub regions: Vec<crate::aggregate::region::Region>,
    pub polities: Vec<crate::aggregate::polity::Polity>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationStats {
    pub years_simulated: u32,
    pub simulation_time_ms: u64,
    pub total_events: u32,
    pub wars_fought: u32,
    pub polities_at_start: u32,
    pub polities_at_end: u32,
    pub polities_destroyed: u32,
    pub polities_created: u32,
}

impl SimulationOutput {
    pub fn new(world: AggregateWorld, history: HistoryLog, years: u32, elapsed: Duration) -> Self {
        let polities_alive = world.polities.iter().filter(|p| p.alive).count() as u32;
        let polities_at_start = world.polities.len() as u32; // Approximate

        let wars_fought = history.events.iter()
            .filter(|e| matches!(e.event_type, crate::aggregate::events::EventType::WarDeclared { .. }))
            .count() as u32;

        Self {
            final_world: WorldSnapshot {
                year: world.year,
                regions: world.regions,
                polities: world.polities,
            },
            history,
            statistics: SimulationStats {
                years_simulated: years,
                simulation_time_ms: elapsed.as_millis() as u64,
                total_events: 0, // Will be filled
                wars_fought,
                polities_at_start,
                polities_at_end: polities_alive,
                polities_destroyed: polities_at_start.saturating_sub(polities_alive),
                polities_created: 0,
            },
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn summary(&self) -> String {
        format!(
            "Simulated {} years in {}ms\n{} events, {} wars, {} polities remain",
            self.statistics.years_simulated,
            self.statistics.simulation_time_ms,
            self.history.events.len(),
            self.statistics.wars_fought,
            self.statistics.polities_at_end,
        )
    }
}
```

Create `src/aggregate/systems/mod.rs`:
```rust
//! Simulation systems

mod generation;
mod expansion;
mod warfare;
mod diplomacy;
mod population;
mod resolution;

pub use generation::{generate_map, generate_polities, initialize_relations};
pub use expansion::find_expansion_targets;
pub use warfare::resolve_active_wars;
pub use diplomacy::decay_relations;
pub use population::update_populations;
pub use resolution::{resolve_event, event_priority, check_polity_viability, apply_cultural_drift};
```

Create `src/aggregate/species/mod.rs`:
```rust
//! Species-specific behavior

mod human;
mod dwarf;
mod elf;

use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;
use crate::core::types::Species;

/// Generate events for a polity based on its species
pub fn tick(polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType> {
    match polity.species {
        Species::Human => human::tick(polity, world, year),
        Species::Dwarf => dwarf::tick(polity, world, year),
        Species::Elf => elf::tick(polity, world, year),
    }
}
```

**Step 5: Verify compilation**

Run: `cd /home/astre/arc-citadel && cargo check 2>&1 | head -50`

Expected: Will fail due to missing system files - that's Task 2.

---

## Task 2: Systems Implementation - Generation

**Files:**
- Create: `src/aggregate/systems/generation.rs`

**Step 1: Create generation.rs**

```rust
//! World and polity generation

use std::collections::{HashMap, HashSet};
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::core::types::Species;
use crate::aggregate::region::{Region, Terrain, ResourceType};
use crate::aggregate::polity::{
    Polity, PolityType, CulturalDrift, Relation, SpeciesState,
    HumanState, DwarfState, ElfState, CraftType,
};
use crate::aggregate::simulation::{MapConfig, PolityConfig};

/// Generate the pseudo-node map
pub fn generate_map(config: &MapConfig, mut rng: ChaCha8Rng) -> Vec<Region> {
    let mut regions = Vec::new();
    let mut id = 0;

    for y in 0..config.height {
        for x in 0..config.width {
            let terrain = generate_terrain(x, y, config, &mut rng);
            let resources = generate_resources(terrain, &mut rng);
            let fitness = Region::calculate_fitness(terrain);

            let region = Region {
                id,
                name: format!("Region_{}", id),
                terrain,
                resources,
                neighbors: Vec::new(), // Will be filled after all regions created
                fitness,
                controller: None,
                contested_by: Vec::new(),
                max_population: calculate_max_pop(terrain, resources),
            };

            regions.push(region);
            id += 1;
        }
    }

    // Build neighbor graph
    for id in 0..(regions.len() as u32) {
        let neighbors = get_neighbors(id, config.width, config.height);
        regions[id as usize].neighbors = neighbors;
    }

    regions
}

fn generate_terrain(x: u32, y: u32, config: &MapConfig, rng: &mut ChaCha8Rng) -> Terrain {
    // Simple noise-based terrain generation
    let noise = simple_noise(x, y, config.seed);

    // Edge of map tends to be coast/water
    let edge_dist = (x.min(config.width - 1 - x).min(y).min(config.height - 1 - y)) as f32;
    let edge_factor = (edge_dist / 5.0).min(1.0);

    if edge_factor < 0.3 && rng.gen::<f32>() < config.water_frequency {
        return Terrain::Coast;
    }

    if noise > 0.7 && rng.gen::<f32>() < config.mountain_frequency {
        return Terrain::Mountain;
    }

    if noise > 0.5 && rng.gen::<f32>() < config.mountain_frequency * 0.7 {
        return Terrain::Hills;
    }

    if noise < 0.4 && rng.gen::<f32>() < config.forest_frequency {
        return Terrain::Forest;
    }

    if noise < 0.2 && rng.gen::<f32>() < 0.3 {
        return Terrain::Marsh;
    }

    if rng.gen::<f32>() < 0.1 {
        return Terrain::River;
    }

    Terrain::Plains
}

fn simple_noise(x: u32, y: u32, seed: u64) -> f32 {
    // Very simple pseudo-noise
    let n = (x as u64).wrapping_mul(374761393)
        .wrapping_add((y as u64).wrapping_mul(668265263))
        .wrapping_add(seed);
    let n = n.wrapping_mul(n).wrapping_mul(n);
    (n as f32) / (u64::MAX as f32)
}

fn generate_resources(terrain: Terrain, rng: &mut ChaCha8Rng) -> ResourceType {
    let roll: f32 = rng.gen();

    if roll > 0.7 {
        return ResourceType::None;
    }

    match terrain {
        Terrain::Mountain => {
            if roll < 0.2 { ResourceType::Gold }
            else if roll < 0.35 { ResourceType::Gems }
            else if roll < 0.5 { ResourceType::Iron }
            else { ResourceType::Stone }
        }
        Terrain::Hills => {
            if roll < 0.3 { ResourceType::Iron }
            else if roll < 0.5 { ResourceType::Stone }
            else { ResourceType::None }
        }
        Terrain::Forest => {
            if roll < 0.5 { ResourceType::Timber }
            else { ResourceType::None }
        }
        Terrain::Plains => {
            if roll < 0.4 { ResourceType::Grain }
            else { ResourceType::None }
        }
        Terrain::Coast | Terrain::River => {
            if roll < 0.4 { ResourceType::Fish }
            else { ResourceType::None }
        }
        _ => ResourceType::None,
    }
}

fn calculate_max_pop(terrain: Terrain, resources: ResourceType) -> u32 {
    let base = match terrain {
        Terrain::Plains => 10000,
        Terrain::River => 12000,
        Terrain::Coast => 8000,
        Terrain::Hills => 5000,
        Terrain::Forest => 4000,
        Terrain::Mountain => 2000,
        Terrain::Marsh => 1500,
        Terrain::Desert => 1000,
    };

    let resource_bonus = match resources {
        ResourceType::Grain => 3000,
        ResourceType::Fish => 2000,
        ResourceType::None => 0,
        _ => 1000,
    };

    base + resource_bonus
}

fn get_neighbors(id: u32, width: u32, height: u32) -> Vec<u32> {
    let x = id % width;
    let y = id / width;
    let mut neighbors = Vec::new();

    // 6-directional (hex-like) neighbors
    let offsets: [(i32, i32); 6] = [
        (-1, 0), (1, 0),  // Left, right
        (0, -1), (0, 1),  // Up, down
        (-1, if y % 2 == 0 { -1 } else { 1 }),
        (1, if y % 2 == 0 { -1 } else { 1 }),
    ];

    for (dx, dy) in offsets {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;

        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
            neighbors.push((ny as u32 * width) + nx as u32);
        }
    }

    neighbors
}

/// Generate initial polities
pub fn generate_polities(
    regions: &[Region],
    config: &PolityConfig,
    mut rng: ChaCha8Rng,
) -> Vec<Polity> {
    let mut polities = Vec::new();
    let mut claimed: HashSet<u32> = HashSet::new();

    // Generate polities for each species
    polities.extend(generate_species_polities(
        Species::Human, config.human_count, regions, &mut claimed, &mut rng, config
    ));
    polities.extend(generate_species_polities(
        Species::Dwarf, config.dwarf_count, regions, &mut claimed, &mut rng, config
    ));
    polities.extend(generate_species_polities(
        Species::Elf, config.elf_count, regions, &mut claimed, &mut rng, config
    ));

    polities
}

fn generate_species_polities(
    species: Species,
    count: u32,
    regions: &[Region],
    claimed: &mut HashSet<u32>,
    rng: &mut ChaCha8Rng,
    config: &PolityConfig,
) -> Vec<Polity> {
    let mut polities = Vec::new();

    // Find high-fitness regions for this species
    let mut candidates: Vec<(u32, f32)> = regions.iter()
        .filter(|r| !claimed.contains(&r.id))
        .map(|r| (r.id, *r.fitness.get(&species).unwrap_or(&0.0)))
        .collect();

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for i in 0..count {
        // Pick capital - mostly high fitness, sometimes low (misplaced)
        let is_misplaced = rng.gen::<f32>() < config.misplaced_fraction;

        let capital_id = if is_misplaced && candidates.len() > 10 {
            // Pick from bottom half
            let idx = rng.gen_range(candidates.len() / 2..candidates.len());
            candidates.remove(idx).0
        } else if !candidates.is_empty() {
            candidates.remove(0).0
        } else {
            continue; // No more space
        };

        claimed.insert(capital_id);

        // Claim starting territory around capital
        let mut territory = HashSet::new();
        territory.insert(capital_id);

        let capital_region = &regions[capital_id as usize];
        let mut frontier: Vec<u32> = capital_region.neighbors.clone();

        let target_size = config.avg_starting_territory.saturating_sub(1);

        while territory.len() < target_size as usize && !frontier.is_empty() {
            let idx = rng.gen_range(0..frontier.len());
            let next = frontier.remove(idx);

            if !claimed.contains(&next) {
                territory.insert(next);
                claimed.insert(next);

                // Add new neighbors to frontier
                for neighbor in &regions[next as usize].neighbors {
                    if !claimed.contains(neighbor) && !frontier.contains(neighbor) {
                        frontier.push(*neighbor);
                    }
                }
            }
        }

        // Create polity
        let polity_id = polities.len() as u32 + i;
        let polity_type = match species {
            Species::Human => {
                if territory.len() > 20 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::CityState }
                else { PolityType::Tribe }
            }
            Species::Dwarf => {
                if territory.len() > 10 { PolityType::Hold }
                else { PolityType::Clan }
            }
            Species::Elf => {
                if territory.len() > 15 { PolityType::Court }
                else { PolityType::Grove }
            }
        };

        let population = territory.len() as u32 * 500 + rng.gen_range(100..1000);

        let species_state = match species {
            Species::Human => SpeciesState::Human(HumanState {
                expansion_pressure: rng.gen_range(0.3..0.7),
                internal_cohesion: rng.gen_range(0.5..0.9),
                reputation: rng.gen_range(0.4..0.8),
                piety: rng.gen_range(0.2..0.8),
                factions: Vec::new(),
            }),
            Species::Dwarf => SpeciesState::Dwarf(DwarfState {
                grudge_ledger: HashMap::new(),
                oaths: Vec::new(),
                ancestral_sites: vec![capital_id],
                craft_focus: CraftType::Stone,
            }),
            Species::Elf => SpeciesState::Elf(ElfState {
                memory: Vec::new(),
                grief_level: rng.gen_range(0.0..0.2),
                pending_decisions: Vec::new(),
                core_territory: territory.clone(),
                pattern_assessment: rng.gen_range(0.5..0.8),
            }),
        };

        let polity = Polity {
            id: polity_id,
            name: generate_polity_name(species, rng),
            species,
            polity_type,
            population,
            territory,
            capital: capital_id,
            military_strength: population as f32 * 0.1,
            economic_strength: population as f32 * 0.08,
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state,
            alive: true,
        };

        polities.push(polity);
    }

    polities
}

fn generate_polity_name(species: Species, rng: &mut ChaCha8Rng) -> String {
    let prefixes = match species {
        Species::Human => ["Alden", "Bran", "Cael", "Dorn", "Eld", "Frey", "Grim", "Hal", "Isen", "Kael"],
        Species::Dwarf => ["Kaz", "Dun", "Bel", "Thor", "Grun", "Mor", "Dur", "Bal", "Khor", "Zar"],
        Species::Elf => ["Aen", "Cel", "Ith", "Lor", "Mel", "Sil", "Thal", "Val", "Yen", "Zeph"],
    };

    let suffixes = match species {
        Species::Human => ["mark", "ford", "heim", "dale", "wick", "ton", "bury", "wood", "vale", "gate"],
        Species::Dwarf => ["heim", "hold", "delve", "deep", "forge", "gard", "mount", "hall", "peak", "stone"],
        Species::Elf => ["wen", "dor", "las", "iel", "ion", "eth", "ath", "oth", "ril", "dal"],
    };

    let prefix = prefixes[rng.gen_range(0..prefixes.len())];
    let suffix = suffixes[rng.gen_range(0..suffixes.len())];

    format!("{}{}", prefix, suffix)
}

/// Initialize relations between all polities
pub fn initialize_relations(world: &mut AggregateWorld) {
    let polity_ids: Vec<u32> = world.polities.iter().map(|p| p.id).collect();

    for polity in &mut world.polities {
        for &other_id in &polity_ids {
            if other_id != polity.id {
                let other = world.polities.iter().find(|p| p.id == other_id).unwrap();

                // Same species = slightly positive, different = slightly negative
                let base_opinion = if polity.species == other.species { 10 } else { -10 };

                polity.relations.insert(other_id, Relation {
                    opinion: base_opinion,
                    trust: 0,
                    at_war: false,
                    alliance: false,
                    grudges: Vec::new(),
                    treaties: Vec::new(),
                });
            }
        }
    }
}
```

**Step 2: Verify compilation**

Run: `cd /home/astre/arc-citadel && cargo check 2>&1 | head -30`

---

## Task 3: Systems Implementation - Expansion

**Files:**
- Create: `src/aggregate/systems/expansion.rs`

**Step 1: Create expansion.rs**

```rust
//! Territory expansion system

use crate::aggregate::world::AggregateWorld;
use crate::aggregate::polity::Polity;
use crate::aggregate::region::Terrain;
use crate::core::types::Species;

/// Targets for expansion
pub struct ExpansionTargets {
    pub unclaimed: Vec<u32>,
    pub weak_neighbors: Vec<(u32, u32)>, // (region_id, controller_id)
}

/// Find expansion targets for a polity
pub fn find_expansion_targets(polity: &Polity, world: &AggregateWorld) -> ExpansionTargets {
    let mut unclaimed = Vec::new();
    let mut weak_neighbors = Vec::new();

    // Check all adjacent regions to our territory
    for &region_id in &polity.territory {
        if let Some(region) = world.get_region(region_id) {
            for &neighbor_id in &region.neighbors {
                if polity.territory.contains(&neighbor_id) {
                    continue;
                }

                if let Some(neighbor) = world.get_region(neighbor_id) {
                    // Check if terrain is suitable for this species
                    let fitness = neighbor.fitness.get(&polity.species).unwrap_or(&0.0);

                    let is_valid_terrain = match polity.species {
                        Species::Dwarf => matches!(neighbor.terrain, Terrain::Mountain | Terrain::Hills),
                        Species::Elf => matches!(neighbor.terrain, Terrain::Forest) && *fitness > 0.8,
                        Species::Human => *fitness > 0.3,
                    };

                    if neighbor.controller.is_none() && is_valid_terrain {
                        if !unclaimed.contains(&neighbor_id) {
                            unclaimed.push(neighbor_id);
                        }
                    } else if let Some(controller) = neighbor.controller {
                        if controller != polity.id {
                            // Check if controller is weak
                            if let Some(other) = world.get_polity(controller) {
                                if other.military_strength < polity.military_strength * 0.7 {
                                    weak_neighbors.push((neighbor_id, controller));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort unclaimed by fitness
    unclaimed.sort_by(|a, b| {
        let fa = world.get_region(*a)
            .and_then(|r| r.fitness.get(&polity.species))
            .unwrap_or(&0.0);
        let fb = world.get_region(*b)
            .and_then(|r| r.fitness.get(&polity.species))
            .unwrap_or(&0.0);
        fb.partial_cmp(fa).unwrap()
    });

    ExpansionTargets { unclaimed, weak_neighbors }
}

/// Calculate expansion pressure for humans
pub fn calculate_human_expansion_pressure(polity: &Polity, world: &AggregateWorld) -> f32 {
    let base = polity.human_state()
        .map(|s| s.expansion_pressure)
        .unwrap_or(0.0);

    // Pressure increases with population density
    let total_capacity: u32 = polity.territory.iter()
        .filter_map(|id| world.get_region(*id))
        .map(|r| r.max_population)
        .sum();

    let density = if total_capacity > 0 {
        polity.population as f32 / total_capacity as f32
    } else {
        0.0
    };

    base + (density * 0.5)
}
```

---

## Task 4: Systems Implementation - Warfare

**Files:**
- Create: `src/aggregate/systems/warfare.rs`

**Step 1: Create warfare.rs**

```rust
//! War resolution system

use rand::Rng;

use crate::aggregate::world::{AggregateWorld, War, WarState, WarCause};
use crate::aggregate::events::{HistoryLog, EventType};
use crate::core::types::Species;

/// Resolve all active wars for this year
pub fn resolve_active_wars(world: &mut AggregateWorld, history: &mut HistoryLog, year: u32) {
    let mut war_outcomes = Vec::new();

    for war in &world.active_wars {
        let outcome = resolve_war_year(war, world, year);
        war_outcomes.push((war.id, outcome));
    }

    // Apply outcomes
    for (war_id, outcome) in war_outcomes {
        match outcome {
            WarYearOutcome::Continues => {}
            WarYearOutcome::RegionChanged { region, from, to } => {
                transfer_region(world, region, from, to);
                history.add_event(
                    EventType::RegionLost { loser: from, winner: to, region },
                    year,
                    vec![from, to],
                    Some(region),
                );
            }
            WarYearOutcome::Ended { victor } => {
                if let Some(war) = world.active_wars.iter_mut().find(|w| w.id == war_id) {
                    war.state = WarState::Concluded { victor };
                }

                // End war relations
                let (aggressor, defender) = {
                    let war = world.active_wars.iter().find(|w| w.id == war_id).unwrap();
                    (war.aggressor, war.defender)
                };

                if let Some(p) = world.get_polity_mut(aggressor) {
                    if let Some(rel) = p.relations.get_mut(&defender) {
                        rel.at_war = false;
                    }
                }
                if let Some(p) = world.get_polity_mut(defender) {
                    if let Some(rel) = p.relations.get_mut(&aggressor) {
                        rel.at_war = false;
                    }
                }

                history.add_event(
                    EventType::WarEnded { war_id, victor },
                    year,
                    vec![aggressor, defender],
                    None,
                );
            }
        }
    }

    // Remove concluded wars
    world.active_wars.retain(|w| !matches!(w.state, WarState::Concluded { .. }));
}

enum WarYearOutcome {
    Continues,
    RegionChanged { region: u32, from: u32, to: u32 },
    Ended { victor: Option<u32> },
}

fn resolve_war_year(war: &War, world: &AggregateWorld, _year: u32) -> WarYearOutcome {
    let aggressor = match world.get_polity(war.aggressor) {
        Some(p) if p.alive => p,
        _ => return WarYearOutcome::Ended { victor: Some(war.defender) },
    };

    let defender = match world.get_polity(war.defender) {
        Some(p) if p.alive => p,
        _ => return WarYearOutcome::Ended { victor: Some(war.aggressor) },
    };

    // Calculate relative strength
    let aggressor_strength = calculate_war_strength(aggressor, war, world);
    let defender_strength = calculate_war_strength(defender, war, world);

    let total = aggressor_strength + defender_strength;
    if total <= 0.0 {
        return WarYearOutcome::Continues;
    }

    let aggressor_chance = aggressor_strength / total;

    // Roll for contested regions
    let mut rng = world.rng.clone();

    for &region_id in &war.contested_regions {
        if let Some(region) = world.get_region(region_id) {
            if region.controller == Some(war.defender) {
                let roll: f32 = rng.gen();
                if roll < aggressor_chance * 0.3 {
                    // Aggressor takes region
                    return WarYearOutcome::RegionChanged {
                        region: region_id,
                        from: war.defender,
                        to: war.aggressor,
                    };
                }
            }
        }
    }

    // Check war exhaustion
    let aggressor_exhausted = is_war_exhausted(aggressor, world);
    let defender_exhausted = is_war_exhausted(defender, world);

    // Dwarves never accept exhaustion for grudge wars
    let aggressor_gives_up = aggressor_exhausted &&
        !(aggressor.species == Species::Dwarf && matches!(war.cause, WarCause::Grudge(_)));

    if aggressor_gives_up {
        return WarYearOutcome::Ended { victor: Some(war.defender) };
    }

    if defender_exhausted {
        return WarYearOutcome::Ended { victor: Some(war.aggressor) };
    }

    WarYearOutcome::Continues
}

fn calculate_war_strength(polity: &Polity, _war: &War, world: &AggregateWorld) -> f32 {
    let base = polity.military_strength;

    // Terrain bonus if defending
    let terrain_bonus = polity.territory.iter()
        .filter_map(|id| world.get_region(*id))
        .map(|r| match r.terrain {
            crate::aggregate::region::Terrain::Mountain => 0.3,
            crate::aggregate::region::Terrain::Hills => 0.15,
            crate::aggregate::region::Terrain::Forest => 0.1,
            _ => 0.0,
        })
        .sum::<f32>() / polity.territory.len().max(1) as f32;

    base * (1.0 + terrain_bonus)
}

fn is_war_exhausted(polity: &Polity, _world: &AggregateWorld) -> bool {
    // Exhausted if population dropped significantly or territory shrunk
    polity.population < 500 || polity.territory.len() < 2
}

fn transfer_region(world: &mut AggregateWorld, region_id: u32, from: u32, to: u32) {
    // Update region controller
    if let Some(region) = world.regions.get_mut(region_id as usize) {
        region.controller = Some(to);
    }

    // Update polity territories
    if let Some(loser) = world.get_polity_mut(from) {
        loser.territory.remove(&region_id);
    }
    if let Some(winner) = world.get_polity_mut(to) {
        winner.territory.insert(region_id);
    }
}

/// Find contested regions between two polities
pub fn find_contested_regions(world: &AggregateWorld, p1: u32, p2: u32) -> Vec<u32> {
    let mut contested = Vec::new();

    if let Some(polity1) = world.get_polity(p1) {
        for &region_id in &polity1.territory {
            if let Some(region) = world.get_region(region_id) {
                for &neighbor_id in &region.neighbors {
                    if let Some(neighbor) = world.get_region(neighbor_id) {
                        if neighbor.controller == Some(p2) && !contested.contains(&neighbor_id) {
                            contested.push(neighbor_id);
                        }
                    }
                }
            }
        }
    }

    contested
}
```

---

## Task 5: Systems Implementation - Diplomacy

**Files:**
- Create: `src/aggregate/systems/diplomacy.rs`

**Step 1: Create diplomacy.rs**

```rust
//! Diplomacy and relations system

use crate::aggregate::world::AggregateWorld;

/// Decay relations over time
pub fn decay_relations(world: &mut AggregateWorld) {
    for polity in &mut world.polities {
        if !polity.alive { continue; }

        for relation in polity.relations.values_mut() {
            // Opinion decays toward neutral
            if relation.opinion > 0 {
                relation.opinion = (relation.opinion - 1).max(0);
            } else if relation.opinion < 0 {
                relation.opinion = (relation.opinion + 1).min(0);
            }

            // Trust decays slower
            if relation.trust > 0 && world.year % 5 == 0 {
                relation.trust = (relation.trust - 1).max(0);
            }
        }
    }
}
```

---

## Task 6: Systems Implementation - Population

**Files:**
- Create: `src/aggregate/systems/population.rs`

**Step 1: Create population.rs**

```rust
//! Population and economy system

use crate::aggregate::world::AggregateWorld;
use crate::core::types::Species;

/// Update populations for all polities
pub fn update_populations(world: &mut AggregateWorld) {
    // Collect population updates (to avoid borrow issues)
    let updates: Vec<(u32, u32, f32, f32)> = world.polities.iter()
        .filter(|p| p.alive)
        .map(|polity| {
            let base_growth = match polity.species {
                Species::Human => 1.01,  // 1% per year
                Species::Dwarf => 1.005, // 0.5% per year
                Species::Elf => 1.002,   // 0.2% per year
            };

            // Territory quality affects growth
            let territory_quality = calculate_territory_quality(polity, world);

            // War penalty
            let at_war = polity.relations.values().any(|r| r.at_war);
            let war_modifier = if at_war { 0.95 } else { 1.0 };

            // Calculate carrying capacity
            let capacity: u32 = polity.territory.iter()
                .filter_map(|id| world.get_region(*id))
                .map(|r| r.max_population)
                .sum();

            // Logistic growth - slow down as approaching capacity
            let density = if capacity > 0 {
                polity.population as f32 / capacity as f32
            } else {
                1.0
            };
            let logistic_modifier = (1.0 - density).max(0.0);

            let new_pop = (polity.population as f32
                * base_growth
                * territory_quality
                * war_modifier
                * (1.0 + logistic_modifier * 0.1)) as u32;

            // Update strengths
            let econ = new_pop as f32 * 0.08 * territory_quality;
            let mil = new_pop as f32 * 0.1;

            (polity.id, new_pop, econ, mil)
        })
        .collect();

    // Apply updates
    for (id, pop, econ, mil) in updates {
        if let Some(polity) = world.get_polity_mut(id) {
            polity.population = pop;
            polity.economic_strength = econ;
            polity.military_strength = mil;
        }
    }
}

fn calculate_territory_quality(polity: &crate::aggregate::polity::Polity, world: &AggregateWorld) -> f32 {
    if polity.territory.is_empty() {
        return 0.5;
    }

    let mut quality_sum = 0.0;

    for region_id in &polity.territory {
        if let Some(region) = world.get_region(*region_id) {
            let fitness = region.fitness.get(&polity.species).unwrap_or(&0.5);
            let resource_bonus = if region.resources != crate::aggregate::region::ResourceType::None {
                0.1
            } else {
                0.0
            };
            quality_sum += fitness + resource_bonus;
        }
    }

    quality_sum / polity.territory.len() as f32
}
```

---

## Task 7: Systems Implementation - Resolution

**Files:**
- Create: `src/aggregate/systems/resolution.rs`

**Step 1: Create resolution.rs**

```rust
//! Event resolution and misc systems

use crate::aggregate::world::{AggregateWorld, War, WarState, WarCause};
use crate::aggregate::events::{HistoryLog, EventType};
use crate::aggregate::systems::warfare::find_contested_regions;

/// Get priority for event ordering (lower = higher priority)
pub fn event_priority(event: &EventType) -> u32 {
    match event {
        EventType::WarDeclared { .. } => 10,
        EventType::GrudgeDeclared { .. } => 15,
        EventType::Betrayal { .. } => 20,
        EventType::AllianceFormed { .. } => 30,
        EventType::AllianceBroken { .. } => 35,
        EventType::Expansion { .. } => 50,
        EventType::Settlement { .. } => 55,
        EventType::CivilWar { .. } => 60,
        EventType::PolityCollapsed { .. } => 70,
        EventType::PolityMerged { .. } => 75,
        EventType::DeliberationComplete { .. } => 80,
        _ => 100,
    }
}

/// Resolve an event
pub fn resolve_event(
    world: &mut AggregateWorld,
    history: &mut HistoryLog,
    event: EventType,
    year: u32,
) {
    match &event {
        EventType::WarDeclared { aggressor, defender, cause } => {
            resolve_war_declaration(world, *aggressor, *defender, cause.clone(), year);
            history.add_event(event, year, vec![*aggressor, *defender], None);
        }

        EventType::Expansion { polity, region } => {
            resolve_expansion(world, *polity, *region);
            history.add_event(event, year, vec![*polity], Some(*region));
        }

        EventType::Betrayal { betrayer, victim } => {
            resolve_betrayal(world, *betrayer, *victim, year);
            history.add_event(event, year, vec![*betrayer, *victim], None);
        }

        EventType::GrudgeDeclared { polity, against, reason } => {
            add_grudge(world, *polity, *against, reason.clone(), year);
            history.add_event(event, year, vec![*polity, *against], None);
        }

        EventType::CivilWar { polity, faction_ids } => {
            resolve_civil_war(world, history, *polity, faction_ids, year);
        }

        EventType::AllianceFormed { members } => {
            form_alliance(world, members);
            history.add_event(event, year, members.clone(), None);
        }

        EventType::Isolation { polity } => {
            isolate_polity(world, *polity);
            history.add_event(event, year, vec![*polity], None);
        }

        EventType::GriefEvent { polity, intensity } => {
            add_grief(world, *polity, *intensity);
            history.add_event(event, year, vec![*polity], None);
        }

        EventType::DeliberationComplete { polity, decision } => {
            execute_elf_decision(world, *polity, decision, year);
            history.add_event(event, year, vec![*polity], None);
        }

        _ => {
            // Log other events without special handling
            history.add_event(event.clone(), year, vec![], None);
        }
    }
}

fn resolve_war_declaration(
    world: &mut AggregateWorld,
    aggressor: u32,
    defender: u32,
    cause: WarCause,
    year: u32,
) {
    // Set at_war flags
    if let Some(p) = world.get_polity_mut(aggressor) {
        if let Some(rel) = p.relations.get_mut(&defender) {
            rel.at_war = true;
            rel.opinion = (rel.opinion - 30).max(-100);
        }
    }
    if let Some(p) = world.get_polity_mut(defender) {
        if let Some(rel) = p.relations.get_mut(&aggressor) {
            rel.at_war = true;
            rel.opinion = (rel.opinion - 30).max(-100);
        }
    }

    // Create war record
    let war = War {
        id: world.next_war_id(),
        aggressor,
        defender,
        cause,
        start_year: year,
        state: WarState::Active,
        contested_regions: find_contested_regions(world, aggressor, defender),
    };

    world.active_wars.push(war);
}

fn resolve_expansion(world: &mut AggregateWorld, polity_id: u32, region_id: u32) {
    if let Some(region) = world.regions.get_mut(region_id as usize) {
        if region.controller.is_none() {
            region.controller = Some(polity_id);

            if let Some(polity) = world.get_polity_mut(polity_id) {
                polity.territory.insert(region_id);
            }
        }
    }
}

fn resolve_betrayal(world: &mut AggregateWorld, betrayer: u32, victim: u32, year: u32) {
    // Break alliance
    if let Some(p) = world.get_polity_mut(betrayer) {
        if let Some(rel) = p.relations.get_mut(&victim) {
            rel.alliance = false;
        }
    }
    if let Some(p) = world.get_polity_mut(victim) {
        if let Some(rel) = p.relations.get_mut(&betrayer) {
            rel.alliance = false;
            rel.opinion = (rel.opinion - 50).max(-100);
            rel.trust = (rel.trust - 50).max(-100);
        }
    }

    // Dwarves get a grudge
    if let Some(victim_polity) = world.get_polity_mut(victim) {
        if victim_polity.species == crate::core::types::Species::Dwarf {
            if let Some(state) = victim_polity.dwarf_state_mut() {
                let grudge = crate::aggregate::polity::Grudge {
                    id: state.grudge_ledger.values().map(|v| v.len()).sum::<usize>() as u32,
                    against: betrayer,
                    reason: crate::aggregate::polity::GrudgeReason::Betrayal,
                    severity: 1.0,
                    year_incurred: year,
                };
                state.grudge_ledger.entry(betrayer).or_default().push(grudge);
            }
        }
    }
}

fn add_grudge(
    world: &mut AggregateWorld,
    polity_id: u32,
    against: u32,
    reason: crate::aggregate::polity::GrudgeReason,
    year: u32,
) {
    if let Some(polity) = world.get_polity_mut(polity_id) {
        if let Some(state) = polity.dwarf_state_mut() {
            let grudge = crate::aggregate::polity::Grudge {
                id: state.grudge_ledger.values().map(|v| v.len()).sum::<usize>() as u32,
                against,
                reason,
                severity: 0.5,
                year_incurred: year,
            };
            state.grudge_ledger.entry(against).or_default().push(grudge);
        }
    }
}

fn resolve_civil_war(
    world: &mut AggregateWorld,
    history: &mut HistoryLog,
    polity_id: u32,
    _faction_ids: &[u32],
    year: u32,
) {
    // Simplified: polity splits in half
    if let Some(polity) = world.get_polity_mut(polity_id) {
        let territory_vec: Vec<u32> = polity.territory.iter().copied().collect();
        let split_point = territory_vec.len() / 2;

        if split_point > 0 {
            let rebel_territory: std::collections::HashSet<u32> =
                territory_vec[split_point..].iter().copied().collect();

            polity.territory = territory_vec[..split_point].iter().copied().collect();
            polity.population /= 2;

            // Create rebel state
            let rebel_id = world.polities.len() as u32;
            let mut rebel = polity.clone();
            rebel.id = rebel_id;
            rebel.name = format!("{}_Rebels", polity.name);
            rebel.territory = rebel_territory;

            if let Some(first) = rebel.territory.iter().next() {
                rebel.capital = *first;
            }

            world.polities.push(rebel);

            history.add_event(
                EventType::PolityCollapsed {
                    polity: polity_id,
                    successor_states: vec![polity_id, rebel_id]
                },
                year,
                vec![polity_id, rebel_id],
                None,
            );
        }
    }
}

fn form_alliance(world: &mut AggregateWorld, members: &[u32]) {
    for &member1 in members {
        for &member2 in members {
            if member1 != member2 {
                if let Some(p) = world.get_polity_mut(member1) {
                    if let Some(rel) = p.relations.get_mut(&member2) {
                        rel.alliance = true;
                        rel.opinion = (rel.opinion + 20).min(100);
                        rel.trust = (rel.trust + 10).min(100);
                    }
                }
            }
        }
    }
}

fn isolate_polity(world: &mut AggregateWorld, polity_id: u32) {
    if let Some(polity) = world.get_polity_mut(polity_id) {
        // Break all alliances, reduce relations
        for rel in polity.relations.values_mut() {
            rel.alliance = false;
        }

        // Set elf state to isolation
        if let Some(state) = polity.elf_state_mut() {
            state.grief_level *= 0.5; // Isolation helps heal grief
        }
    }
}

fn add_grief(world: &mut AggregateWorld, polity_id: u32, intensity: f32) {
    if let Some(polity) = world.get_polity_mut(polity_id) {
        if let Some(state) = polity.elf_state_mut() {
            state.grief_level = (state.grief_level + intensity).min(1.0);
        }
    }
}

fn execute_elf_decision(
    world: &mut AggregateWorld,
    polity_id: u32,
    decision: &crate::aggregate::polity::DecisionType,
    year: u32,
) {
    match decision {
        crate::aggregate::polity::DecisionType::War { target } => {
            resolve_war_declaration(
                world,
                polity_id,
                *target,
                WarCause::Grief,
                year,
            );
        }
        crate::aggregate::polity::DecisionType::Alliance { with } => {
            form_alliance(world, &[polity_id, *with]);
        }
        crate::aggregate::polity::DecisionType::Isolation => {
            isolate_polity(world, polity_id);
        }
        _ => {}
    }
}

/// Check if polities should die
pub fn check_polity_viability(world: &mut AggregateWorld, history: &mut HistoryLog, year: u32) {
    let dead_polities: Vec<u32> = world.polities.iter()
        .filter(|p| p.alive && (p.territory.is_empty() || p.population < 100))
        .map(|p| p.id)
        .collect();

    for polity_id in dead_polities {
        if let Some(polity) = world.get_polity_mut(polity_id) {
            polity.alive = false;
        }

        history.add_event(
            EventType::PolityCollapsed { polity: polity_id, successor_states: vec![] },
            year,
            vec![polity_id],
            None,
        );
    }
}

/// Apply cultural drift over time
pub fn apply_cultural_drift(world: &mut AggregateWorld, _year: u32) {
    // Simplified: just decay drift values slightly
    for polity in &mut world.polities {
        if !polity.alive { continue; }

        if let Some((_, ref mut val)) = polity.cultural_drift.primary_drift {
            *val *= 0.99;
        }
    }
}
```

---

## Task 8: Species Behavior - Human

**Files:**
- Create: `src/aggregate/species/human.rs`

**Step 1: Create human.rs**

```rust
//! Human species behavior - expansion, fragmentation, ambition

use crate::aggregate::polity::Polity;
use crate::aggregate::world::{AggregateWorld, WarCause};
use crate::aggregate::events::EventType;
use crate::aggregate::systems::expansion::{find_expansion_targets, calculate_human_expansion_pressure};

const EXPANSION_THRESHOLD: f32 = 0.6;
const CIVIL_WAR_THRESHOLD: f32 = 0.3;
const REPUTATION_CRISIS: f32 = 0.2;
const BETRAYAL_THRESHOLD: f32 = 0.7;

pub fn tick(polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType> {
    let mut events = Vec::new();

    let state = match polity.human_state() {
        Some(s) => s,
        None => return events,
    };

    // AMBITION: Expansion pressure
    let expansion_pressure = calculate_human_expansion_pressure(polity, world);

    if expansion_pressure > EXPANSION_THRESHOLD {
        let targets = find_expansion_targets(polity, world);

        if let Some(&easy_target) = targets.unclaimed.first() {
            events.push(EventType::Expansion { polity: polity.id, region: easy_target });
        } else if let Some(&(region, controller)) = targets.weak_neighbors.first() {
            // Consider war
            if should_declare_war(polity, controller, world) {
                events.push(EventType::WarDeclared {
                    aggressor: polity.id,
                    defender: controller,
                    cause: WarCause::Expansion,
                });
            }
        }
    }

    // INTERNAL COHESION: Risk of civil war
    if state.internal_cohesion < CIVIL_WAR_THRESHOLD && polity.territory.len() > 5 {
        events.push(EventType::CivilWar {
            polity: polity.id,
            faction_ids: vec![],
        });
    }

    // HONOR: Reputation crisis
    if state.reputation < REPUTATION_CRISIS {
        if let Some(target) = find_honor_target(polity, world) {
            events.push(EventType::WarDeclared {
                aggressor: polity.id,
                defender: target,
                cause: WarCause::Honor,
            });
        }
    }

    // BETRAYAL: Humans can break alliances
    if should_betray_ally(polity, world) {
        if let Some(victim) = pick_betrayal_victim(polity, world) {
            events.push(EventType::Betrayal { betrayer: polity.id, victim });
        }
    }

    events
}

fn should_declare_war(polity: &Polity, target: u32, world: &AggregateWorld) -> bool {
    // Already at war with them?
    if let Some(rel) = polity.relations.get(&target) {
        if rel.at_war || rel.alliance {
            return false;
        }
    }

    // Compare strength
    if let Some(target_polity) = world.get_polity(target) {
        polity.military_strength > target_polity.military_strength * 1.2
    } else {
        false
    }
}

fn find_honor_target(polity: &Polity, world: &AggregateWorld) -> Option<u32> {
    // Find a weak neighbor to pick a fight with
    polity.relations.iter()
        .filter(|(_, rel)| !rel.at_war && !rel.alliance && rel.opinion < 0)
        .filter_map(|(&id, _)| world.get_polity(id).map(|p| (id, p)))
        .filter(|(_, other)| other.alive && other.military_strength < polity.military_strength)
        .map(|(id, _)| id)
        .next()
}

fn should_betray_ally(polity: &Polity, _world: &AggregateWorld) -> bool {
    let state = match polity.human_state() {
        Some(s) => s,
        None => return false,
    };

    // Low cohesion + high expansion pressure = betrayal
    state.internal_cohesion < 0.5 && state.expansion_pressure > BETRAYAL_THRESHOLD
}

fn pick_betrayal_victim(polity: &Polity, world: &AggregateWorld) -> Option<u32> {
    polity.relations.iter()
        .filter(|(_, rel)| rel.alliance)
        .filter_map(|(&id, _)| world.get_polity(id).map(|p| (id, p)))
        .filter(|(_, other)| other.alive && other.military_strength < polity.military_strength)
        .map(|(id, _)| id)
        .next()
}
```

---

## Task 9: Species Behavior - Dwarf

**Files:**
- Create: `src/aggregate/species/dwarf.rs`

**Step 1: Create dwarf.rs**

```rust
//! Dwarf species behavior - grudges, oaths, stone-bound

use crate::aggregate::polity::{Polity, GrudgeReason};
use crate::aggregate::world::{AggregateWorld, WarCause};
use crate::aggregate::events::EventType;
use crate::aggregate::systems::expansion::find_expansion_targets;
use crate::aggregate::region::Terrain;

const GRUDGE_WAR_THRESHOLD: f32 = 0.8;
const ANCESTRAL_SITE_GRUDGE_SEVERITY: f32 = 0.6;
const EXPANSION_POP_THRESHOLD: u32 = 5000;

pub fn tick(polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType> {
    let mut events = Vec::new();

    let state = match polity.dwarf_state() {
        Some(s) => s,
        None => return events,
    };

    // GRUDGE-BALANCE: Unresolved grudges MUST be addressed
    for (&target_id, grudges) in &state.grudge_ledger {
        let total_severity: f32 = grudges.iter().map(|g| g.severity).sum();

        if total_severity > GRUDGE_WAR_THRESHOLD {
            // Check if we can reach them and aren't already at war
            if let Some(rel) = polity.relations.get(&target_id) {
                if rel.at_war {
                    continue;
                }
            }

            if let Some(target) = world.get_polity(target_id) {
                if target.alive && can_reach(polity, target, world) {
                    // Dwarves will declare grudge war even if outmatched
                    // This is NOT strategic. This is compulsive.
                    events.push(EventType::WarDeclared {
                        aggressor: polity.id,
                        defender: target_id,
                        cause: WarCause::Grudge(grudges.iter().map(|g| g.id).collect()),
                    });
                }
            }
        }
    }

    // STONE-DEBT: Only expand into appropriate terrain
    let targets = find_expansion_targets(polity, world);
    let valid_expansion: Vec<u32> = targets.unclaimed.iter()
        .filter(|&&region_id| {
            world.get_region(region_id)
                .map(|r| matches!(r.terrain, Terrain::Mountain | Terrain::Hills))
                .unwrap_or(false)
        })
        .copied()
        .collect();

    if !valid_expansion.is_empty() && polity.population > EXPANSION_POP_THRESHOLD {
        events.push(EventType::Expansion {
            polity: polity.id,
            region: valid_expansion[0],
        });
    }

    // ANCESTOR-WEIGHT: Check if ancestral sites are held by others
    for &site in &state.ancestral_sites {
        if !polity.territory.contains(&site) {
            if let Some(region) = world.get_region(site) {
                if let Some(holder) = region.controller {
                    if holder != polity.id {
                        // Check if we already have a grudge for this
                        let has_grudge = state.grudge_ledger
                            .get(&holder)
                            .map(|gs| gs.iter().any(|g| matches!(g.reason, GrudgeReason::HoldsAncestralSite(s) if s == site)))
                            .unwrap_or(false);

                        if !has_grudge {
                            events.push(EventType::GrudgeDeclared {
                                polity: polity.id,
                                against: holder,
                                reason: GrudgeReason::HoldsAncestralSite(site),
                            });
                        }
                    }
                }
            }
        }
    }

    // HOLD-BOND: Dwarves NEVER betray their Hold
    // This is not a decision. It's hardcoded. They can't.

    events
}

fn can_reach(polity: &Polity, target: &Polity, world: &AggregateWorld) -> bool {
    // Simple: check if any of our territory neighbors any of their territory
    for &our_region in &polity.territory {
        if let Some(region) = world.get_region(our_region) {
            for &neighbor in &region.neighbors {
                if target.territory.contains(&neighbor) {
                    return true;
                }
            }
        }
    }
    false
}
```

---

## Task 10: Species Behavior - Elf

**Files:**
- Create: `src/aggregate/species/elf.rs`

**Step 1: Create elf.rs**

```rust
//! Elf species behavior - memory, deliberation, grief, patterns

use crate::aggregate::polity::{Polity, DecisionType, PendingDecision, HistoricalMemory};
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;
use crate::aggregate::systems::expansion::find_expansion_targets;
use crate::aggregate::region::Terrain;
use crate::core::types::Species;

const DELIBERATION_MIN_YEARS: u32 = 5;
const DELIBERATION_MAX_YEARS: u32 = 20;
const GRIEF_PARALYSIS_THRESHOLD: f32 = 0.9;
const GRIEF_ERUPTION_THRESHOLD: f32 = 0.7;
const EXPANSION_POP_THRESHOLD: u32 = 8000;

pub fn tick(polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType> {
    let mut events = Vec::new();

    let state = match polity.elf_state() {
        Some(s) => s,
        None => return events,
    };

    // LONG-CONSEQUENCE: Check if any deliberations are complete
    for decision in &state.pending_decisions {
        let elapsed = year.saturating_sub(decision.deliberation_started);

        if elapsed >= decision.deliberation_required {
            // NOW they act - and it's decisive
            events.push(EventType::DeliberationComplete {
                polity: polity.id,
                decision: decision.decision_type.clone(),
            });
        }
    }

    // CHANGE-GRIEF: Calculate grief from recent changes
    let grief_this_year = calculate_grief_this_year(polity, world, year);

    if grief_this_year > 0.01 {
        events.push(EventType::GriefEvent {
            polity: polity.id,
            intensity: grief_this_year,
        });
    }

    let total_grief = state.grief_level + grief_this_year;

    if total_grief > GRIEF_PARALYSIS_THRESHOLD {
        // Withdraw from world
        events.push(EventType::Isolation { polity: polity.id });
    } else if total_grief > GRIEF_ERUPTION_THRESHOLD {
        // Lash out at primary grief source
        if let Some(target) = find_primary_grief_source(polity, world) {
            // Start deliberation about war (they don't attack immediately)
            if !state.pending_decisions.iter().any(|d|
                matches!(&d.decision_type, DecisionType::War { target: t } if *t == target)
            ) {
                // Will be picked up next year as a pending decision
                events.push(EventType::DeliberationComplete {
                    polity: polity.id,
                    decision: DecisionType::War { target },
                });
            }
        }
    }

    // FOREST-SELF: Absolute defense of core territory
    for &region_id in &state.core_territory {
        if let Some(region) = world.get_region(region_id) {
            if !region.contested_by.is_empty() {
                // Would trigger immediate military response
                // (handled in warfare system based on core_territory)
            }
        }
    }

    // SLOW-GROWTH: Elves rarely expand
    if polity.population > EXPANSION_POP_THRESHOLD {
        let targets = find_expansion_targets(polity, world);
        let perfect_forest: Option<u32> = targets.unclaimed.iter()
            .filter(|&&region_id| {
                world.get_region(region_id)
                    .map(|r| {
                        r.terrain == Terrain::Forest &&
                        r.fitness.get(&Species::Elf).unwrap_or(&0.0) > &0.9
                    })
                    .unwrap_or(false)
            })
            .copied()
            .next();

        if let Some(region) = perfect_forest {
            events.push(EventType::Expansion { polity: polity.id, region });
        }
    }

    events
}

fn calculate_grief_this_year(polity: &Polity, world: &AggregateWorld, _year: u32) -> f32 {
    let mut grief = 0.0;

    // Territory changes cause grief
    let state = match polity.elf_state() {
        Some(s) => s,
        None => return 0.0,
    };

    // Core territory violations are devastating
    for &core_region in &state.core_territory {
        if !polity.territory.contains(&core_region) {
            grief += 0.3; // Major grief for lost core territory
        }
    }

    // Population decline causes grief
    // (Would need to track previous population - simplified here)

    // Nearby destruction causes grief
    for &region_id in &polity.territory {
        if let Some(region) = world.get_region(region_id) {
            for &neighbor_id in &region.neighbors {
                if let Some(neighbor) = world.get_region(neighbor_id) {
                    // If formerly-elf territory is now non-elf
                    if let Some(controller) = neighbor.controller {
                        if let Some(other) = world.get_polity(controller) {
                            if other.species != Species::Elf {
                                grief += 0.02; // Minor grief for nearby non-elf presence
                            }
                        }
                    }
                }
            }
        }
    }

    grief.min(0.3) // Cap grief per year
}

fn find_primary_grief_source(polity: &Polity, world: &AggregateWorld) -> Option<u32> {
    // Find who's most responsible for grief - usually whoever took core territory
    let state = polity.elf_state()?;

    for &core_region in &state.core_territory {
        if !polity.territory.contains(&core_region) {
            if let Some(region) = world.get_region(core_region) {
                if let Some(controller) = region.controller {
                    if controller != polity.id {
                        return Some(controller);
                    }
                }
            }
        }
    }

    // Otherwise, find the most hated neighbor
    polity.relations.iter()
        .filter(|(_, rel)| rel.opinion < -30 && !rel.at_war)
        .min_by_key(|(_, rel)| rel.opinion)
        .map(|(&id, _)| id)
}
```

---

## Task 11: Binary Entry Point

**Files:**
- Create: `src/bin/aggregate_sim.rs`

**Step 1: Create aggregate_sim.rs**

```rust
//! Aggregate History Simulation binary

use std::time::Instant;
use arc_citadel::aggregate::{simulate, SimulationConfig};

fn main() {
    let config = SimulationConfig::default();

    println!("Starting Aggregate History Simulation");
    println!("======================================");
    println!("Map: {}x{} regions", config.map.width, config.map.height);
    println!("Polities: {} humans, {} dwarves, {} elves",
        config.polities.human_count,
        config.polities.dwarf_count,
        config.polities.elf_count,
    );
    println!("Simulating {} years...", config.years);
    println!();

    let start = Instant::now();
    let output = simulate(config);
    let elapsed = start.elapsed();

    println!("{}", output.summary());
    println!("Actual time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);

    // Write output to file
    let json = output.to_json();
    std::fs::write("simulation_output.json", &json).expect("Failed to write output");
    println!("\nFull output written to simulation_output.json");

    // Print some interesting stats
    println!("\n--- Species Summary ---");

    let humans_alive = output.final_world.polities.iter()
        .filter(|p| p.alive && p.species == arc_citadel::core::types::Species::Human)
        .count();
    let dwarves_alive = output.final_world.polities.iter()
        .filter(|p| p.alive && p.species == arc_citadel::core::types::Species::Dwarf)
        .count();
    let elves_alive = output.final_world.polities.iter()
        .filter(|p| p.alive && p.species == arc_citadel::core::types::Species::Elf)
        .count();

    println!("Humans: {} polities survived", humans_alive);
    println!("Dwarves: {} polities survived", dwarves_alive);
    println!("Elves: {} polities survived", elves_alive);
}
```

**Step 2: Verify compilation**

Run: `cd /home/astre/arc-citadel && cargo build --bin aggregate_sim`

**Step 3: Run simulation**

Run: `cd /home/astre/arc-citadel && cargo run --bin aggregate_sim --release`

Expected: Completes 250 years in under 60 seconds with interesting emergent patterns.

---

## Task 12: Verification and Testing

**Files:**
- Create: `tests/aggregate_tests.rs`

**Step 1: Create aggregate tests**

```rust
//! Tests for aggregate simulation

use arc_citadel::aggregate::*;
use arc_citadel::core::types::Species;

#[test]
fn test_simulation_completes() {
    let config = SimulationConfig {
        years: 50,
        ..Default::default()
    };

    let output = simulate(config);

    assert_eq!(output.statistics.years_simulated, 50);
    assert!(output.history.events.len() > 0);
}

#[test]
fn test_species_survive() {
    let config = SimulationConfig {
        years: 100,
        ..Default::default()
    };

    let output = simulate(config);

    let humans = output.final_world.polities.iter()
        .filter(|p| p.alive && p.species == Species::Human)
        .count();
    let dwarves = output.final_world.polities.iter()
        .filter(|p| p.alive && p.species == Species::Dwarf)
        .count();
    let elves = output.final_world.polities.iter()
        .filter(|p| p.alive && p.species == Species::Elf)
        .count();

    // All species should have survivors after 100 years
    assert!(humans > 0, "No humans survived");
    assert!(dwarves > 0, "No dwarves survived");
    assert!(elves > 0, "No elves survived");
}

#[test]
fn test_deterministic_simulation() {
    let config1 = SimulationConfig::default();
    let config2 = SimulationConfig::default();

    let output1 = simulate(config1);
    let output2 = simulate(config2);

    // Same seed should produce same number of events
    assert_eq!(output1.history.events.len(), output2.history.events.len());
}

#[test]
fn test_wars_occur() {
    let config = SimulationConfig {
        years: 100,
        ..Default::default()
    };

    let output = simulate(config);

    assert!(output.statistics.wars_fought > 0, "No wars occurred in 100 years");
}

#[test]
fn test_territory_changes() {
    let config = SimulationConfig {
        years: 100,
        ..Default::default()
    };

    let output = simulate(config);

    let territory_events = output.history.events.iter()
        .filter(|e| matches!(e.event_type,
            EventType::Expansion { .. } |
            EventType::RegionLost { .. }
        ))
        .count();

    assert!(territory_events > 0, "No territory changes occurred");
}

#[test]
fn test_performance() {
    use std::time::Instant;

    let config = SimulationConfig {
        years: 250,
        ..Default::default()
    };

    let start = Instant::now();
    let _output = simulate(config);
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 60,
        "Simulation took {}s, expected < 60s", elapsed.as_secs());
}
```

**Step 2: Run tests**

Run: `cd /home/astre/arc-citadel && cargo test aggregate --release`

---

## Validation Checklist

After completing all tasks, verify:

- [ ] `cargo build --release` succeeds
- [ ] `cargo run --bin aggregate_sim --release` completes in < 60 seconds
- [ ] All three species have surviving polities at year 250
- [ ] Wars occur and territory changes hands
- [ ] Dwarf grudge mechanics produce multi-generational conflicts
- [ ] Elf deliberation creates delayed-response patterns
- [ ] Human civil wars occur
- [ ] No single polity dominates everything
- [ ] `cargo test aggregate --release` passes all tests
- [ ] `simulation_output.json` is generated with valid JSON

---

## Success Criteria Verification Commands

```bash
# Build
cd /home/astre/arc-citadel && cargo build --release

# Run simulation
cargo run --bin aggregate_sim --release

# Run tests
cargo test aggregate --release -- --nocapture

# Check JSON output
jq '.statistics' simulation_output.json
jq '.history.events | length' simulation_output.json
```
