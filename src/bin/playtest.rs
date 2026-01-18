//! Automated playtester - runs the simulation and reports on gameplay quality
//!
//! This is a headless test that simulates what a player would experience,
//! issuing commands and observing outcomes.

use std::collections::HashMap;

use arc_citadel::actions::catalog::ActionId;
use arc_citadel::combat::WeaponProperties;
use arc_citadel::core::types::{EntityId, Vec2 as SimVec2};
use arc_citadel::ecs::world::{Abundance, World};
use arc_citadel::entity::tasks::{Task, TaskPriority, TaskSource};
use arc_citadel::simulation::tick::run_simulation_tick;
use arc_citadel::simulation::{check_win_condition, GameOutcome, SimulationEvent};

const WORLD_SIZE: f32 = 200.0;
const TEST_TICKS: u64 = 1000;

/// Simple pseudo-random number generator
fn pseudo_random(seed: &mut u64, min: f32, max: f32) -> f32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    let normalized = ((*seed >> 32) as u32) as f32 / u32::MAX as f32;
    min + normalized * (max - min)
}

fn main() {
    println!("=== Arc Citadel Automated Playtest ===\n");

    let mut world = World::new();
    let mut rng_seed: u64 = 42;

    // Setup: 50 humans, 10 orcs, grid of food zones (matching emergence_sim)

    // Create food zone grid across the map
    let food_zone_spacing = 40.0;
    let mut x = food_zone_spacing;
    while x < WORLD_SIZE {
        let mut y = food_zone_spacing;
        while y < WORLD_SIZE {
            world.add_food_zone(SimVec2::new(x, y), 15.0, Abundance::Unlimited);
            y += food_zone_spacing;
        }
        x += food_zone_spacing;
    }

    // Spawn humans with randomized needs and values (matching emergence_sim)
    let entity_spacing = 10.0;
    let mut human_count = 0;
    let mut y = 20.0;
    while y < WORLD_SIZE - 20.0 && human_count < 50 {
        let mut x = 20.0;
        while x < WORLD_SIZE - 20.0 && human_count < 50 {
            let id = world.spawn_human(format!("Human_{}", human_count));
            if let Some(idx) = world.humans.index_of(id) {
                world.humans.positions[idx] = SimVec2::new(x, y);

                // Randomize needs (matching emergence_sim)
                world.humans.needs[idx].food = pseudo_random(&mut rng_seed, 0.3, 0.7);
                world.humans.needs[idx].rest = pseudo_random(&mut rng_seed, 0.3, 0.7);
                world.humans.needs[idx].social = pseudo_random(&mut rng_seed, 0.3, 0.8);
                world.humans.needs[idx].safety = 0.0;
                world.humans.needs[idx].purpose = 0.0;

                // Randomize values (matching emergence_sim)
                world.humans.values[idx].honor = pseudo_random(&mut rng_seed, 0.0, 1.0);
                world.humans.values[idx].curiosity = pseudo_random(&mut rng_seed, 0.0, 1.0);
                world.humans.values[idx].loyalty = pseudo_random(&mut rng_seed, 0.0, 1.0);
                world.humans.values[idx].love = pseudo_random(&mut rng_seed, 0.0, 1.0);
                world.humans.values[idx].ambition = pseudo_random(&mut rng_seed, 0.0, 1.0);

                // Equip with swords so combat is effective
                world.humans.combat_states[idx].weapon = WeaponProperties::sword();
            }
            human_count += 1;
            x += entity_spacing;
        }
        y += entity_spacing;
    }

    // Spawn orcs scattered within the population area (not at edges)
    for i in 0..10 {
        let id = world.spawn_orc(format!("Orc_{}", i));
        // Random position within human area (20-180)
        let x = pseudo_random(&mut rng_seed, 20.0, WORLD_SIZE - 20.0);
        let y = pseudo_random(&mut rng_seed, 20.0, WORLD_SIZE - 20.0);
        if let Some(idx) = world.orcs.index_of(id) {
            world.orcs.positions[idx] = SimVec2::new(x, y);
        }
    }

    // Spawn dwarves in a corner
    for i in 0..5 {
        let id = world.spawn_dwarf(format!("Dwarf_{}", i));
        let x = pseudo_random(&mut rng_seed, 25.0, 50.0);
        let y = pseudo_random(&mut rng_seed, 25.0, 50.0);
        if let Some(idx) = world.dwarves.index_of(id) {
            world.dwarves.positions[idx] = SimVec2::new(x, y);
        }
    }

    // Spawn elves in another area
    for i in 0..5 {
        let id = world.spawn_elf(format!("Elf_{}", i));
        let x = pseudo_random(&mut rng_seed, 150.0, 180.0);
        let y = pseudo_random(&mut rng_seed, 150.0, 180.0);
        if let Some(idx) = world.elves.index_of(id) {
            world.elves.positions[idx] = SimVec2::new(x, y);
        }
    }

    let initial_humans = world.humans.count();
    let initial_orcs = world.orcs.count();
    let initial_dwarves = world.dwarves.count();
    let initial_elves = world.elves.count();

    println!("Initial state: {} humans, {} orcs, {} dwarves, {} elves",
             initial_humans, initial_orcs, initial_dwarves, initial_elves);
    println!("Running {} ticks...\n", TEST_TICKS);

    // Tracking metrics
    let mut action_counts: HashMap<ActionId, u32> = HashMap::new();
    let mut combat_events = 0;
    let mut deaths_human = 0;
    let mut deaths_orc = 0;
    let mut deaths_dwarf = 0;
    let mut deaths_elf = 0;
    let mut player_commands_issued = 0;
    let mut ticks_with_activity = 0;

    // Simulate player issuing some commands
    let player_commands = vec![
        (50, "move Human_0 to food zone"),
        (100, "attack Human_5 at nearest orc"),
        (150, "gather Human_10"),
        (200, "rest Human_20"),
        (250, "move Human_30 to center"),
    ];

    let mut prev_humans = world.humans.iter_living().count();
    let mut prev_orcs = world.orcs.iter_living().count();
    let mut prev_dwarves = world.dwarves.iter_living().count();
    let mut prev_elves = world.elves.iter_living().count();

    for tick in 0..TEST_TICKS {
        // Issue player commands at scheduled times
        for (cmd_tick, cmd_desc) in &player_commands {
            if tick == *cmd_tick {
                // Simulate player command
                issue_player_command(&mut world, tick, cmd_desc);
                player_commands_issued += 1;
                println!("[Tick {}] Player: {}", tick, cmd_desc);
            }
        }

        // Run simulation
        let events = run_simulation_tick(&mut world);

        // Track actions from events
        let mut tick_had_activity = false;
        let mut game_over = false;
        for event in &events {
            match event {
                SimulationEvent::TaskStarted { action, .. } => {
                    *action_counts.entry(*action).or_insert(0) += 1;
                    tick_had_activity = true;
                }
                SimulationEvent::CombatHit { .. } => {
                    combat_events += 1;
                    tick_had_activity = true;
                }
                SimulationEvent::GameOver { tick: end_tick, outcome } => {
                    println!("\n[Tick {}] Game Over: {:?}", end_tick, outcome);
                    game_over = true;
                }
                _ => {}
            }
        }

        if tick_had_activity {
            ticks_with_activity += 1;
        }

        // Check for deaths (count LIVING entities, not total spawned)
        let current_humans = world.humans.iter_living().count();
        let current_orcs = world.orcs.iter_living().count();
        let current_dwarves = world.dwarves.iter_living().count();
        let current_elves = world.elves.iter_living().count();
        if current_humans < prev_humans {
            deaths_human += prev_humans - current_humans;
            prev_humans = current_humans;
        }
        if current_orcs < prev_orcs {
            deaths_orc += prev_orcs - current_orcs;
            prev_orcs = current_orcs;
        }
        if current_dwarves < prev_dwarves {
            deaths_dwarf += prev_dwarves - current_dwarves;
            prev_dwarves = current_dwarves;
        }
        if current_elves < prev_elves {
            deaths_elf += prev_elves - current_elves;
            prev_elves = current_elves;
        }

        // End simulation if game over event was received
        if game_over {
            break;
        }
    }

    // Final state (count LIVING only)
    let final_humans = world.humans.iter_living().count();
    let final_orcs = world.orcs.iter_living().count();
    let final_dwarves = world.dwarves.iter_living().count();
    let final_elves = world.elves.iter_living().count();

    println!("\n=== PLAYTEST REPORT ===\n");

    // Outcome - use core win condition system
    println!("## Outcome");
    let outcome = check_win_condition(&world);
    match outcome {
        GameOutcome::Victory { orcs_killed } => {
            println!("Result: VICTORY (all {} orcs defeated)", orcs_killed);
        }
        GameOutcome::Defeat { humans_killed, dwarves_killed, elves_killed } => {
            println!("Result: DEFEAT (lost {} humans, {} dwarves, {} elves)",
                     humans_killed, dwarves_killed, elves_killed);
        }
        GameOutcome::Draw => {
            println!("Result: DRAW (mutual annihilation)");
        }
        GameOutcome::InProgress => {
            println!("Result: IN PROGRESS (simulation ended before conclusion)");
        }
    }
    println!("Survivors: {} humans, {} dwarves, {} elves, {} orcs",
             final_humans, final_dwarves, final_elves, final_orcs);
    println!("Casualties: {} humans, {} dwarves, {} elves, {} orcs",
             deaths_human, deaths_dwarf, deaths_elf, deaths_orc);
    println!();

    // Action diversity
    println!("## Action Diversity");
    let total_actions: u32 = action_counts.values().sum();
    let mut sorted_actions: Vec<_> = action_counts.iter().collect();
    sorted_actions.sort_by(|a, b| b.1.cmp(a.1));

    for (action, count) in &sorted_actions {
        let pct = (**count as f32 / total_actions as f32) * 100.0;
        println!("  {:?}: {} ({:.1}%)", action, **count, pct);
    }
    println!("Total: {} actions across {} types", total_actions, action_counts.len());
    println!();

    // Activity level
    println!("## Activity Level");
    let activity_pct = (ticks_with_activity as f32 / TEST_TICKS as f32) * 100.0;
    println!("Ticks with activity: {}/{} ({:.1}%)", ticks_with_activity, TEST_TICKS, activity_pct);
    println!("Combat events: {}", combat_events);
    println!("Player commands: {}", player_commands_issued);
    println!();

    // Gameplay quality assessment
    println!("## Quality Assessment");

    let mut issues = Vec::new();
    let mut positives = Vec::new();

    // Check action variety (should have at least 5 different actions)
    if action_counts.len() >= 5 {
        positives.push(format!("Good action variety ({} types)", action_counts.len()));
    } else {
        issues.push(format!("Low action variety (only {} types)", action_counts.len()));
    }

    // Check for combat engagement
    if combat_events > 0 {
        positives.push(format!("Combat system engaged ({} hits)", combat_events));
    } else {
        issues.push("No combat occurred - orcs and humans may not be encountering each other".to_string());
    }

    // Check activity level (should be active most ticks)
    if activity_pct > 80.0 {
        positives.push(format!("High activity level ({:.0}%)", activity_pct));
    } else if activity_pct > 50.0 {
        positives.push(format!("Moderate activity level ({:.0}%)", activity_pct));
    } else {
        issues.push(format!("Low activity level ({:.0}%) - entities may be stuck", activity_pct));
    }

    // Check for social behavior
    if action_counts.get(&ActionId::TalkTo).unwrap_or(&0) > &0 {
        positives.push("Social interactions occurring (TalkTo)".to_string());
    }

    // Check for survival behaviors
    if action_counts.get(&ActionId::Eat).unwrap_or(&0) > &0
        && action_counts.get(&ActionId::Rest).unwrap_or(&0) > &0 {
        positives.push("Survival behaviors active (Eat, Rest)".to_string());
    }

    // Check for threat response
    if action_counts.get(&ActionId::Flee).unwrap_or(&0) > &0
        || action_counts.get(&ActionId::Attack).unwrap_or(&0) > &0 {
        positives.push("Threat responses observed (Attack/Flee)".to_string());
    }

    println!("Positives:");
    for p in &positives {
        println!("  + {}", p);
    }

    if !issues.is_empty() {
        println!("\nIssues:");
        for i in &issues {
            println!("  - {}", i);
        }
    }

    // Overall score
    let score = positives.len() as f32 / (positives.len() + issues.len()) as f32 * 100.0;
    println!("\nOverall Score: {:.0}/100", score);

    if score >= 80.0 {
        println!("Verdict: PLAYABLE - Good emergent behavior");
    } else if score >= 60.0 {
        println!("Verdict: ACCEPTABLE - Some issues to address");
    } else {
        println!("Verdict: NEEDS WORK - Significant gameplay issues");
    }
}

/// Simulate a player command
fn issue_player_command(world: &mut World, tick: u64, description: &str) {
    // Parse simple command patterns and issue tasks
    // Collect indices first to avoid borrow conflicts
    if description.contains("move") && description.contains("food") {
        let idx: Option<usize> = world.humans.iter_living().next();
        if let Some(idx) = idx {
            let mut task = Task::new(ActionId::MoveTo, TaskPriority::High, tick);
            task.source = TaskSource::PlayerCommand;
            task.target_position = Some(SimVec2::new(WORLD_SIZE / 2.0, WORLD_SIZE / 2.0));
            world.humans.task_queues[idx].push(task);
        }
    } else if description.contains("move") && description.contains("center") {
        let idx: Option<usize> = world.humans.iter_living().nth(30);
        if let Some(idx) = idx {
            let mut task = Task::new(ActionId::MoveTo, TaskPriority::High, tick);
            task.source = TaskSource::PlayerCommand;
            task.target_position = Some(SimVec2::new(WORLD_SIZE / 2.0, WORLD_SIZE / 2.0));
            world.humans.task_queues[idx].push(task);
        }
    } else if description.contains("attack") {
        let human_idx: Option<usize> = world.humans.iter_living().nth(5);
        let orc_data: Option<(usize, EntityId)> = world.orcs.iter_living().next()
            .map(|idx| (idx, world.orcs.ids[idx]));
        if let (Some(human_idx), Some((_orc_idx, orc_id))) = (human_idx, orc_data) {
            let mut task = Task::new(ActionId::Attack, TaskPriority::Critical, tick);
            task.source = TaskSource::PlayerCommand;
            task.target_entity = Some(orc_id);
            world.humans.task_queues[human_idx].push(task);
        }
    } else if description.contains("gather") {
        let idx: Option<usize> = world.humans.iter_living().nth(10);
        if let Some(idx) = idx {
            let mut task = Task::new(ActionId::Gather, TaskPriority::High, tick);
            task.source = TaskSource::PlayerCommand;
            world.humans.task_queues[idx].push(task);
        }
    } else if description.contains("rest") {
        let idx: Option<usize> = world.humans.iter_living().nth(20);
        if let Some(idx) = idx {
            let mut task = Task::new(ActionId::Rest, TaskPriority::High, tick);
            task.source = TaskSource::PlayerCommand;
            world.humans.task_queues[idx].push(task);
        }
    }
}
