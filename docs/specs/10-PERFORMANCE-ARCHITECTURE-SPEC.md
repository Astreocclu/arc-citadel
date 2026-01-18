# 10-PERFORMANCE-ARCHITECTURE-SPEC
> Optimization strategies, profiling points, and performance targets

## Overview

Arc Citadel targets real-time simulation with 1000+ entities while maintaining responsive gameplay. This specification details the performance architecture, optimization strategies, and measurement targets.

---

## Performance Targets

### Primary Metrics

| Metric | Target | Rationale |
|--------|--------|-----------|
| **Tick Time** | < 16ms | 60 TPS capability |
| **Entity Count** | 1000+ | Settlement-scale simulation |
| **Spatial Query** | < 1ms | Radius queries for perception |
| **Memory/Entity** | < 1KB | SoA efficiency |
| **LLM Latency** | < 2s | Player command response |
| **Save/Load** | < 5s | Session transition |

### Secondary Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| Frame render | < 8ms | UI responsiveness |
| Pathfinding | < 5ms | Per-entity path query |
| Combat resolution | < 1ms | Per-attack |
| Thought generation | < 0.5ms | Per-entity per-tick |
| Action selection | < 1ms | Per-entity decision |

---

## Structure of Arrays (SoA)

### Why SoA?

Traditional OOP stores entities as objects:

```rust
// ❌ Array of Structures (AoS) - cache inefficient
struct Entity {
    position: Vec2,
    velocity: Vec2,
    needs: Needs,
    thoughts: ThoughtBuffer,
    // ... many fields
}
let entities: Vec<Entity> = vec![...];

// Iterating positions loads entire entities into cache
for entity in &entities {
    process_position(&entity.position);  // Cache line contains irrelevant data
}
```

SoA stores components in parallel arrays:

```rust
// ✅ Structure of Arrays (SoA) - cache efficient
struct HumanArchetype {
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    needs: Vec<Needs>,
    thoughts: Vec<ThoughtBuffer>,
    // ... parallel arrays
}

// Iterating positions keeps cache hot
for pos in &archetype.positions {
    process_position(pos);  // Cache line contains adjacent positions
}
```

### Cache Efficiency Analysis

```
Cache Line Size: 64 bytes
Vec2: 8 bytes (2 × f32)

AoS iteration of 1000 positions:
  - Entity size: ~512 bytes (estimated)
  - Entities per cache line: 0.125
  - Cache lines needed: 8000
  - Cache misses (assuming 32KB L1): ~8000

SoA iteration of 1000 positions:
  - Positions per cache line: 8
  - Cache lines needed: 125
  - Cache misses: ~125

Speedup factor: ~64x for position-only iteration
```

### SoA Implementation

```rust
pub struct HumanArchetype {
    // Identity
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,

    // Physics (hot path - often iterated together)
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,

    // Body state
    pub body_states: Vec<BodyState>,

    // Mind (cognitive processing)
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<HumanValues>,
    pub task_queues: Vec<TaskQueue>,

    // Lifecycle
    pub alive: Vec<bool>,

    // Index for O(1) lookup
    id_to_index: HashMap<EntityId, usize>,
}

impl HumanArchetype {
    /// O(1) lookup by entity ID
    pub fn index_of(&self, id: EntityId) -> Option<usize> {
        self.id_to_index.get(&id).copied()
    }

    /// Iterate indices of living entities
    pub fn iter_alive(&self) -> impl Iterator<Item = usize> + '_ {
        self.alive.iter()
            .enumerate()
            .filter(|(_, &alive)| alive)
            .map(|(idx, _)| idx)
    }

    /// Batch position update (SIMD-friendly)
    pub fn update_positions(&mut self, dt: f32) {
        for idx in self.iter_alive() {
            self.positions[idx].x += self.velocities[idx].x * dt;
            self.positions[idx].y += self.velocities[idx].y * dt;
        }
    }
}
```

---

## Sparse Hash Grid

### O(1) Spatial Queries

```rust
pub struct SparseHashGrid {
    cells: HashMap<(i32, i32), Vec<EntityId>>,
    cell_size: f32,
}

impl SparseHashGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cells: HashMap::new(),
            cell_size,
        }
    }

    /// Convert position to cell key
    #[inline]
    fn cell_key(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    /// Insert entity at position
    pub fn insert(&mut self, id: EntityId, pos: Vec2) {
        let key = self.cell_key(pos);
        self.cells.entry(key).or_default().push(id);
    }

    /// Query entities within radius
    pub fn query_radius(&self, center: Vec2, radius: f32) -> Vec<EntityId> {
        let cell_radius = (radius / self.cell_size).ceil() as i32;
        let center_cell = self.cell_key(center);
        let radius_sq = radius * radius;

        let mut results = Vec::new();

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let key = (center_cell.0 + dx, center_cell.1 + dy);
                if let Some(entities) = self.cells.get(&key) {
                    results.extend(entities.iter().copied());
                }
            }
        }

        results
    }

    /// Clear and rebuild from positions
    pub fn rebuild(&mut self, ids: &[EntityId], positions: &[Vec2], alive: &[bool]) {
        self.cells.clear();

        for (idx, (&id, &pos)) in ids.iter().zip(positions.iter()).enumerate() {
            if alive[idx] {
                self.insert(id, pos);
            }
        }
    }
}
```

### Grid Size Optimization

| Cell Size | Entities/Cell (1000 entities, 500x500 world) | Queries Checked |
|-----------|---------------------------------------------|-----------------|
| 5.0 | ~0.1 | Few entities, many cells checked |
| 10.0 | ~0.4 | Balanced |
| 20.0 | ~1.6 | Fewer cells, more entities per cell |
| 50.0 | ~10 | Many entities per cell |

**Optimal**: 10.0 for typical perception range of 20 units (queries check 25 cells max).

---

## Tick Optimization

### Phased Execution

```rust
pub fn tick(world: &mut World) {
    // Phase 1: Needs decay (vectorizable)
    // ~0.5ms for 1000 entities
    update_needs_batch(&mut world.humans.needs);

    // Phase 2: Build spatial index
    // ~1ms for 1000 entities
    world.spatial_index.rebuild(
        &world.humans.ids,
        &world.humans.positions,
        &world.humans.alive,
    );

    // Phase 3: Perception (O(n × k) where k = average neighbors)
    // ~3ms for 1000 entities with ~10 neighbors each
    let perceptions = run_perception_batch(world);

    // Phase 4: Thought generation
    // ~2ms for 1000 entities
    generate_thoughts_batch(world, &perceptions);

    // Phase 5: Thought decay
    // ~0.5ms for 1000 entities
    decay_thoughts_batch(&mut world.humans.thoughts);

    // Phase 6: Action selection (only idle entities)
    // ~1ms for ~100 idle entities
    select_actions_idle(world);

    // Phase 7: Task execution
    // ~2ms for 1000 entities
    execute_tasks_batch(world);

    // Phase 8: Production tick
    // ~0.5ms for ~50 buildings
    run_production(world);

    // Total: ~10.5ms (budget: 16ms)
    world.current_tick = world.current_tick.next();
}
```

### Batch Operations

```rust
/// Batch needs decay using SIMD-friendly patterns
fn update_needs_batch(needs: &mut [Needs]) {
    for need in needs.iter_mut() {
        // Compiler can auto-vectorize this
        need.hunger = (need.hunger + 0.001).min(1.0);
        need.thirst = (need.thirst + 0.002).min(1.0);
        need.rest = (need.rest + 0.0005).min(1.0);
        need.safety = (need.safety - 0.0001).max(0.0);
        need.social = (need.social + 0.001).min(1.0);
        need.purpose = (need.purpose + 0.0005).min(1.0);
    }
}

/// Batch thought decay
fn decay_thoughts_batch(thoughts: &mut [ThoughtBuffer]) {
    for buffer in thoughts.iter_mut() {
        buffer.decay_all(0.05);
    }
}
```

---

## Lazy Evaluation

### Perception on Demand

Not every entity needs full perception every tick:

```rust
pub struct PerceptionCache {
    last_update: HashMap<EntityId, Tick>,
    cached_perceptions: HashMap<EntityId, Vec<Perception>>,
    update_interval: u32,  // Ticks between updates
}

impl PerceptionCache {
    /// Get perceptions, updating if stale
    pub fn get_perceptions(
        &mut self,
        entity: EntityId,
        current_tick: Tick,
        world: &World,
    ) -> &[Perception] {
        let needs_update = self.last_update
            .get(&entity)
            .map(|&t| current_tick.0 - t.0 >= self.update_interval)
            .unwrap_or(true);

        if needs_update {
            let perceptions = compute_perceptions(entity, world);
            self.cached_perceptions.insert(entity, perceptions);
            self.last_update.insert(entity, current_tick);
        }

        &self.cached_perceptions[&entity]
    }
}
```

### Priority-Based Updates

```rust
/// Entities with urgent needs get updated more frequently
fn perception_priority(entity_idx: usize, needs: &Needs) -> u32 {
    let urgency = needs.max_urgency();

    match urgency {
        u if u > 0.8 => 1,   // Every tick
        u if u > 0.5 => 3,   // Every 3 ticks
        u if u > 0.2 => 10,  // Every 10 ticks
        _ => 30,             // Every 30 ticks
    }
}
```

---

## Memory Management

### Arena Allocation

```rust
/// Pre-allocated buffers for temporary data
pub struct TickArena {
    perception_buffer: Vec<Perception>,
    thought_buffer: Vec<Thought>,
    action_buffer: Vec<ActionCandidate>,
    entity_buffer: Vec<EntityId>,
}

impl TickArena {
    pub fn new(max_entities: usize) -> Self {
        Self {
            perception_buffer: Vec::with_capacity(max_entities * 10),
            thought_buffer: Vec::with_capacity(max_entities * 20),
            action_buffer: Vec::with_capacity(max_entities * 5),
            entity_buffer: Vec::with_capacity(max_entities),
        }
    }

    pub fn clear(&mut self) {
        self.perception_buffer.clear();
        self.thought_buffer.clear();
        self.action_buffer.clear();
        self.entity_buffer.clear();
    }
}
```

### Object Pooling

```rust
/// Pool for thought objects to avoid allocation
pub struct ThoughtPool {
    free: Vec<Box<Thought>>,
}

impl ThoughtPool {
    pub fn acquire(&mut self) -> Box<Thought> {
        self.free.pop().unwrap_or_else(|| Box::new(Thought::default()))
    }

    pub fn release(&mut self, thought: Box<Thought>) {
        self.free.push(thought);
    }
}
```

---

## Async LLM Integration

### Non-Blocking Commands

```rust
pub struct LlmService {
    client: reqwest::Client,
    pending: HashMap<CommandId, oneshot::Receiver<ParsedIntent>>,
}

impl LlmService {
    /// Submit command for parsing, returns immediately
    pub fn submit_command(&mut self, input: String) -> CommandId {
        let id = CommandId::new();
        let (tx, rx) = oneshot::channel();

        let client = self.client.clone();
        tokio::spawn(async move {
            let result = parse_with_llm(&client, &input).await;
            let _ = tx.send(result);
        });

        self.pending.insert(id, rx);
        id
    }

    /// Check if command is ready (non-blocking)
    pub fn poll_command(&mut self, id: CommandId) -> Option<ParsedIntent> {
        if let Some(rx) = self.pending.get_mut(&id) {
            match rx.try_recv() {
                Ok(intent) => {
                    self.pending.remove(&id);
                    Some(intent)
                }
                Err(oneshot::error::TryRecvError::Empty) => None,
                Err(oneshot::error::TryRecvError::Closed) => {
                    self.pending.remove(&id);
                    None
                }
            }
        } else {
            None
        }
    }
}
```

### Timeout Handling

```rust
pub async fn parse_with_timeout(
    client: &LlmClient,
    input: &str,
    timeout_ms: u64,
) -> ParsedIntent {
    match tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        client.parse(input),
    ).await {
        Ok(Ok(intent)) => intent,
        Ok(Err(e)) => {
            log::warn!("LLM error: {e}, using fallback");
            FallbackParser::parse(input)
        }
        Err(_) => {
            log::warn!("LLM timeout, using fallback");
            FallbackParser::parse(input)
        }
    }
}
```

---

## Profiling Infrastructure

### Instrumentation Points

```rust
#[derive(Default)]
pub struct TickMetrics {
    pub needs_update_us: u64,
    pub spatial_rebuild_us: u64,
    pub perception_us: u64,
    pub thought_gen_us: u64,
    pub thought_decay_us: u64,
    pub action_select_us: u64,
    pub task_execute_us: u64,
    pub production_us: u64,
    pub total_us: u64,
}

macro_rules! timed {
    ($metrics:expr, $field:ident, $expr:expr) => {{
        let start = std::time::Instant::now();
        let result = $expr;
        $metrics.$field = start.elapsed().as_micros() as u64;
        result
    }};
}

pub fn tick_instrumented(world: &mut World) -> TickMetrics {
    let mut metrics = TickMetrics::default();
    let total_start = std::time::Instant::now();

    timed!(metrics, needs_update_us, update_needs_batch(&mut world.humans.needs));
    timed!(metrics, spatial_rebuild_us, world.spatial_index.rebuild(...));
    timed!(metrics, perception_us, run_perception_batch(world));
    // ... other phases

    metrics.total_us = total_start.elapsed().as_micros() as u64;
    metrics
}
```

### Performance Dashboard

```rust
pub struct PerformanceMonitor {
    samples: VecDeque<TickMetrics>,
    max_samples: usize,
}

impl PerformanceMonitor {
    pub fn record(&mut self, metrics: TickMetrics) {
        self.samples.push_back(metrics);
        while self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
    }

    pub fn average(&self) -> TickMetrics {
        let n = self.samples.len() as u64;
        if n == 0 { return TickMetrics::default(); }

        TickMetrics {
            needs_update_us: self.samples.iter().map(|m| m.needs_update_us).sum::<u64>() / n,
            spatial_rebuild_us: self.samples.iter().map(|m| m.spatial_rebuild_us).sum::<u64>() / n,
            // ... average all fields
            total_us: self.samples.iter().map(|m| m.total_us).sum::<u64>() / n,
        }
    }

    pub fn worst_case(&self) -> TickMetrics {
        // Return the tick with highest total_us
        self.samples.iter()
            .max_by_key(|m| m.total_us)
            .cloned()
            .unwrap_or_default()
    }
}
```

---

## Scaling Strategies

### Entity Count Scaling

| Entities | Strategy |
|----------|----------|
| < 500 | Full evaluation every tick |
| 500-2000 | Priority-based lazy perception |
| 2000-5000 | Region-based simulation (active region only) |
| 5000+ | Background agents, player-visible only real-time |

### Region-Based Simulation

```rust
pub struct SimulationRegion {
    center: Vec2,
    radius: f32,
    entities: HashSet<EntityId>,
}

impl World {
    /// Only simulate entities in active region at full fidelity
    pub fn tick_with_regions(&mut self, active_region: &SimulationRegion) {
        // Full simulation for active region
        for &id in &active_region.entities {
            self.tick_entity_full(id);
        }

        // Simplified simulation for background (every 10 ticks)
        if self.current_tick.0 % 10 == 0 {
            for id in self.all_entities() {
                if !active_region.entities.contains(&id) {
                    self.tick_entity_simplified(id);
                }
            }
        }
    }
}
```

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [02-IMPLEMENTATION-ARCHITECTURE](02-IMPLEMENTATION-ARCHITECTURE.md) | Core architecture |
| [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) | Entity systems to optimize |
| [09-GAP-ANALYSIS](09-GAP-ANALYSIS.md) | Implementation priorities |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
