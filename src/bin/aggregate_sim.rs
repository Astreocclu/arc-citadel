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
