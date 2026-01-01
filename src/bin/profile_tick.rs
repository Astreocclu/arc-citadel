use std::time::Instant;
use arc_citadel::ecs::world::World;
use arc_citadel::core::types::Vec2;

fn main() {
    let count = 10000;
    println!("Profiling tick phases with {} entities\n", count);

    let mut world = World::new();

    // Spawn entities spread across world
    for i in 0..count {
        world.spawn_human(format!("Entity_{}", i));
        let x = (i % 100) as f32 * 10.0;
        let y = (i / 100) as f32 * 10.0;
        world.humans.positions[i] = Vec2::new(x, y);
    }

    // Warm up
    for _ in 0..5 {
        arc_citadel::simulation::tick::run_simulation_tick(&mut world);
    }

    // Profile 100 ticks
    let mut times = TickTimes::default();

    for _ in 0..100 {
        profile_tick(&mut world, &mut times);
    }

    println!("=== Average times per tick (100 samples) ===\n");
    println!("Phase           | Time       | % of total");
    println!("----------------|------------|------------");

    let total = times.total / 100;
    println!("Needs update    | {:>8.2?} | {:>5.1}%", times.needs / 100, pct(times.needs, times.total));
    println!("Perception      | {:>8.2?} | {:>5.1}%", times.perception / 100, pct(times.perception, times.total));
    println!("Thought gen     | {:>8.2?} | {:>5.1}%", times.thought_gen / 100, pct(times.thought_gen, times.total));
    println!("Thought decay   | {:>8.2?} | {:>5.1}%", times.thought_decay / 100, pct(times.thought_decay, times.total));
    println!("Action select   | {:>8.2?} | {:>5.1}%", times.action_select / 100, pct(times.action_select, times.total));
    println!("Task execute    | {:>8.2?} | {:>5.1}%", times.task_execute / 100, pct(times.task_execute, times.total));
    println!("----------------|------------|------------");
    println!("TOTAL           | {:>8.2?} | 100.0%", total);
    println!("\nTarget: <16.6ms for 60 ticks/sec");
}

fn pct(part: std::time::Duration, total: std::time::Duration) -> f64 {
    (part.as_nanos() as f64 / total.as_nanos() as f64) * 100.0
}

#[derive(Default)]
struct TickTimes {
    needs: std::time::Duration,
    perception: std::time::Duration,
    thought_gen: std::time::Duration,
    thought_decay: std::time::Duration,
    action_select: std::time::Duration,
    task_execute: std::time::Duration,
    total: std::time::Duration,
}

fn profile_tick(world: &mut World, times: &mut TickTimes) {
    let tick_start = Instant::now();

    // Phase 1: Needs
    let start = Instant::now();
    update_needs(world);
    times.needs += start.elapsed();

    // Phase 2: Perception
    let start = Instant::now();
    let perceptions = run_perception(world);
    times.perception += start.elapsed();

    // Phase 3: Thought generation
    let start = Instant::now();
    generate_thoughts(world, &perceptions);
    times.thought_gen += start.elapsed();

    // Phase 4: Thought decay
    let start = Instant::now();
    decay_thoughts(world);
    times.thought_decay += start.elapsed();

    // Phase 5: Action selection
    let start = Instant::now();
    select_actions(world);
    times.action_select += start.elapsed();

    // Phase 6: Task execution
    let start = Instant::now();
    execute_tasks(world);
    times.task_execute += start.elapsed();

    world.tick();
    times.total += tick_start.elapsed();
}

// Inline the tick functions here for profiling
use arc_citadel::spatial::sparse_hash::SparseHashGrid;
use arc_citadel::simulation::perception::{Perception, PerceivedEntity, RelationshipType};
use arc_citadel::simulation::action_select::{select_action_human, SelectionContext};
use arc_citadel::entity::thoughts::{Thought, Valence, CauseType};
use arc_citadel::entity::needs::NeedType;

fn update_needs(world: &mut World) {
    let dt = 1.0;
    for i in 0..world.humans.ids.len() {
        if !world.humans.alive[i] { continue; }
        let is_active = world.humans.task_queues[i].current().is_some();
        world.humans.needs[i].decay(dt, is_active);
    }
}

fn run_perception(world: &World) -> Vec<Perception> {
    let mut grid = SparseHashGrid::new(10.0);
    let positions = &world.humans.positions;
    let ids = &world.humans.ids;

    grid.rebuild(ids.iter().cloned().zip(positions.iter().cloned()));

    let id_to_idx: ahash::AHashMap<_, _> = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    ids.iter()
        .enumerate()
        .map(|(i, &observer_id)| {
            let observer_pos = positions[i];
            let nearby: Vec<_> = grid
                .query_neighbors(observer_pos)
                .filter(|&e| e != observer_id)
                .collect();

            let perceived_entities: Vec<_> = nearby
                .iter()
                .filter_map(|&entity| {
                    let entity_idx = *id_to_idx.get(&entity)?;
                    let entity_pos = positions[entity_idx];
                    let distance = observer_pos.distance(&entity_pos);
                    if distance <= 50.0 {
                        Some(PerceivedEntity {
                            entity,
                            distance,
                            relationship: RelationshipType::Unknown,
                            disposition: arc_citadel::entity::social::Disposition::Unknown,
                            threat_level: 0.0,
                            notable_features: vec![],
                        })
                    } else {
                        None
                    }
                })
                .collect();

            Perception {
                observer: observer_id,
                perceived_entities,
                perceived_objects: vec![],
                perceived_events: vec![],
                nearest_food_zone: None,
            }
        })
        .collect()
}

fn generate_thoughts(world: &mut World, perceptions: &[Perception]) {
    // Build O(1) lookup map once
    let id_to_idx: ahash::AHashMap<_, _> = world.humans.ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    for perception in perceptions {
        let Some(&idx) = id_to_idx.get(&perception.observer) else { continue };
        let values = &world.humans.values[idx];

        for perceived in &perception.perceived_entities {
            if perceived.threat_level > 0.5 {
                let thought = Thought::new(
                    Valence::Negative,
                    perceived.threat_level,
                    if values.safety > 0.5 { "fear" } else { "concern" },
                    "threatening entity nearby",
                    CauseType::Entity,
                    world.current_tick,
                );
                world.humans.thoughts[idx].add(thought);
                world.humans.needs[idx].safety =
                    (world.humans.needs[idx].safety + perceived.threat_level * 0.3).min(1.0);
            }
            if perceived.relationship == RelationshipType::Ally {
                world.humans.needs[idx].satisfy(NeedType::Social, 0.1);
            }
        }
    }
}

fn decay_thoughts(world: &mut World) {
    for i in 0..world.humans.ids.len() {
        if !world.humans.alive[i] { continue; }
        world.humans.thoughts[i].decay_all();
    }
}

fn select_actions(world: &mut World) {
    use arc_citadel::simulation::perception::find_nearest_food_zone;

    let current_tick = world.current_tick;
    for i in 0..world.humans.ids.len() {
        if !world.humans.alive[i] { continue; }
        if world.humans.task_queues[i].current().is_some() { continue; }

        let pos = world.humans.positions[i];
        // Check if entity is AT a food zone
        let food_available = world.food_zones.iter()
            .any(|zone| zone.contains(pos));
        // Find nearest food zone within perception range
        let nearest_food_zone = find_nearest_food_zone(pos, 50.0, &world.food_zones);

        let ctx = SelectionContext {
            body: &world.humans.body_states[i],
            needs: &world.humans.needs[i],
            thoughts: &world.humans.thoughts[i],
            values: &world.humans.values[i],
            has_current_task: false,
            threat_nearby: world.humans.needs[i].safety > 0.5,
            food_available,
            safe_location: world.humans.needs[i].safety < 0.3,
            entity_nearby: true,
            current_tick,
            nearest_food_zone,
            perceived_dispositions: vec![],
        };

        if let Some(task) = select_action_human(&ctx) {
            world.humans.task_queues[i].push(task);
        }
    }
}

fn execute_tasks(world: &mut World) {
    for i in 0..world.humans.ids.len() {
        if !world.humans.alive[i] { continue; }

        let task_info = world.humans.task_queues[i].current_mut().map(|task| {
            task.progress += 0.01;
            let action = task.action;
            let duration = task.action.base_duration();
            let is_complete = duration > 0 && task.progress >= 1.0;
            (action, is_complete)
        });

        if let Some((action, is_complete)) = task_info {
            for (need, amount) in action.satisfies_needs() {
                world.humans.needs[i].satisfy(need, amount * 0.01);
            }
            if is_complete {
                world.humans.task_queues[i].complete_current();
            }
        }
    }
}
