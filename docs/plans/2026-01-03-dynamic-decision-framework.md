# Dynamic Decision Framework

**Goal:** Replace static thresholds (e.g., "1.2x military = declare war") with dynamic calculations influenced by personality traits and situational state.

**Scope:** ~100-150 lines of changes to existing files. No new modules.

**Key insight:** Decisions become `threshold = base + personality_mod + state_mod` instead of `threshold = 0.8`.

---

## Task 1: Add Personality and State Fields to Polity Structs

**File:** `src/aggregate/polity.rs`

**What:** Add `boldness`, `caution`, `war_exhaustion`, and `morale` to each species state struct.

### 1.1 Add to HumanState

Find and modify:
```rust
pub struct HumanState {
    pub expansion_pressure: f32,
    pub internal_cohesion: f32,
    pub reputation: f32,
    pub piety: f32,
    pub factions: Vec<u32>,
}
```

Change to:
```rust
pub struct HumanState {
    pub expansion_pressure: f32,
    pub internal_cohesion: f32,
    pub reputation: f32,
    pub piety: f32,
    pub factions: Vec<u32>,
    // Personality (set at generation, doesn't change)
    pub boldness: f32,      // 0.0-1.0: willingness to take risks
    pub caution: f32,       // 0.0-1.0: aversion to risk
    // Dynamic state (changes based on events)
    pub war_exhaustion: f32, // 0.0-1.0: accumulated war weariness
    pub morale: f32,         // -1.0 to 1.0: recent successes/failures
}
```

### 1.2 Add to DwarfState

Find and modify:
```rust
pub struct DwarfState {
    pub grudge_ledger: HashMap<u32, Vec<Grudge>>,
    pub oaths: Vec<Oath>,
    pub ancestral_sites: Vec<u32>,
    pub craft_focus: CraftType,
}
```

Change to:
```rust
pub struct DwarfState {
    pub grudge_ledger: HashMap<u32, Vec<Grudge>>,
    pub oaths: Vec<Oath>,
    pub ancestral_sites: Vec<u32>,
    pub craft_focus: CraftType,
    // Personality
    pub boldness: f32,
    pub caution: f32,
    // Dynamic state
    pub war_exhaustion: f32,
    pub morale: f32,
}
```

### 1.3 Add to ElfState

Find and modify:
```rust
pub struct ElfState {
    pub memory: Vec<MemoryEntry>,
    pub grief_level: f32,
    pub pending_decisions: Vec<PendingDecision>,
    pub core_territory: HashSet<u32>,
    pub pattern_assessment: f32,
}
```

Change to:
```rust
pub struct ElfState {
    pub memory: Vec<MemoryEntry>,
    pub grief_level: f32,
    pub pending_decisions: Vec<PendingDecision>,
    pub core_territory: HashSet<u32>,
    pub pattern_assessment: f32,
    // Personality
    pub boldness: f32,
    pub caution: f32,
    // Dynamic state
    pub war_exhaustion: f32,
    pub morale: f32,
}
```

**Verification:**
```bash
cargo check 2>&1 | head -50
# Expect: errors about missing fields in generation.rs (we'll fix next)
```

---

## Task 2: Generate Personality at Polity Creation

**File:** `src/aggregate/systems/generation.rs`

**What:** Initialize the new personality/state fields when polities are created.

### 2.1 Update HumanState generation

Find in `generate_species_polities`:
```rust
Species::Human => SpeciesState::Human(HumanState {
    expansion_pressure: rng.gen_range(0.3..0.7),
    internal_cohesion: rng.gen_range(0.5..0.9),
    reputation: rng.gen_range(0.4..0.8),
    piety: rng.gen_range(0.2..0.8),
    factions: Vec::new(),
}),
```

Change to:
```rust
Species::Human => SpeciesState::Human(HumanState {
    expansion_pressure: rng.gen_range(0.3..0.7),
    internal_cohesion: rng.gen_range(0.5..0.9),
    reputation: rng.gen_range(0.4..0.8),
    piety: rng.gen_range(0.2..0.8),
    factions: Vec::new(),
    // Humans: balanced personality distribution
    boldness: rng.gen_range(0.3..0.7),
    caution: rng.gen_range(0.3..0.7),
    war_exhaustion: 0.0,
    morale: rng.gen_range(-0.1..0.1),
}),
```

### 2.2 Update DwarfState generation

Find:
```rust
Species::Dwarf => SpeciesState::Dwarf(DwarfState {
    grudge_ledger: HashMap::new(),
    oaths: Vec::new(),
    ancestral_sites: vec![capital_id],
    craft_focus: CraftType::Stone,
}),
```

Change to:
```rust
Species::Dwarf => SpeciesState::Dwarf(DwarfState {
    grudge_ledger: HashMap::new(),
    oaths: Vec::new(),
    ancestral_sites: vec![capital_id],
    craft_focus: CraftType::Stone,
    // Dwarves: cautious but bold when honor demands
    boldness: rng.gen_range(0.4..0.8),
    caution: rng.gen_range(0.5..0.9),
    war_exhaustion: 0.0,
    morale: rng.gen_range(0.0..0.2),
}),
```

### 2.3 Update ElfState generation

Find:
```rust
Species::Elf => SpeciesState::Elf(ElfState {
    memory: Vec::new(),
    grief_level: rng.gen_range(0.0..0.2),
    pending_decisions: Vec::new(),
    core_territory: territory.clone(),
    pattern_assessment: rng.gen_range(0.5..0.8),
}),
```

Change to:
```rust
Species::Elf => SpeciesState::Elf(ElfState {
    memory: Vec::new(),
    grief_level: rng.gen_range(0.0..0.2),
    pending_decisions: Vec::new(),
    core_territory: territory.clone(),
    pattern_assessment: rng.gen_range(0.5..0.8),
    // Elves: very cautious, low boldness
    boldness: rng.gen_range(0.1..0.4),
    caution: rng.gen_range(0.6..0.9),
    war_exhaustion: 0.0,
    morale: rng.gen_range(-0.2..0.1),
}),
```

**Verification:**
```bash
cargo check 2>&1 | head -20
# Expect: clean compile or only warnings
```

---

## Task 3: Create Dynamic Threshold Helper

**File:** `src/aggregate/polity.rs`

**What:** Add a helper method to calculate modified thresholds.

### 3.1 Add threshold calculation impl

Add after the Polity struct definition:
```rust
impl Polity {
    /// Calculate a dynamic threshold based on personality and state.
    /// Returns a modifier to apply to base thresholds.
    /// Positive = more cautious, negative = more aggressive
    pub fn decision_modifier(&self) -> f32 {
        let (boldness, caution, exhaustion, morale) = match &self.species_state {
            SpeciesState::Human(s) => (s.boldness, s.caution, s.war_exhaustion, s.morale),
            SpeciesState::Dwarf(s) => (s.boldness, s.caution, s.war_exhaustion, s.morale),
            SpeciesState::Elf(s) => (s.boldness, s.caution, s.war_exhaustion, s.morale),
            _ => return 0.0, // Other species use base thresholds
        };

        // Personality influence: cautious polities need better odds
        let personality_mod = (caution - boldness) * 0.3;

        // State influence: exhausted/demoralized polities are more cautious
        let exhaustion_mod = exhaustion * 0.2;
        let morale_mod = -morale * 0.15; // High morale = lower threshold (more aggressive)

        personality_mod + exhaustion_mod + morale_mod
    }
}
```

**Verification:**
```bash
cargo check 2>&1 | head -20
# Expect: clean compile
```

---

## Task 4: Apply Dynamic Thresholds in Human Behavior

**File:** `src/aggregate/species/human.rs`

**What:** Replace static thresholds with dynamic calculations.

### 4.1 Update war declaration logic

Find in `should_declare_war`:
```rust
fn should_declare_war(polity: &Polity, target: u32, world: &AggregateWorld) -> bool {
    // Already at war with them?
    if let Some(rel) = polity.relations.get(&target) {
        if rel.at_war || rel.alliance {
            return false;
        }
    }

    // Compare strength - willing to fight even when slightly weaker
    if let Some(target_polity) = world.get_polity(target) {
        polity.military_strength > target_polity.military_strength * 0.8
    } else {
        false
    }
}
```

Change to:
```rust
fn should_declare_war(polity: &Polity, target: u32, world: &AggregateWorld) -> bool {
    // Already at war with them?
    if let Some(rel) = polity.relations.get(&target) {
        if rel.at_war || rel.alliance {
            return false;
        }
    }

    // Dynamic threshold based on personality and state
    let base_threshold = 0.8;
    let modifier = polity.decision_modifier();
    let threshold = (base_threshold + modifier).clamp(0.5, 1.5);

    // Compare strength with dynamic threshold
    if let Some(target_polity) = world.get_polity(target) {
        polity.military_strength > target_polity.military_strength * threshold
    } else {
        false
    }
}
```

### 4.2 Update expansion threshold

Find the expansion check in `tick`:
```rust
if expansion_pressure > EXPANSION_THRESHOLD {
```

Change to:
```rust
let dynamic_expansion_threshold = EXPANSION_THRESHOLD + polity.decision_modifier() * 0.5;
if expansion_pressure > dynamic_expansion_threshold {
```

### 4.3 Update betrayal threshold

Find:
```rust
fn should_betray_ally(polity: &Polity, _world: &AggregateWorld) -> bool {
    let state = match polity.human_state() {
        Some(s) => s,
        None => return false,
    };

    // Low cohesion + high expansion pressure = betrayal
    state.internal_cohesion < 0.5 && state.expansion_pressure > BETRAYAL_THRESHOLD
}
```

Change to:
```rust
fn should_betray_ally(polity: &Polity, _world: &AggregateWorld) -> bool {
    let state = match polity.human_state() {
        Some(s) => s,
        None => return false,
    };

    // Bold polities with low cohesion betray more easily
    let boldness_factor = state.boldness * 0.2;
    let adjusted_threshold = BETRAYAL_THRESHOLD - boldness_factor;

    state.internal_cohesion < 0.5 && state.expansion_pressure > adjusted_threshold
}
```

**Verification:**
```bash
cargo check 2>&1 | head -20
# Expect: clean compile
```

---

## Task 5: Apply Dynamic Thresholds in Dwarf Behavior

**File:** `src/aggregate/species/dwarf.rs`

**What:** Make grudge war threshold dynamic.

### 5.1 Update grudge war logic

Find in `tick`:
```rust
if total_severity > GRUDGE_WAR_THRESHOLD {
```

Change to:
```rust
// Cautious dwarves wait longer, bold dwarves act faster
let modifier = polity.decision_modifier();
let dynamic_threshold = (GRUDGE_WAR_THRESHOLD + modifier * 0.3).clamp(0.2, 0.8);
if total_severity > dynamic_threshold {
```

**Verification:**
```bash
cargo check 2>&1 | head -20
```

---

## Task 6: Apply Dynamic Thresholds in Elf Behavior

**File:** `src/aggregate/species/elf.rs`

**What:** Make grief eruption threshold dynamic.

### 6.1 Update grief response logic

Find in `tick`:
```rust
if total_grief > GRIEF_PARALYSIS_THRESHOLD {
    // Withdraw from world
    events.push(EventType::Isolation { polity: polity.id.0 });
} else if total_grief > GRIEF_ERUPTION_THRESHOLD {
```

Change to:
```rust
// Personality affects grief thresholds
let modifier = polity.decision_modifier();
let paralysis_threshold = (GRIEF_PARALYSIS_THRESHOLD - modifier * 0.1).clamp(0.8, 1.0);
let eruption_threshold = (GRIEF_ERUPTION_THRESHOLD + modifier * 0.2).clamp(0.3, 0.8);

if total_grief > paralysis_threshold {
    // Withdraw from world
    events.push(EventType::Isolation { polity: polity.id.0 });
} else if total_grief > eruption_threshold {
```

**Verification:**
```bash
cargo check 2>&1 | head -20
```

---

## Task 7: Update State After War Events

**File:** `src/aggregate/systems/warfare.rs`

**What:** Update war_exhaustion and morale when wars start/end.

### 7.1 Add state update helper

Add at the top of the file (after imports):
```rust
/// Update a polity's war exhaustion and morale after combat
fn update_war_state(polity: &mut crate::aggregate::polity::Polity, won: bool, intensity: f32) {
    match &mut polity.species_state {
        crate::aggregate::polity::SpeciesState::Human(s) => {
            s.war_exhaustion = (s.war_exhaustion + intensity * 0.1).min(1.0);
            s.morale = if won {
                (s.morale + 0.2).min(1.0)
            } else {
                (s.morale - 0.3).max(-1.0)
            };
        }
        crate::aggregate::polity::SpeciesState::Dwarf(s) => {
            s.war_exhaustion = (s.war_exhaustion + intensity * 0.08).min(1.0); // Dwarves endure
            s.morale = if won {
                (s.morale + 0.15).min(1.0)
            } else {
                (s.morale - 0.2).max(-1.0)
            };
        }
        crate::aggregate::polity::SpeciesState::Elf(s) => {
            s.war_exhaustion = (s.war_exhaustion + intensity * 0.15).min(1.0); // Elves tire of war
            s.morale = if won {
                (s.morale + 0.1).min(1.0)
            } else {
                (s.morale - 0.4).max(-1.0) // Losses hit elves hard
            };
        }
        _ => {}
    }
}
```

### 7.2 Call update after war resolution

Find the war resolution logic (where victor is determined) and add:
```rust
// After determining victor, update states
if let Some(winner_id) = victor {
    if let Some(winner) = world.get_polity_mut(winner_id) {
        update_war_state(winner, true, 0.5);
    }
    if let Some(loser) = world.get_polity_mut(loser_id) {
        update_war_state(loser, false, 0.5);
    }
}
```

### 7.3 Decay exhaustion over time

In the yearly tick (or population update), add decay:
```rust
// War exhaustion decays slowly over peacetime
if !at_war {
    match &mut polity.species_state {
        SpeciesState::Human(s) => s.war_exhaustion = (s.war_exhaustion - 0.05).max(0.0),
        SpeciesState::Dwarf(s) => s.war_exhaustion = (s.war_exhaustion - 0.03).max(0.0),
        SpeciesState::Elf(s) => s.war_exhaustion = (s.war_exhaustion - 0.02).max(0.0),
        _ => {}
    }
}
```

**Verification:**
```bash
cargo check 2>&1 | head -20
```

---

## Task 8: Final Integration Test

**Verification:**
```bash
# Full compile
cargo build 2>&1 | tail -10

# Run simulation
cargo run --example run_history 2>&1

# Expected: Simulation completes, potentially different war patterns due to personality variance
```

**Success criteria:**
1. Compiles without errors
2. Simulation runs in <500ms
3. Different polities with same species show different behavior patterns
4. War declarations vary based on personality (bold polities attack more)

---

## Summary

| File | Changes | Lines |
|------|---------|-------|
| polity.rs | Add 4 fields to 3 structs + helper method | ~40 |
| generation.rs | Initialize new fields | ~20 |
| human.rs | Dynamic thresholds | ~25 |
| dwarf.rs | Dynamic thresholds | ~10 |
| elf.rs | Dynamic thresholds | ~15 |
| warfare.rs | State updates | ~40 |
| **Total** | | **~150** |
