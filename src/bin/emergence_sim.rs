//! 10-minute emergence simulation
//! Observes what patterns emerge from 10,000 autonomous entities

use arc_citadel::actions::catalog::ActionId;
use arc_citadel::core::types::Vec2;
use arc_citadel::ecs::world::{Abundance, World};
use arc_citadel::simulation::tick::{run_simulation_tick, SimulationEvent};
use std::collections::HashMap;
use std::time::{Duration, Instant};

fn main() {
    let duration = Duration::from_secs(10 * 60); // 10 minutes
    let count = 10000;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          ARC CITADEL: 10-MINUTE EMERGENCE SIMULATION         ║");
    println!(
        "║                    {} entities                             ║",
        count
    );
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut world = World::new();
    let mut rng_seed = 42u64;

    // Spawn entities with varied personalities
    println!(
        "Spawning {} entities with diverse personalities...\n",
        count
    );
    for i in 0..count {
        world.spawn_human(generate_name(i, &mut rng_seed));

        // Spread across world - 10-unit spacing ensures neighbors within 50-unit perception
        // This creates density where entities can perceive ~25 neighbors for social interactions
        let x = (i % 100) as f32 * 10.0;
        let y = (i / 100) as f32 * 10.0;
        world.humans.positions[i] = Vec2::new(x, y);

        // Diverse starting needs - some entities start hungry/tired for observable behaviors
        world.humans.needs[i].food = pseudo_random(&mut rng_seed, 0.3, 0.7); // Higher food need
        world.humans.needs[i].rest = pseudo_random(&mut rng_seed, 0.3, 0.7); // Higher rest need
        world.humans.needs[i].safety = pseudo_random(&mut rng_seed, 0.0, 0.3);
        // Social range 0.3..0.8 ensures ~60% start above 0.35 threshold for immediate TalkTo
        // Combined with denser spacing, this creates observable social interactions from tick 1
        world.humans.needs[i].social = pseudo_random(&mut rng_seed, 0.3, 0.8);
        world.humans.needs[i].purpose = pseudo_random(&mut rng_seed, 0.2, 0.6);

        // Diverse values (creates different "personality types")
        world.humans.values[i].honor = pseudo_random(&mut rng_seed, 0.0, 1.0);
        world.humans.values[i].beauty = pseudo_random(&mut rng_seed, 0.0, 1.0);
        world.humans.values[i].comfort = pseudo_random(&mut rng_seed, 0.0, 1.0);
        world.humans.values[i].ambition = pseudo_random(&mut rng_seed, 0.0, 1.0);
        world.humans.values[i].loyalty = pseudo_random(&mut rng_seed, 0.0, 1.0);
        world.humans.values[i].love = pseudo_random(&mut rng_seed, 0.0, 1.0);
        world.humans.values[i].justice = pseudo_random(&mut rng_seed, 0.0, 1.0);
        world.humans.values[i].curiosity = pseudo_random(&mut rng_seed, 0.0, 1.0);
        world.humans.values[i].safety = pseudo_random(&mut rng_seed, 0.0, 1.0);
        world.humans.values[i].piety = pseudo_random(&mut rng_seed, 0.0, 1.0);
    }

    // Create food zones - spread across the ENTIRE map so entities can find food
    // Entities are placed in a 100x100 grid with spacing of 20 units (0,0) to (1980, 1980)
    // Perception range is 50 units, so food zones every 80 units ensures coverage
    println!("Creating food zones...\n");

    // Grid of food zones to match entity distribution
    // With 80-unit spacing and 40-unit radius, there's overlap for good coverage
    for x in (0..=2000).step_by(80) {
        for y in (0..=2000).step_by(80) {
            let abundance = if (x < 200 || x > 1800) && (y < 200 || y > 1800) {
                // Corner zones are abundant
                Abundance::Unlimited
            } else {
                // Inner zones are scarce to create competition
                Abundance::Scarce {
                    current: 100.0,
                    max: 100.0,
                    regen: 5.0,
                }
            };
            world.add_food_zone(Vec2::new(x as f32, y as f32), 40.0, abundance);
        }
    }

    println!(
        "Created {} food zones (4 abundant corners, {} scarce center)\n",
        world.food_zones.len(),
        world.food_zones.len() - 4
    );

    // Spawn hostile orcs scattered across the map for E4/E7 testing
    // These create threat scenarios that trigger safety and honor behaviors
    let orc_count = 100; // 1% of human population
    println!("Spawning {} hostile orcs...\n", orc_count);
    for i in 0..orc_count {
        let orc_name = format!("Orc_{}", i);
        world.spawn_orc(orc_name);

        // Scatter orcs within human population area (0-990 range matches human grid)
        // This ensures orcs are within perception range (50 units) of humans
        let x = pseudo_random(&mut rng_seed, 0.0, 990.0);
        let y = pseudo_random(&mut rng_seed, 0.0, 990.0);
        world.orcs.positions[i] = Vec2::new(x, y);

        // Orcs have high aggression - they will attack nearby humans
        world.orcs.needs[i].safety = 0.1; // Low safety need = aggressive
    }

    // Track emergence
    let mut tracker = EmergenceTracker::new();
    let start_time = Instant::now();
    let mut last_report = Instant::now();
    let mut tick_count = 0u64;

    println!("Starting simulation...\n");
    println!("Time     | Tick    | Ticks/s | Events");
    println!("---------|---------|---------|--------------------------------------------------");

    while start_time.elapsed() < duration {
        // Run simulation tick and capture events
        let events = run_simulation_tick(&mut world);
        tick_count += 1;

        // Log action events for evaluation (includes curiosity for E5, social for E6)
        for event in &events {
            match event {
                SimulationEvent::TaskStarted { entity_name, action, curiosity, social_need, honor } => {
                    // Include curiosity for IdleObserve (E5), social_need for TalkTo (E6), honor for Attack (E4)
                    println!("[ACTION] {} (curiosity={:.2}, social={:.2}, honor={:.2}) started {:?}", entity_name, curiosity, social_need, honor, action);
                }
                SimulationEvent::TaskCompleted { entity_name, action } => {
                    println!("[COMPLETE] {} finished {:?}", entity_name, action);
                }
                SimulationEvent::CombatHit { attacker, defender } => {
                    println!("[COMBAT] {} hit {}", attacker, defender);
                }
                SimulationEvent::ProductionComplete { building_idx, recipe } => {
                    println!("[PRODUCTION] Building {} produced {}", building_idx, recipe);
                }
            }
        }

        // Sample state every 100 ticks
        if tick_count % 100 == 0 {
            tracker.sample(&world, tick_count);
        }

        // Report every 30 seconds
        if last_report.elapsed() >= Duration::from_secs(30) {
            let elapsed = start_time.elapsed();
            let tps = tick_count as f64 / elapsed.as_secs_f64();

            let events = tracker.get_recent_events();
            let event_str = if events.is_empty() {
                "steady state".to_string()
            } else {
                events.join(", ")
            };

            println!(
                "{:>4}:{:02} | {:>7} | {:>7.1} | {}",
                elapsed.as_secs() / 60,
                elapsed.as_secs() % 60,
                tick_count,
                tps,
                truncate(&event_str, 50)
            );

            last_report = Instant::now();
            tracker.clear_recent_events();
        }
    }

    let total_time = start_time.elapsed();
    let final_tps = tick_count as f64 / total_time.as_secs_f64();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    SIMULATION COMPLETE                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("=== PERFORMANCE ===");
    println!("Total time:     {:?}", total_time);
    println!("Total ticks:    {}", tick_count);
    println!("Avg ticks/sec:  {:.1}", final_tps);
    println!();

    // Analyze emergence
    tracker.analyze_emergence(&world, tick_count);
}

struct EmergenceTracker {
    // Track action distributions over time
    action_history: Vec<HashMap<ActionId, usize>>,

    // Track need distributions
    need_snapshots: Vec<NeedSnapshot>,

    // Track notable events
    recent_events: Vec<String>,
    all_events: Vec<(u64, String)>,

    // Track personality clusters
    personality_clusters: Vec<PersonalityCluster>,
}

#[derive(Clone)]
struct NeedSnapshot {
    tick: u64,
    avg_food: f32,
    avg_rest: f32,
    avg_safety: f32,
    avg_social: f32,
    avg_purpose: f32,
    critical_count: usize,
}

#[derive(Clone)]
struct PersonalityCluster {
    dominant_value: String,
    count: usize,
    avg_position: Vec2,
}

impl EmergenceTracker {
    fn new() -> Self {
        Self {
            action_history: Vec::new(),
            need_snapshots: Vec::new(),
            recent_events: Vec::new(),
            all_events: Vec::new(),
            personality_clusters: Vec::new(),
        }
    }

    fn sample(&mut self, world: &World, tick: u64) {
        // Sample action distribution
        let mut action_counts: HashMap<ActionId, usize> = HashMap::new();
        for i in 0..world.humans.ids.len() {
            if let Some(task) = world.humans.task_queues[i].current() {
                *action_counts.entry(task.action).or_default() += 1;
            }
        }

        // Detect unusual action spikes
        let total: usize = action_counts.values().sum();
        for (action, count) in &action_counts {
            let pct = *count as f64 / total as f64 * 100.0;
            if pct > 30.0 && !matches!(action, ActionId::IdleWander | ActionId::IdleObserve) {
                self.recent_events
                    .push(format!("{:.0}% doing {:?}", pct, action));
                self.all_events
                    .push((tick, format!("{:.0}% doing {:?}", pct, action)));
            }
        }

        self.action_history.push(action_counts);

        // Sample need averages
        let mut food_sum = 0.0f32;
        let mut rest_sum = 0.0f32;
        let mut safety_sum = 0.0f32;
        let mut social_sum = 0.0f32;
        let mut purpose_sum = 0.0f32;
        let mut critical = 0usize;
        let n = world.humans.ids.len() as f32;

        for i in 0..world.humans.ids.len() {
            food_sum += world.humans.needs[i].food;
            rest_sum += world.humans.needs[i].rest;
            safety_sum += world.humans.needs[i].safety;
            social_sum += world.humans.needs[i].social;
            purpose_sum += world.humans.needs[i].purpose;

            if world.humans.needs[i].has_critical().is_some() {
                critical += 1;
            }
        }

        let snapshot = NeedSnapshot {
            tick,
            avg_food: food_sum / n,
            avg_rest: rest_sum / n,
            avg_safety: safety_sum / n,
            avg_social: social_sum / n,
            avg_purpose: purpose_sum / n,
            critical_count: critical,
        };

        // Detect need crises
        if critical > world.humans.ids.len() / 4 {
            let pct = critical as f64 / world.humans.ids.len() as f64 * 100.0;
            self.recent_events.push(format!("{:.0}% in crisis", pct));
            self.all_events
                .push((tick, format!("{:.0}% in crisis", pct)));
        }

        self.need_snapshots.push(snapshot);
    }

    fn get_recent_events(&self) -> Vec<String> {
        self.recent_events.clone()
    }

    fn clear_recent_events(&mut self) {
        self.recent_events.clear();
    }

    fn analyze_emergence(&self, world: &World, final_tick: u64) {
        println!("=== EMERGENT PATTERNS ===\n");

        // 1. Action distribution evolution
        println!("Action Distribution Over Time:");
        if let Some(first) = self.action_history.first() {
            if let Some(last) = self.action_history.last() {
                println!("  Start vs End:");
                let all_actions: Vec<ActionId> = first.keys().chain(last.keys()).cloned().collect();
                for action in all_actions.iter().take(8) {
                    let start = first.get(action).unwrap_or(&0);
                    let end = last.get(action).unwrap_or(&0);
                    if *start > 0 || *end > 0 {
                        let delta = *end as i64 - *start as i64;
                        let symbol = if delta > 100 {
                            "↑"
                        } else if delta < -100 {
                            "↓"
                        } else {
                            "→"
                        };
                        println!(
                            "    {:?}: {} {} {} {}",
                            action,
                            start,
                            symbol,
                            end,
                            if delta.abs() > 100 {
                                format!("({:+})", delta)
                            } else {
                                String::new()
                            }
                        );
                    }
                }
            }
        }
        println!();

        // 2. Need trends
        println!("Need Evolution:");
        if let (Some(first), Some(last)) = (self.need_snapshots.first(), self.need_snapshots.last())
        {
            println!(
                "  Food:    {:.2} → {:.2} {}",
                first.avg_food,
                last.avg_food,
                trend(first.avg_food, last.avg_food)
            );
            println!(
                "  Rest:    {:.2} → {:.2} {}",
                first.avg_rest,
                last.avg_rest,
                trend(first.avg_rest, last.avg_rest)
            );
            println!(
                "  Safety:  {:.2} → {:.2} {}",
                first.avg_safety,
                last.avg_safety,
                trend(first.avg_safety, last.avg_safety)
            );
            println!(
                "  Social:  {:.2} → {:.2} {}",
                first.avg_social,
                last.avg_social,
                trend(first.avg_social, last.avg_social)
            );
            println!(
                "  Purpose: {:.2} → {:.2} {}",
                first.avg_purpose,
                last.avg_purpose,
                trend(first.avg_purpose, last.avg_purpose)
            );
            println!(
                "  Crisis:  {} → {} entities",
                first.critical_count, last.critical_count
            );
        }
        println!();

        // 3. Personality type analysis
        println!("Personality Types (by dominant value):");
        let mut type_counts: HashMap<&str, usize> = HashMap::new();
        for i in 0..world.humans.ids.len() {
            let (dominant, _) = world.humans.values[i].dominant();
            *type_counts.entry(dominant).or_default() += 1;
        }
        let mut sorted: Vec<_> = type_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        for (value, count) in sorted.iter().take(5) {
            let pct = *count as f64 / world.humans.ids.len() as f64 * 100.0;
            println!("  {}: {} ({:.1}%)", value, count, pct);
        }
        println!();

        // 4. Behavioral patterns by personality
        println!("Behavior by Personality Type:");
        let mut behavior_by_type: HashMap<&str, HashMap<ActionId, usize>> = HashMap::new();
        for i in 0..world.humans.ids.len() {
            let (dominant, _) = world.humans.values[i].dominant();
            if let Some(task) = world.humans.task_queues[i].current() {
                *behavior_by_type
                    .entry(dominant)
                    .or_default()
                    .entry(task.action)
                    .or_default() += 1;
            }
        }
        for (ptype, actions) in behavior_by_type.iter().take(4) {
            let mut sorted: Vec<_> = actions.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            let top_action = sorted
                .first()
                .map(|(a, _)| format!("{:?}", a))
                .unwrap_or("none".to_string());
            println!("  {} → mostly {:?}", ptype, top_action);
        }
        println!();

        // 5. Notable events timeline
        if !self.all_events.is_empty() {
            println!("Notable Events Timeline:");
            for (tick, event) in self.all_events.iter().take(10) {
                let time_pct = *tick as f64 / final_tick as f64 * 100.0;
                println!("  [{:>5.1}%] {}", time_pct, event);
            }
            if self.all_events.len() > 10 {
                println!("  ... and {} more events", self.all_events.len() - 10);
            }
        } else {
            println!("Notable Events: None (stable simulation)");
        }
        println!();

        // 6. Spatial clustering
        println!("Spatial Distribution:");
        let mut quadrant_counts = [0usize; 4];
        for i in 0..world.humans.ids.len() {
            let pos = world.humans.positions[i];
            let q = match (pos.x > 1000.0, pos.y > 1000.0) {
                (false, false) => 0,
                (true, false) => 1,
                (false, true) => 2,
                (true, true) => 3,
            };
            quadrant_counts[q] += 1;
        }
        println!("  NW: {} | NE: {}", quadrant_counts[2], quadrant_counts[3]);
        println!("  SW: {} | SE: {}", quadrant_counts[0], quadrant_counts[1]);
        println!();

        // 7. Summary
        println!("=== EMERGENCE SUMMARY ===");
        let stable = self.all_events.len() < 5;
        let balanced = self
            .need_snapshots
            .last()
            .map(|s| s.critical_count < world.humans.ids.len() / 10)
            .unwrap_or(true);

        if stable && balanced {
            println!("✅ Simulation reached stable equilibrium");
            println!("   - Entities found sustainable behavior patterns");
            println!("   - Needs stayed within healthy ranges");
        } else if !stable {
            println!("⚡ Simulation showed dynamic behavior");
            println!("   - {} notable events occurred", self.all_events.len());
            println!("   - Population underwent behavioral shifts");
        } else {
            println!("⚠️  Simulation showed stress patterns");
            println!("   - Some entities struggled to meet needs");
        }
    }
}

fn pseudo_random(seed: &mut u64, min: f32, max: f32) -> f32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    // Extract upper 32 bits (not 31!) and divide by u32::MAX for 0.0-1.0 range
    let t = ((*seed >> 32) as u32 as f32) / (u32::MAX as f32);
    min + t * (max - min)
}

fn generate_name(_i: usize, seed: &mut u64) -> String {
    let first_names = [
        "Ada", "Bjorn", "Cira", "Dag", "Eira", "Finn", "Greta", "Hans", "Ingrid", "Jorn", "Kira",
        "Lars", "Mira", "Nils", "Olga", "Per",
    ];
    let last_names = [
        "Stone", "River", "Hill", "Wood", "Field", "Brook", "Dale", "Marsh", "Glen", "Vale",
        "Cliff", "Shore", "Moor", "Heath", "Fen", "Wold",
    ];

    let fi = (pseudo_random(seed, 0.0, 16.0) as usize) % 16;
    let li = (pseudo_random(seed, 0.0, 16.0) as usize) % 16;
    format!("{} {}", first_names[fi], last_names[li])
}

fn trend(start: f32, end: f32) -> &'static str {
    let delta = end - start;
    if delta > 0.1 {
        "↑ (increasing)"
    } else if delta < -0.1 {
        "↓ (decreasing)"
    } else {
        "→ (stable)"
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
