//! Arc Citadel - Entry Point
//!
//! This is the main entry point for the Arc Citadel simulation game.
//! It sets up the async runtime, spawns test entities, runs simulation ticks,
//! and provides a basic game loop for interacting with the simulation.

use arc_citadel::command::CommandExecutor;
use arc_citadel::core::error::Result;
use arc_citadel::ecs::world::World;
use arc_citadel::llm::client::LlmClient;
use arc_citadel::llm::context::GameContext;
use arc_citadel::llm::parser::{parse_command, IntentAction};
use arc_citadel::simulation::tick::run_simulation_tick;

use std::io::{self, Write};
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter("arc_citadel=debug")
        .init();

    tracing::info!("Arc Citadel starting...");

    // Create the async runtime for LLM calls
    let rt = Runtime::new()?;

    // Create the game world
    let mut world = World::new();

    // Spawn initial test population
    spawn_initial_population(&mut world);

    // Try to create LLM client (optional - works without it)
    let llm_client = LlmClient::from_env().ok();
    if llm_client.is_none() {
        tracing::warn!("LLM_API_KEY not set - running without natural language commands");
    }

    // Display welcome message
    println!("\n=== ARC CITADEL ===");
    println!("A deep simulation strategy game with emergent entity behavior");
    println!();
    println!("Commands:");
    println!("  tick / t        - Advance simulation by one tick");
    println!("  spawn <name>    - Spawn a new human entity");
    println!("  status / s      - Show detailed status");
    println!("  run <n>         - Run n simulation ticks");
    println!("  quit / q        - Exit the game");
    if llm_client.is_some() {
        println!("  <any text>      - Natural language command (parsed by LLM)");
    }
    println!();

    // Main game loop
    loop {
        // Display current status
        display_status(&world);

        // Prompt for input
        print!("> ");
        io::stdout().flush()?;

        // Read input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Handle empty input
        if input.is_empty() {
            continue;
        }

        // Handle quit command
        if input == "quit" || input == "q" {
            break;
        }

        // Handle tick command
        if input == "tick" || input == "t" {
            run_simulation_tick(&mut world);
            println!("Tick {} complete.", world.current_tick);
            continue;
        }

        // Handle status command
        if input == "status" || input == "s" {
            display_detailed_status(&world);
            continue;
        }

        // Handle run <n> command
        if input.starts_with("run ") {
            if let Ok(n) = input.strip_prefix("run ").unwrap().parse::<u32>() {
                println!("Running {} ticks...", n);
                for _ in 0..n {
                    run_simulation_tick(&mut world);
                }
                println!("Completed {} ticks. Now at tick {}.", n, world.current_tick);
            } else {
                println!("Usage: run <number>");
            }
            continue;
        }

        // Handle spawn command
        if input.starts_with("spawn ") {
            let name = input.strip_prefix("spawn ").unwrap();
            if name.is_empty() {
                println!("Usage: spawn <name>");
            } else {
                let id = world.spawn_human(name.into());
                println!("Spawned {} (ID: {:?})", name, id);
            }
            continue;
        }

        // Try LLM command parsing if available
        if let Some(ref client) = llm_client {
            let context = GameContext::from_world(&world);

            match rt.block_on(parse_command(client, input, &context)) {
                Ok(intent) => {
                    println!();
                    println!("Parsed Intent:");
                    println!("  Action: {:?}", intent.action);
                    if let Some(target) = &intent.target {
                        println!("  Target: {}", target);
                    }
                    if let Some(location) = &intent.location {
                        println!("  Location: {}", location);
                    }
                    if let Some(subjects) = &intent.subjects {
                        println!("  Subjects: {:?}", subjects);
                    }
                    println!("  Priority: {:?}", intent.priority);
                    println!("  Confidence: {:.0}%", intent.confidence * 100.0);
                    if !intent.ambiguous_concepts.is_empty() {
                        println!("  Ambiguous concepts: {:?}", intent.ambiguous_concepts);
                        println!("  (These may be interpreted differently by different species)");
                    }

                    // Execute the command
                    match &intent.action {
                        IntentAction::Query => {
                            // Handle queries (status, info) without creating tasks
                            println!("Query: {:?}", intent.target);
                        }
                        _ => {
                            // Execute command
                            let current_tick = world.current_tick;
                            let result =
                                CommandExecutor::execute(&mut world, &intent, current_tick);

                            if let Some(error) = &result.error {
                                println!("Command failed: {}", error);
                            } else if result.tasks_created > 0 {
                                println!("Assigned {} task(s) to:", result.tasks_created);
                                for (_, name) in &result.assigned_to {
                                    println!("  - {}", name);
                                }
                            } else {
                                println!("No tasks created");
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Could not parse command: {}", e);
                }
            }
        } else {
            println!("Unknown command. Available: tick, spawn <name>, status, run <n>, quit");
        }
    }

    println!(
        "\nGoodbye! Final state: {} entities, {} ticks elapsed.",
        world.entity_count(),
        world.current_tick
    );
    Ok(())
}

/// Spawn the initial population of test entities
fn spawn_initial_population(world: &mut World) {
    let names = ["Marcus", "Elena", "Thomas", "Sarah", "William"];
    for name in names {
        let id = world.spawn_human(name.into());

        // Set some varied values for testing emergent behavior
        if let Some(idx) = world.humans.index_of(id) {
            match name {
                "Marcus" => {
                    // Marcus is brave and curious
                    world.humans.values[idx].safety = 0.3;
                    world.humans.values[idx].curiosity = 0.8;
                    world.humans.values[idx].honor = 0.7;
                }
                "Elena" => {
                    // Elena is cautious and social
                    world.humans.values[idx].safety = 0.9;
                    world.humans.values[idx].love = 0.8;
                    world.humans.values[idx].comfort = 0.6;
                }
                "Thomas" => {
                    // Thomas is ambitious and hardworking
                    world.humans.values[idx].ambition = 0.9;
                    world.humans.values[idx].curiosity = 0.7;
                    world.humans.values[idx].comfort = 0.2;
                }
                "Sarah" => {
                    // Sarah values justice and loyalty
                    world.humans.values[idx].justice = 0.9;
                    world.humans.values[idx].loyalty = 0.8;
                    world.humans.values[idx].piety = 0.5;
                }
                "William" => {
                    // William appreciates beauty and comfort
                    world.humans.values[idx].beauty = 0.9;
                    world.humans.values[idx].comfort = 0.8;
                    world.humans.values[idx].ambition = 0.3;
                }
                _ => {}
            }
        }
    }
    tracing::info!("Spawned {} initial humans with varied values", names.len());
}

/// Display a brief status summary
fn display_status(world: &World) {
    println!();
    println!(
        "--- Tick {} | Population: {} ---",
        world.current_tick,
        world.entity_count()
    );

    for i in world.humans.iter_living().take(5) {
        let name = &world.humans.names[i];
        let body = &world.humans.body_states[i];
        let needs = &world.humans.needs[i];
        let (top_need, level) = needs.most_pressing();

        let task_desc = world.humans.task_queues[i]
            .current()
            .map(|t| format!("{:?}", t.action))
            .unwrap_or_else(|| "idle".to_string());

        println!(
            "  {} - Fatigue: {:.0}%, Top need: {:?} ({:.0}%), Task: {}",
            name,
            body.fatigue * 100.0,
            top_need,
            level * 100.0,
            task_desc
        );
    }

    if world.entity_count() > 5 {
        println!("  ... and {} more", world.entity_count() - 5);
    }
    println!();
}

/// Display detailed status of all entities
fn display_detailed_status(world: &World) {
    println!();
    println!("=== Detailed Status (Tick {}) ===", world.current_tick);
    println!();

    for i in world.humans.iter_living() {
        let name = &world.humans.names[i];
        let body = &world.humans.body_states[i];
        let needs = &world.humans.needs[i];
        let values = &world.humans.values[i];
        let thoughts = &world.humans.thoughts[i];
        let task_queue = &world.humans.task_queues[i];

        println!("{}", name);
        println!(
            "  Body: Fatigue {:.0}%, Hunger {:.0}%, Pain {:.0}%, Health {:.0}%",
            body.fatigue * 100.0,
            body.hunger * 100.0,
            body.pain * 100.0,
            body.overall_health * 100.0
        );
        println!(
            "  Needs: Rest {:.0}%, Food {:.0}%, Safety {:.0}%, Social {:.0}%, Purpose {:.0}%",
            needs.rest * 100.0,
            needs.food * 100.0,
            needs.safety * 100.0,
            needs.social * 100.0,
            needs.purpose * 100.0
        );

        let (dominant_value, level) = values.dominant();
        println!(
            "  Dominant value: {} ({:.0}%)",
            dominant_value,
            level * 100.0
        );

        if let Some(strongest) = thoughts.strongest() {
            println!(
                "  Strongest thought: {:?} ({}) - {:.0}% intensity",
                strongest.valence,
                strongest.concept_category,
                strongest.intensity * 100.0
            );
        }

        if let Some(task) = task_queue.current() {
            println!(
                "  Current task: {:?} (progress: {:.0}%)",
                task.action,
                task.progress * 100.0
            );
        } else {
            println!("  Current task: idle");
        }
        println!();
    }
}
