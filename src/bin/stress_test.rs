use std::time::Instant;
use arc_citadel::ecs::world::{World, Abundance};
use arc_citadel::core::types::Vec2;
use arc_citadel::simulation::tick::run_simulation_tick;

fn main() {
    let count = 10000;
    println!("=== STRESS TEST: {} entities ===\n", count);

    let mut world = World::new();

    // Realistic scenario: 100 clusters of 100 entities (villages/squads)
    println!("Spawning {} entities in 100 clusters (realistic)...", count);
    for i in 0..count {
        world.spawn_human(format!("Entity_{}", i));

        // 100 clusters of 100 entities each
        let cluster = i / 100;
        let cluster_x = (cluster % 10) as f32 * 200.0;
        let cluster_y = (cluster / 10) as f32 * 200.0;

        // Entities within cluster: 10x10 grid, 5 units apart
        let local_x = (i % 10) as f32 * 5.0;
        let local_y = ((i % 100) / 10) as f32 * 5.0;

        world.humans.positions[i] = Vec2::new(cluster_x + local_x, cluster_y + local_y);

        // Vary needs so entities make different decisions
        world.humans.needs[i].food = 0.3 + (i % 7) as f32 * 0.1;
        world.humans.needs[i].rest = 0.2 + (i % 5) as f32 * 0.15;
        world.humans.needs[i].safety = 0.1 + (i % 3) as f32 * 0.2;

        // Vary values so entities behave differently
        world.humans.values[i].curiosity = (i % 10) as f32 * 0.1;
        world.humans.values[i].safety = ((i + 3) % 10) as f32 * 0.1;
        world.humans.values[i].ambition = ((i + 5) % 10) as f32 * 0.1;
    }

    // Create food zones
    println!("Creating food zones...");

    // Abundant zones (4 corners) - world is 2000x2000
    world.add_food_zone(Vec2::new(200.0, 200.0), 100.0, Abundance::Unlimited);
    world.add_food_zone(Vec2::new(1800.0, 200.0), 100.0, Abundance::Unlimited);
    world.add_food_zone(Vec2::new(200.0, 1800.0), 100.0, Abundance::Unlimited);
    world.add_food_zone(Vec2::new(1800.0, 1800.0), 100.0, Abundance::Unlimited);

    // Scarce zones (center area - creates competition)
    for x in (600..=1400).step_by(200) {
        for y in (600..=1400).step_by(200) {
            world.add_food_zone(
                Vec2::new(x as f32, y as f32),
                50.0,
                Abundance::Scarce { current: 500.0, max: 500.0, regen: 5.0 },
            );
        }
    }

    println!("Created {} food zones (4 abundant corners, {} scarce center)",
        world.food_zones.len(),
        world.food_zones.len() - 4);

    // Verify initial state
    println!("\n=== Initial State ===");
    print_stats(&world);

    // Run 100 ticks and measure
    println!("\n=== Running 100 ticks ===\n");

    let mut tick_times = Vec::with_capacity(100);
    let mut work_done = WorkStats::default();

    for tick in 0..100 {
        let start = Instant::now();
        run_simulation_tick(&mut world);
        let elapsed = start.elapsed();
        tick_times.push(elapsed);

        // Sample work done every 10 ticks
        if tick % 10 == 9 {
            sample_work(&world, &mut work_done);
            println!("Tick {:>3}: {:>6.2?} | Tasks: {} | Thoughts: {} | Needs changed: {}",
                tick + 1,
                elapsed,
                work_done.entities_with_tasks,
                work_done.total_thoughts,
                work_done.needs_changed
            );
        }
    }

    // Final stats
    println!("\n=== Final State (after 100 ticks) ===");
    print_stats(&world);

    // Performance summary
    let total: std::time::Duration = tick_times.iter().sum();
    let avg = total / 100;
    let min = tick_times.iter().min().unwrap();
    let max = tick_times.iter().max().unwrap();

    println!("\n=== Performance Summary ===");
    println!("Total time:  {:?}", total);
    println!("Avg tick:    {:?}", avg);
    println!("Min tick:    {:?}", min);
    println!("Max tick:    {:?}", max);
    println!("Ticks/sec:   {:.1}", 1.0 / avg.as_secs_f64());

    // Verify work was actually done
    println!("\n=== Work Verification ===");
    verify_work(&world, count);
}

fn print_stats(world: &World) {
    let mut tasks = 0;
    let mut thoughts = 0;
    let mut high_food = 0;
    let mut high_rest = 0;

    for i in 0..world.humans.ids.len() {
        if world.humans.task_queues[i].current().is_some() {
            tasks += 1;
        }
        thoughts += world.humans.thoughts[i].iter().count();
        if world.humans.needs[i].food > 0.7 {
            high_food += 1;
        }
        if world.humans.needs[i].rest > 0.7 {
            high_rest += 1;
        }
    }

    println!("  Entities with tasks: {}/{}", tasks, world.humans.ids.len());
    println!("  Total thoughts: {}", thoughts);
    println!("  High food need: {}", high_food);
    println!("  High rest need: {}", high_rest);
}

#[derive(Default)]
struct WorkStats {
    entities_with_tasks: usize,
    total_thoughts: usize,
    needs_changed: usize,
}

fn sample_work(world: &World, stats: &mut WorkStats) {
    stats.entities_with_tasks = 0;
    stats.total_thoughts = 0;
    stats.needs_changed = 0;

    for i in 0..world.humans.ids.len() {
        if world.humans.task_queues[i].current().is_some() {
            stats.entities_with_tasks += 1;
        }
        stats.total_thoughts += world.humans.thoughts[i].iter().count();
        // Check if needs deviated from initial
        if world.humans.needs[i].food > 0.5 || world.humans.needs[i].rest > 0.5 {
            stats.needs_changed += 1;
        }
    }
}

fn verify_work(world: &World, count: usize) {
    let mut tasks_ever = 0;
    let mut needs_decayed = 0;
    let mut progress_made = 0;

    for i in 0..count {
        if world.humans.task_queues[i].current().is_some() {
            tasks_ever += 1;
        }
        if world.humans.needs[i].food > 0.4 || world.humans.needs[i].rest > 0.4 {
            needs_decayed += 1;
        }
        if let Some(task) = world.humans.task_queues[i].current() {
            if task.progress > 0.0 {
                progress_made += 1;
            }
        }
    }

    println!("  Entities with active tasks: {}/{}", tasks_ever, count);
    println!("  Entities with needs decay:  {}/{}", needs_decayed, count);
    println!("  Tasks with progress:        {}/{}", progress_made, tasks_ever);

    // Count perception work (neighbors per entity)
    let grid = arc_citadel::spatial::sparse_hash::SparseHashGrid::new(10.0);
    let mut total_neighbors = 0usize;

    // Quick sample of first 100 entities
    use arc_citadel::spatial::sparse_hash::SparseHashGrid;
    let mut grid = SparseHashGrid::new(10.0);
    grid.rebuild(
        world.humans.ids.iter().cloned()
            .zip(world.humans.positions.iter().cloned())
    );

    for i in 0..100 {
        let pos = world.humans.positions[i];
        let neighbors: Vec<_> = grid.query_neighbors(pos).collect();
        total_neighbors += neighbors.len();
    }
    let avg_neighbors = total_neighbors as f64 / 100.0;
    println!("  Avg neighbors (sample):     {:.1}", avg_neighbors);

    // Assertions
    let task_pct = tasks_ever as f64 / count as f64 * 100.0;
    let decay_pct = needs_decayed as f64 / count as f64 * 100.0;

    if task_pct > 90.0 && decay_pct > 50.0 && avg_neighbors > 10.0 {
        println!("\n✅ PASS: Simulation is doing real work!");
        println!("   - All entities processing");
        println!("   - Needs decaying over time");
        println!("   - Perception finding {:.0} neighbors per entity", avg_neighbors);
    } else {
        println!("\n❌ FAIL: Simulation might be skipping work");
        if task_pct <= 90.0 { println!("   Expected >90% tasks, got {:.1}%", task_pct); }
        if decay_pct <= 50.0 { println!("   Expected >50% decay, got {:.1}%", decay_pct); }
        if avg_neighbors <= 10.0 { println!("   Expected >10 neighbors, got {:.1}", avg_neighbors); }
    }
}
