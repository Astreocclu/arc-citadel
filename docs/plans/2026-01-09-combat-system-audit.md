# Combat System Audit - Pre-GUI Gaps

> Systematic audit of combat mechanics to ensure core systems are solid before GUI implementation.

## Status: COMPLETE

| Section | Status |
|---------|--------|
| 1. Ranged Combat | Gaps identified - HIGH priority |
| 2. Melee Combat Edge Cases | Gaps identified - LOW effort fixes |
| 3. Movement/Pathfinding | Defer ZoC for MVP |
| 4. Order/Courier System | Gaps identified - LOW effort fixes |
| 5. AI Decision Edge Cases | Defer for MVP |
| 6. Morale/Routing Edge Cases | Gaps identified - LOW effort fixes |
| 7. System Integrations | Defer for MVP |

---

## 1. Ranged Combat Completion

**Current State:** Unit types have `is_ranged: bool` but no projectile mechanics exist.

### Gaps to Implement

| Gap | Priority | Description |
|-----|----------|-------------|
| Target selection | High | Formation-scale targeting: Officers, Horses, Center mass, Flanks |
| Range/accuracy falloff | High | Hit probability decreases with distance |
| Volley resolution | High | Archer count x accuracy x target modifier → casualties + stress |
| Cover mechanics | High | Terrain/shields reduce incoming damage |
| Ammunition tracking | Medium | Quiver depletion, resupply from baggage train |
| Arc vs direct fire | Medium | Longbows arc over obstacles, crossbows need LOS |
| Suppression | Medium | Incoming fire causes stress even on misses |

### Target Selection Trade-offs

| Target | Accuracy Mod | Effect on Hit |
|--------|--------------|---------------|
| Center mass | 100% | Standard casualties |
| Officers | 40% | Morale cascade on kill |
| Horses (cavalry) | 70% | Dismount, mobility kill |
| Flanks | 80% | Bypasses front-rank shields |

### Volley Resolution (Individual Arrow Simulation)

**Not aggregate - iterate each archer individually:**

```rust
for archer in archers {
    // 1. Skill-based hit roll per arrow
    let hit_roll = archer.skill.roll_accuracy(range, target_cover);

    // 2. Individual target selection via spatial query
    let target = spatial_index.query_nearest_enemy(archer.position, target_priority);

    // 3. Resolve hit/miss/kill
    let outcome = resolve_arrow_hit(hit_roll, target);

    // 4. Attribute outcome to this archer
    archer.combat_history.record(outcome);
}
```

**Requirements:**

| Requirement | Details |
|-------------|---------|
| Per-arrow resolution | Each archer rolls individually, no aggregate math |
| Skill-based hit roll | Accuracy influenced by archer skill level |
| Individual target selection | Each arrow picks a target (not "the formation") |
| Attributed outcomes | Track who killed whom |
| Combat history component | Per-entity storage: kills, hits, misses |
| Spatial partitioning | Quadtree query for target selection, not linear scan |

**Combat History Component:**

```rust
pub struct CombatHistory {
    pub kills: Vec<EntityId>,      // Who this entity killed
    pub hits: Vec<EntityId>,       // Who this entity wounded
    pub misses: u32,               // Shot count that missed
    pub damage_dealt: f32,         // Total damage inflicted
    pub volleys_participated: u32, // Engagement count
}
```

**Spatial Query for Target Selection:**

```rust
// O(log n) target lookup, not O(n) linear scan
let candidates = quadtree.query_radius(archer.position, max_range);
let target = candidates
    .filter(|t| matches_target_priority(t, priority))
    .min_by_key(|t| distance(archer.position, t.position));
```

---

## 2. Melee Combat Edge Cases

**Current State:** 42 tests passing. Core mechanics solid.

### Verified OK

| Check | Status | Details |
|-------|--------|---------|
| Fatigue cap | OK | Clamped at 1.0 in `state.rs:54` |
| Stress accumulation | OK | No cap - keeps accumulating past break threshold |
| Penetration math | OK | Pure categorical lookup, no negative values possible |
| Mutual hits | OK | `resolve_exchange` handles both combatants being hit |

### Gaps to Fix

| Gap | Priority | Details |
|-----|----------|---------|
| No path INTO Broken stance | High | Stance transitions never produce `Broken`. TookHit→Recovering, Knockdown→Recovering. Nothing leads to Broken. |
| Wound storage missing | High | Combat produces `Wound` structs but nothing collects them. Where do accumulated wounds live? |
| Wound effects not applied | Medium | Wounds have `mobility_impact` and `grip_impact` flags, but nothing reads these. |

### Proposed Broken Triggers

Broken = permanently out of this fight. Triggers:

| Trigger | Condition |
|---------|-----------|
| Critical head wound | Instant incapacitation |
| Critical torso wound | Instant incapacitation |
| Morale break | Stress exceeds threshold → fled |
| Accumulated wounds | 3+ Serious wounds → too injured to continue |

### Implementation Notes

Add transitions in `stance.rs`:
```rust
// New triggers needed
CriticalWoundHead => Broken,
CriticalWoundTorso => Broken,
MoraleBreak => Broken,
WoundThresholdExceeded => Broken,
```

Add wound storage to entity archetype or CombatState:
```rust
pub wounds: Vec<Wound>,
```

---

## 3. Movement/Pathfinding Edge Cases

**Current State:** A* pathfinding works. 100m hex scale.

### Verified OK

| Check | Status | Details |
|-------|--------|---------|
| Wait conditions | OK | `is_waiting_with_context` fully implements all conditions |
| Impassable terrain | OK | Pathfinding returns None when blocked |
| Cavalry restrictions | OK | Forest/building blocks cavalry |
| Fatigue slowdown | OK | High fatigue reduces speed but doesn't halt movement |

### Gaps to Fix

| Gap | Priority | Details |
|-----|----------|---------|
| No congestion modeling | Medium | Multiple units in one hex should slow movement/deployment |
| No zone of control | Medium | Moving near enemies should be contested/risky |
| Path not recalculated | Low | Stale paths when terrain changes |
| Routing units stuck | Low | Blocked retreat direction - rare edge case |

### Congestion Model (Proposed)

At 100m hex scale, multiple units can share a hex. Effects:

```rust
// Movement cost increases with congestion
let congestion_multiplier = 1.0 + (0.2 * units_in_hex.saturating_sub(1) as f32);
let effective_cost = base_cost * congestion_multiplier;

// Combat effectiveness decreases when overcrowded (>3 units)
let overcrowd_penalty = (units_in_hex.saturating_sub(3) as f32) * 0.1;
```

### Zone of Control Options

**DECISION REQUIRED: Choose ZoC approach**

#### Option A: Movement Tax (75/100)

Adjacent enemy hexes increase movement cost.

| Pros | Cons |
|------|------|
| Simple to implement | Doesn't feel dangerous |
| Predictable | No tactical decisions during move |
| Low computation | Can be "paid through" with fast units |

```rust
let zoc_cost = if adjacent_to_enemy { base_cost * 1.5 } else { base_cost };
```

#### Option B: Disengage Check (85/100)

Must pass morale check to leave enemy ZoC.

| Pros | Cons |
|------|------|
| Creates tension | More complex |
| Rewards disciplined troops | Can feel random |
| Historical accuracy | Slows gameplay |

```rust
fn attempt_disengage(unit: &BattleUnit) -> bool {
    let check = unit.morale.current_stress < unit.morale.base_threshold * 0.5;
    // Veterans disengage easier
    check || unit.skill >= SkillLevel::Veteran
}
```

#### Option C: Opportunity Fire (80/100)

Leaving ZoC triggers free ranged attack from enemy.

| Pros | Cons |
|------|------|
| Tactical depth | Only matters vs ranged enemies |
| Clear consequences | Requires ranged combat system first |
| Player agency (can choose to take the hit) | More bookkeeping |

```rust
fn leaving_zoc_triggers(unit: &BattleUnit, enemies: &[&BattleUnit]) -> Vec<OpportunityAttack> {
    enemies.iter()
        .filter(|e| e.can_opportunity_fire())
        .map(|e| OpportunityAttack::new(e.id, unit.id))
        .collect()
}
```

#### Option D: Combination (90/100)

- Movement tax (A) for passing through ZoC
- Disengage check (B) for leaving melee engagement
- Opportunity fire (C) only for ranged units with ammunition

| Pros | Cons |
|------|------|
| Most realistic | Most complex |
| Different tactical situations | More rules to learn |
| Rewards combined arms | Higher implementation cost |

---

## 4. Order/Courier System Edge Cases

**Verified OK:**

| Check | Status | Details |
|-------|--------|---------|
| Courier interception | OK | Patrol/Alert units intercept nearby couriers with probability check |
| Order application | OK | All order types implemented with tests |
| Formation orders | OK | Orders to formations apply to all units |

**Gaps Found:**

| Gap | Severity | Details |
|-----|----------|---------|
| Courier entity death not detected | Medium | `CourierStatus::Lost` exists but never triggered. If `courier_entity` dies, courier keeps flying. |
| Attack order to dead target | High | `OrderType::Attack(target_id)` uses `unwrap_or_default()` - sends unit to (0,0) if target destroyed |
| Order to non-existent unit | Low | Silently "succeeds" without doing anything. |
| No order acknowledgment | Low | Commander doesn't know if order was received/executed |

**Proposed Fixes:**

| Fix | Approach |
|-----|----------|
| Courier entity death | Each tick, check if `courier_entity` exists. If not, mark `Lost`. |
| Attack dead target | Return `success: false` with message "Target no longer exists" |
| Order acknowledgment | Optional: return courier with status report (doubles travel time) |

---

## 5. AI Decision Edge Cases

**Verified OK:**

| Check | Status | Details |
|-------|--------|---------|
| Fog of war filtering | OK | `DecisionContext` correctly filters visibility |
| Strength ratio | OK | Returns `f32::MAX` when no visible enemies |
| Re-evaluation interval | OK | Prevents order spam |
| Mistake chance | OK | Personality-driven random failures |

**Gaps Found:**

| Gap | Severity | Details |
|-----|----------|---------|
| Commander entity death ignored | Medium | `AiCommander` is not tied to an EntityId. If commander unit dies, AI keeps working omnisciently. |
| No visible enemies = idle | Low | Units do nothing when `visible_enemies.is_empty()`. Should they scout? |
| Retreat to overrun HQ | Medium | `generate_retreat_orders` sends units to `hq_position` even if enemies hold it. |
| Surrounded but can't see | Medium | If enemies surround but stay out of sight, strength_ratio = MAX → no retreat decision. |
| No courier availability check | Low | AI issues orders without checking `available_couriers()`. Orders may never be sent. |

**Proposed Fixes:**

| Fix | Approach |
|-----|----------|
| Commander death | Tie AI to commander EntityId. If dead: degrade to simpler "last orders" mode or pass command to subordinate. |
| No visible enemies | Add scouting behavior: send light cavalry/skirmishers to explore. |
| HQ overrun | Before retreating to HQ, check if enemy holds it. Pick alternate rally point if compromised. |
| Surrounded blind | Add "suspected enemy" heuristic: if couriers intercepted or units attacked from fog, increase caution. |

---

## 6. Morale/Routing Edge Cases

**Verified OK:** Stress cap (0.0-2.0), contagion stress, rally conditions, officer death stress.

**Gaps:**

| Gap | Severity | Fix |
|-----|----------|-----|
| Rallying → Formed never happens | High | Add tick counter, transition after N ticks without stress |
| No capture mechanic | Medium | Surrounded routing units with no escape → Captured/Destroyed |
| Mass rout cascade | Low | Optional: cap contagion sources at 3 |

---

## 7. System Integrations

**Not blocking MVP.** These are enhancements:

| Integration | Status | Notes |
|-------------|--------|-------|
| Terrain → Combat | Partial | Defensive bonuses exist, cover not wired to ranged |
| Terrain → Movement | Done | Pathfinding respects terrain costs |
| Visibility → Targeting | Done | AI respects fog of war |
| Weather | Not started | Future enhancement |
| Time of day | Not started | Future enhancement |

---

## MVP Priority Gaps

**Must fix for MVP:**

| # | Gap | Location | Effort |
|---|-----|----------|--------|
| 1 | Ranged combat (individual arrows, spatial query) | New module | High |
| 2 | No path INTO Broken stance | `combat/stance.rs` | Low |
| 3 | Rallying → Formed never happens | `battle/execution.rs` | Low |
| 4 | Attack order to dead target sends to (0,0) | `battle/orders.rs` | Low |

**Can defer:**

- ZoC (movement tax is fine for MVP)
- Congestion modeling
- Courier entity death detection
- AI commander death
- Capture mechanic
- Weather/time of day

---

## Decisions Made

| Decision | Choice |
|----------|--------|
| Zone of Control | Defer - use simple movement tax if needed |
| Ranged targeting | Formation-scale only (Officers, Horses, Flanks, Center mass) |
| Arrow resolution | Individual per-archer, spatial query, combat history tracking |
