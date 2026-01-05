//! Run the historical simulation

use arc_citadel::aggregate::events::EventType;
use arc_citadel::aggregate::simulation::{simulate, MapConfig, PolityConfig, SimulationConfig};

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║            AGGREGATE HISTORY SIMULATION                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let config = SimulationConfig {
        map: MapConfig {
            width: 20, // Smaller map = more crowded
            height: 15,
            seed: 42,
            mountain_frequency: 0.15,
            forest_frequency: 0.25,
            water_frequency: 0.1,
        },
        polities: PolityConfig {
            human_count: 12, // More polities
            dwarf_count: 6,
            elf_count: 4,
            avg_starting_territory: 8, // Smaller starting territory
            misplaced_fraction: 0.15,
        },
        years: 250,
    };

    println!(
        "Generating world: {}x{} regions",
        config.map.width, config.map.height
    );
    println!(
        "Starting polities: {} humans, {} dwarves, {} elves",
        config.polities.human_count, config.polities.dwarf_count, config.polities.elf_count
    );
    println!("Simulating {} years...\n", config.years);

    let output = simulate(config);

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  SIMULATION COMPLETE");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("{}", output.summary());
    println!();

    // Show surviving polities
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  SURVIVING POLITIES");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut survivors: Vec<_> = output
        .final_world
        .polities
        .iter()
        .filter(|p| p.alive)
        .collect();
    // Sort by military strength (what actually matters for dominance)
    survivors.sort_by(|a, b| {
        b.military_strength
            .partial_cmp(&a.military_strength)
            .unwrap()
    });

    for (i, polity) in survivors.iter().enumerate() {
        let territory_count = output
            .final_world
            .regions
            .iter()
            .filter(|r| r.controller == Some(polity.id.0))
            .count();

        println!("  {}. {} ({:?})", i + 1, polity.name, polity.species);
        println!(
            "     Mil: {:>6.0} | Pop: {:>6} | Regions: {:>2} | Econ: {:>6.0}",
            polity.military_strength, polity.population, territory_count, polity.economic_strength
        );
    }

    // Show notable events
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  NOTABLE EVENTS (last 20 years)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let recent_events: Vec<_> = output
        .history
        .events
        .iter()
        .filter(|e| e.year >= 230)
        .filter(|e| {
            matches!(
                e.event_type,
                EventType::WarDeclared { .. }
                    | EventType::WarEnded { .. }
                    | EventType::PolityCollapsed { .. }
                    | EventType::Expansion { .. }
            )
        })
        .take(15)
        .collect();

    for event in recent_events {
        let desc = match &event.event_type {
            EventType::WarDeclared {
                aggressor,
                defender,
                ..
            } => {
                let agg_name = output
                    .final_world
                    .polities
                    .iter()
                    .find(|p| p.id.0 == *aggressor)
                    .map(|p| p.name.as_str())
                    .unwrap_or("???");
                let def_name = output
                    .final_world
                    .polities
                    .iter()
                    .find(|p| p.id.0 == *defender)
                    .map(|p| p.name.as_str())
                    .unwrap_or("???");
                format!("War: {} vs {}", agg_name, def_name)
            }
            EventType::WarEnded { victor, .. } => {
                let victor_name = victor
                    .and_then(|v| output.final_world.polities.iter().find(|p| p.id.0 == v))
                    .map(|p| p.name.as_str())
                    .unwrap_or("stalemate");
                format!("War ended - victor: {}", victor_name)
            }
            EventType::PolityCollapsed { polity, .. } => {
                let name = output
                    .final_world
                    .polities
                    .iter()
                    .find(|p| p.id.0 == *polity)
                    .map(|p| p.name.as_str())
                    .unwrap_or("???");
                format!("Collapsed: {}", name)
            }
            EventType::Expansion { polity, region } => {
                let name = output
                    .final_world
                    .polities
                    .iter()
                    .find(|p| p.id.0 == *polity)
                    .map(|p| p.name.as_str())
                    .unwrap_or("???");
                format!("{} expanded into region {}", name, region)
            }
            _ => format!("{:?}", event.event_type),
        };
        println!("  Year {:>3}: {}", event.year, desc);
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  WAR STATISTICS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("  Total wars: {}", output.statistics.wars_fought);
    println!(
        "  Polities destroyed: {}",
        output.statistics.polities_destroyed
    );
    println!(
        "  Simulation time: {}ms",
        output.statistics.simulation_time_ms
    );
    println!();
}
