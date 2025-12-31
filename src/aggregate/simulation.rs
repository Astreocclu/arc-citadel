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
        pending_events.sort_by_key(|e| systems::event_priority(e));

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
