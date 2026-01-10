use arc_citadel::core::types::Vec2;
use arc_citadel::ecs::world::World;
use arc_citadel::simulation::tick::run_simulation_tick;
use std::time::Instant;

fn main() {
    let counts = [100, 1000, 2000, 5000, 10000, 20000];

    for count in counts {
        println!("\n=== Testing {} entities ===", count);

        let mut world = World::new();

        // Spawn entities spread across world (100x100 grid)
        let spawn_start = Instant::now();
        for i in 0..count {
            world.spawn_human(format!("Entity_{}", i));
            // Spread entities across a 1000x1000 area
            let x = (i % 100) as f32 * 10.0;
            let y = (i / 100) as f32 * 10.0;
            world.humans.positions[i] = Vec2::new(x, y);
        }
        let spawn_time = spawn_start.elapsed();
        println!("Spawn time: {:?}", spawn_time);

        // Run 100 ticks
        let tick_start = Instant::now();
        for _ in 0..100 {
            run_simulation_tick(&mut world);
        }
        let tick_time = tick_start.elapsed();

        println!("100 ticks: {:?}", tick_time);
        println!("Avg tick: {:?}", tick_time / 100);
        println!("Ticks/sec: {:.1}", 100.0 / tick_time.as_secs_f64());
    }
}
