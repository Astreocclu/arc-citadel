use serde::{Deserialize, Serialize};
use crate::core::types::EntityId;
use super::memory::RelationshipMemory;
use super::event_types::{EventType, Valence};
use super::expectations::{BehaviorPattern, PatternType, MAX_PATTERNS_PER_SLOT, SALIENCE_FLOOR};
use super::service_types::{ServiceType, TraitIndicator};
use crate::core::calendar::TimePeriod;

/// How an entity feels about another based on memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disposition {
    Hostile,     // net < -0.5
    Suspicious,  // -0.5 <= net < -0.1
    Neutral,     // -0.1 <= net <= 0.1
    Friendly,    // 0.1 < net <= 0.5
    Favorable,   // net > 0.5
    Unknown,     // No memories
}

/// A known entity with bounded memory buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipSlot {
    /// Who this relationship is about
    pub target_id: EntityId,
    /// Bounded memory buffer, sorted by salience (highest first)
    pub memories: Vec<RelationshipMemory>,
    /// When we first met
    pub first_contact: u64,
    /// Most recent interaction
    pub last_contact: u64,
    /// Total interactions ever (even if memories evicted)
    pub interaction_count: u32,
    /// Behavioral expectations about this entity
    pub expectations: Vec<BehaviorPattern>,
}

impl RelationshipSlot {
    const MAX_MEMORIES: usize = 5;

    pub fn new(target_id: EntityId, first_contact: u64) -> Self {
        Self {
            target_id,
            memories: Vec::with_capacity(Self::MAX_MEMORIES),
            first_contact,
            last_contact: first_contact,
            interaction_count: 0,
            expectations: Vec::new(),
        }
    }

    /// Add memory, keeping only top 5 by salience
    pub fn add_memory(&mut self, memory: RelationshipMemory, current_tick: u64) {
        self.last_contact = current_tick;
        self.interaction_count += 1;

        self.memories.push(memory);

        // Sort by weighted importance (descending)
        self.memories.sort_by(|a, b| {
            b.weighted_importance()
                .partial_cmp(&a.weighted_importance())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Keep only top 5
        self.memories.truncate(Self::MAX_MEMORIES);
    }

    /// Apply decay to all memories and re-sort
    pub fn apply_decay(&mut self, current_tick: u64, decay_rate: f32) {
        for memory in &mut self.memories {
            memory.apply_decay(current_tick, decay_rate);
        }

        // Re-sort after decay
        self.memories.sort_by(|a, b| {
            b.weighted_importance()
                .partial_cmp(&a.weighted_importance())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Compute disposition from memories (no stored score)
    pub fn get_disposition(&self) -> Disposition {
        if self.memories.is_empty() {
            return Disposition::Unknown;
        }

        let positive: f32 = self.memories.iter()
            .filter(|m| m.valence == Valence::Positive)
            .map(|m| m.weighted_importance())
            .sum();

        let negative: f32 = self.memories.iter()
            .filter(|m| m.valence == Valence::Negative)
            .map(|m| m.weighted_importance())
            .sum();

        let net = positive - negative;

        match net {
            n if n > 0.5 => Disposition::Favorable,
            n if n > 0.1 => Disposition::Friendly,
            n if n < -0.5 => Disposition::Hostile,
            n if n < -0.1 => Disposition::Suspicious,
            _ => Disposition::Neutral,
        }
    }

    /// Calculate relationship strength for eviction decisions
    pub fn strength(&self, current_tick: u64, params: &SocialMemoryParams) -> f32 {
        let ticks_per_day = 1000;
        let days_since = (current_tick.saturating_sub(self.last_contact)) as f32 / ticks_per_day as f32;
        let recency_score = 1.0 / (1.0 + days_since * 0.1);

        let intensity_score: f32 = self.memories.iter()
            .map(|m| m.weighted_importance())
            .sum::<f32>() / Self::MAX_MEMORIES as f32;

        let depth_score = (self.interaction_count as f32 / 20.0).min(1.0);

        recency_score * params.recency_weight +
        intensity_score * params.intensity_weight +
        depth_score * params.interaction_count_weight
    }

    // ===== Expectation Methods =====

    /// Add an expectation, strengthening if similar exists, evicting lowest salience if at capacity
    pub fn add_expectation(&mut self, pattern: BehaviorPattern) {
        // Check if similar pattern exists - strengthen it instead
        if let Some(existing) = self.find_expectation_mut(&pattern.pattern_type) {
            existing.record_observation(pattern.last_confirmed);
            return;
        }

        // Evict lowest salience if at capacity
        if self.expectations.len() >= MAX_PATTERNS_PER_SLOT {
            if let Some(min_idx) = self.expectations
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.salience.partial_cmp(&b.salience).unwrap())
                .map(|(i, _)| i)
            {
                self.expectations.remove(min_idx);
            }
        }

        self.expectations.push(pattern);
    }

    /// Find an expectation by pattern type
    pub fn find_expectation(&self, pattern_type: &PatternType) -> Option<&BehaviorPattern> {
        self.expectations.iter().find(|p| Self::pattern_matches(&p.pattern_type, pattern_type))
    }

    /// Find an expectation by pattern type (mutable)
    pub fn find_expectation_mut(&mut self, pattern_type: &PatternType) -> Option<&mut BehaviorPattern> {
        self.expectations.iter_mut().find(|p| Self::pattern_matches(&p.pattern_type, pattern_type))
    }

    /// Compare PatternType variants, matching on key fields (not all fields)
    ///
    /// - ProvidesWhenAsked: matches if service_type matches
    /// - BehavesWithTrait: matches if trait_indicator matches
    /// - LocationDuring: matches if location_id AND time_period match
    /// - RespondsToEvent: matches if event_type matches (response can differ)
    pub fn pattern_matches(a: &PatternType, b: &PatternType) -> bool {
        match (a, b) {
            (PatternType::ProvidesWhenAsked { service_type: s1 },
             PatternType::ProvidesWhenAsked { service_type: s2 }) => s1 == s2,
            (PatternType::BehavesWithTrait { trait_indicator: t1 },
             PatternType::BehavesWithTrait { trait_indicator: t2 }) => t1 == t2,
            (PatternType::LocationDuring { location_id: l1, time_period: tp1 },
             PatternType::LocationDuring { location_id: l2, time_period: tp2 }) => l1 == l2 && tp1 == tp2,
            (PatternType::RespondsToEvent { event_type: e1, .. },
             PatternType::RespondsToEvent { event_type: e2, .. }) => e1 == e2,
            _ => false,
        }
    }

    /// Decay all expectations and remove stale ones
    pub fn decay_expectations(&mut self, decay_rate: f32) {
        for pattern in &mut self.expectations {
            pattern.apply_decay(decay_rate);
        }
        self.expectations.retain(|p| !p.is_stale(SALIENCE_FLOOR));
    }
}

/// Pre-threshold encounter tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingEncounter {
    pub target_id: EntityId,
    pub accumulated_salience: f32,
    pub encounter_count: u32,
    pub most_recent_tick: u64,
    pub most_significant: Option<(EventType, f32)>, // (event, intensity)
}

impl PendingEncounter {
    pub fn new(target_id: EntityId, tick: u64) -> Self {
        Self {
            target_id,
            accumulated_salience: 0.0,
            encounter_count: 0,
            most_recent_tick: tick,
            most_significant: None,
        }
    }

    pub fn add_encounter(&mut self, event_type: EventType, intensity: f32, tick: u64) {
        self.accumulated_salience += intensity;
        self.encounter_count += 1;
        self.most_recent_tick = tick;

        // Track most significant event
        if self.most_significant.map_or(true, |(_, i)| intensity > i) {
            self.most_significant = Some((event_type, intensity));
        }
    }
}

/// Species-specific social memory parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMemoryParams {
    pub max_relationship_slots: usize,
    pub memories_per_slot: usize,
    pub encounter_buffer_size: usize,
    pub slot_allocation_threshold: f32,
    pub memory_importance_floor: f32,
    pub memory_salience_decay: f32,
    pub recency_weight: f32,
    pub intensity_weight: f32,
    pub interaction_count_weight: f32,
}

impl Default for SocialMemoryParams {
    fn default() -> Self {
        // Human defaults from spec
        Self {
            max_relationship_slots: 200,
            memories_per_slot: 5,
            encounter_buffer_size: 50,
            slot_allocation_threshold: 0.3,
            memory_importance_floor: 0.2,
            memory_salience_decay: 0.02, // 2% per day
            recency_weight: 0.4,
            intensity_weight: 0.4,
            interaction_count_weight: 0.2,
        }
    }
}

/// Complete social memory for one entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMemory {
    /// Known entities (bounded)
    pub slots: Vec<RelationshipSlot>,
    /// Pre-threshold encounters
    pub encounter_buffer: Vec<PendingEncounter>,
    /// Species parameters
    pub params: SocialMemoryParams,
}

impl SocialMemory {
    pub fn new() -> Self {
        Self::with_params(SocialMemoryParams::default())
    }

    pub fn with_params(params: SocialMemoryParams) -> Self {
        Self {
            slots: Vec::with_capacity(params.max_relationship_slots),
            encounter_buffer: Vec::with_capacity(params.encounter_buffer_size),
            params,
        }
    }

    /// Find slot for a known entity
    pub fn find_slot(&self, target: EntityId) -> Option<&RelationshipSlot> {
        self.slots.iter().find(|s| s.target_id == target)
    }

    pub fn find_slot_mut(&mut self, target: EntityId) -> Option<&mut RelationshipSlot> {
        self.slots.iter_mut().find(|s| s.target_id == target)
    }

    /// Get disposition toward entity (computed from memories)
    pub fn get_disposition(&self, target: EntityId) -> Disposition {
        self.find_slot(target)
            .map(|slot| slot.get_disposition())
            .unwrap_or(Disposition::Unknown)
    }

    /// Record an encounter (may go to buffer or directly to slot)
    pub fn record_encounter(
        &mut self,
        target: EntityId,
        event_type: EventType,
        intensity: f32,
        current_tick: u64,
    ) {
        // Below importance floor? Ignore
        if intensity < self.params.memory_importance_floor {
            return;
        }

        // Already known? Add memory directly
        if let Some(slot) = self.find_slot_mut(target) {
            let memory = RelationshipMemory::new(
                event_type,
                event_type.default_valence(),
                intensity,
                current_tick,
            );
            slot.add_memory(memory, current_tick);
            return;
        }

        // Update or create pending encounter
        if let Some(encounter) = self.encounter_buffer.iter_mut()
            .find(|e| e.target_id == target)
        {
            encounter.add_encounter(event_type, intensity, current_tick);

            // Check promotion threshold
            if encounter.accumulated_salience >= self.params.slot_allocation_threshold {
                self.promote_encounter(target, current_tick);
            }
        } else {
            // New encounter
            let mut encounter = PendingEncounter::new(target, current_tick);
            encounter.add_encounter(event_type, intensity, current_tick);

            // Check if immediately significant enough
            if encounter.accumulated_salience >= self.params.slot_allocation_threshold {
                // Significant enough for immediate promotion - create slot with memory
                self.promote_encounter_with_event(target, event_type, intensity, current_tick);
            } else {
                // Add to buffer (evict oldest if full)
                if self.encounter_buffer.len() >= self.params.encounter_buffer_size {
                    // Remove oldest by most_recent_tick
                    if let Some(oldest_idx) = self.encounter_buffer.iter()
                        .enumerate()
                        .min_by_key(|(_, e)| e.most_recent_tick)
                        .map(|(i, _)| i)
                    {
                        self.encounter_buffer.remove(oldest_idx);
                    }
                }
                self.encounter_buffer.push(encounter);
            }
        }
    }

    /// Promote a new significant encounter directly to a relationship slot
    fn promote_encounter_with_event(
        &mut self,
        target: EntityId,
        event_type: EventType,
        intensity: f32,
        current_tick: u64,
    ) {
        // Ensure we have room (evict if needed)
        if self.slots.len() >= self.params.max_relationship_slots {
            self.evict_weakest_slot(current_tick);
        }

        // Create new slot with the initial memory
        let mut slot = RelationshipSlot::new(target, current_tick);
        let memory = RelationshipMemory::new(
            event_type,
            event_type.default_valence(),
            intensity,
            current_tick,
        );
        slot.add_memory(memory, current_tick);
        self.slots.push(slot);
    }

    /// Promote encounter to relationship slot
    pub fn promote_encounter(&mut self, target: EntityId, current_tick: u64) {
        // Find and remove from buffer
        let encounter = self.encounter_buffer.iter()
            .position(|e| e.target_id == target)
            .map(|i| self.encounter_buffer.remove(i));

        // Ensure we have room (evict if needed)
        if self.slots.len() >= self.params.max_relationship_slots {
            self.evict_weakest_slot(current_tick);
        }

        // Create new slot
        let mut slot = RelationshipSlot::new(target, current_tick);

        // Seed with most significant event from encounter
        if let Some(enc) = encounter {
            if let Some((event_type, intensity)) = enc.most_significant {
                let memory = RelationshipMemory::new(
                    event_type,
                    event_type.default_valence(),
                    intensity,
                    current_tick,
                );
                slot.add_memory(memory, current_tick);
            }
        }

        self.slots.push(slot);
    }

    /// Evict weakest relationship to make room
    pub fn evict_weakest_slot(&mut self, current_tick: u64) {
        if self.slots.is_empty() {
            return;
        }

        let weakest_idx = self.slots.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.strength(current_tick, &self.params)
                    .partial_cmp(&b.strength(current_tick, &self.params))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
            .unwrap_or(0);

        self.slots.remove(weakest_idx);
    }

    /// Decay all memories (call once per day)
    pub fn apply_decay(&mut self, current_tick: u64) {
        for slot in &mut self.slots {
            slot.apply_decay(current_tick, self.params.memory_salience_decay);
        }

        // Decay encounter buffer salience
        for encounter in &mut self.encounter_buffer {
            encounter.accumulated_salience *= 1.0 - self.params.memory_salience_decay;
        }

        // Remove near-zero encounters
        self.encounter_buffer.retain(|e| e.accumulated_salience > 0.01);
    }
}

impl Default for SocialMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_keeps_top_5_by_salience() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);

        // Add 6 memories with varying intensities
        for i in 0..6 {
            let memory = RelationshipMemory::new(
                EventType::Transaction,
                Valence::Positive,
                0.1 * (i as f32 + 1.0), // 0.1, 0.2, 0.3, 0.4, 0.5, 0.6
                i as u64 * 10,
            );
            slot.add_memory(memory, i as u64 * 10);
        }

        // Should only have 5 memories
        assert_eq!(slot.memories.len(), 5);

        // Lowest intensity (0.1) should have been evicted
        let min_intensity = slot.memories.iter()
            .map(|m| m.intensity)
            .fold(f32::MAX, f32::min);
        assert!(min_intensity > 0.15); // 0.1 was evicted
    }

    #[test]
    fn test_disposition_from_memories() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);

        // Add positive memories
        slot.add_memory(RelationshipMemory::new(
            EventType::AidReceived, Valence::Positive, 0.8, 0
        ), 0);
        slot.add_memory(RelationshipMemory::new(
            EventType::GiftReceived, Valence::Positive, 0.6, 10
        ), 10);

        let disposition = slot.get_disposition();
        assert_eq!(disposition, Disposition::Favorable);
    }

    #[test]
    fn test_disposition_hostile_from_negative_memories() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);

        // Add negative memories
        slot.add_memory(RelationshipMemory::new(
            EventType::HarmReceived, Valence::Negative, 0.8, 0
        ), 0);
        slot.add_memory(RelationshipMemory::new(
            EventType::Betrayal, Valence::Negative, 0.9, 10
        ), 10);

        let disposition = slot.get_disposition();
        assert_eq!(disposition, Disposition::Hostile);
    }

    #[test]
    fn test_disposition_neutral_from_mixed_memories() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);

        // Add balanced positive and negative memories
        slot.add_memory(RelationshipMemory::new(
            EventType::AidReceived, Valence::Positive, 0.5, 0
        ), 0);
        slot.add_memory(RelationshipMemory::new(
            EventType::HarmReceived, Valence::Negative, 0.5, 10
        ), 10);

        let disposition = slot.get_disposition();
        assert_eq!(disposition, Disposition::Neutral);
    }

    #[test]
    fn test_disposition_unknown_no_memories() {
        let target = EntityId::new();
        let slot = RelationshipSlot::new(target, 0);

        let disposition = slot.get_disposition();
        assert_eq!(disposition, Disposition::Unknown);
    }

    #[test]
    fn test_pending_encounter_accumulation() {
        let target = EntityId::new();
        let mut encounter = PendingEncounter::new(target, 0);

        encounter.add_encounter(EventType::Transaction, 0.2, 100);
        encounter.add_encounter(EventType::Transaction, 0.3, 200);

        assert_eq!(encounter.encounter_count, 2);
        assert!((encounter.accumulated_salience - 0.5).abs() < 0.01);
        assert_eq!(encounter.most_recent_tick, 200);

        // Most significant should be the 0.3 intensity one
        assert!(encounter.most_significant.is_some());
        let (event, intensity) = encounter.most_significant.unwrap();
        assert_eq!(event, EventType::Transaction);
        assert!((intensity - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_social_memory_record_encounter_creates_slot() {
        let mut memory = SocialMemory::new();
        let target = EntityId::new();

        // Record a significant encounter (above threshold)
        memory.record_encounter(target, EventType::AidReceived, 0.7, 0);

        // Should have created a slot
        assert_eq!(memory.slots.len(), 1);
        assert!(memory.find_slot(target).is_some());
    }

    #[test]
    fn test_social_memory_record_encounter_buffers_small() {
        let mut memory = SocialMemory::new();
        let target = EntityId::new();

        // Record a small encounter (below threshold but above floor)
        memory.record_encounter(target, EventType::Transaction, 0.25, 0);

        // Should be in buffer, not slot
        assert_eq!(memory.slots.len(), 0);
        assert_eq!(memory.encounter_buffer.len(), 1);
    }

    #[test]
    fn test_social_memory_ignores_below_floor() {
        let mut memory = SocialMemory::new();
        let target = EntityId::new();

        // Record a tiny encounter (below importance floor of 0.2)
        memory.record_encounter(target, EventType::Observation, 0.1, 0);

        // Should be ignored completely
        assert_eq!(memory.slots.len(), 0);
        assert_eq!(memory.encounter_buffer.len(), 0);
    }

    #[test]
    fn test_social_memory_get_disposition() {
        let mut memory = SocialMemory::new();
        let target = EntityId::new();
        let unknown = EntityId::new();

        // Record encounters that create a friendly relationship
        memory.record_encounter(target, EventType::AidReceived, 0.7, 0);
        memory.record_encounter(target, EventType::GiftReceived, 0.6, 10);

        assert_eq!(memory.get_disposition(target), Disposition::Favorable);
        assert_eq!(memory.get_disposition(unknown), Disposition::Unknown);
    }

    #[test]
    fn test_social_memory_decay() {
        let mut memory = SocialMemory::new();
        let target = EntityId::new();

        // Record a significant encounter
        memory.record_encounter(target, EventType::AidReceived, 0.7, 0);

        // Get initial salience
        let initial_salience = memory.find_slot(target).unwrap().memories[0].salience;
        assert!((initial_salience - 1.0).abs() < 0.01);

        // Apply decay at tick 1000 (1 day later)
        memory.apply_decay(1000);

        // Salience should have decayed
        let final_salience = memory.find_slot(target).unwrap().memories[0].salience;
        assert!(final_salience < initial_salience);
    }

    #[test]
    fn test_relationship_slot_strength() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);
        let params = SocialMemoryParams::default();

        // Add some memories
        slot.add_memory(RelationshipMemory::new(
            EventType::AidReceived, Valence::Positive, 0.8, 0
        ), 0);

        // Strength should be positive
        let strength = slot.strength(0, &params);
        assert!(strength > 0.0);

        // Strength at later tick should be lower (recency decay)
        let strength_later = slot.strength(10000, &params);
        assert!(strength_later < strength);
    }

    #[test]
    fn test_evict_weakest_slot() {
        let mut memory = SocialMemory::with_params(SocialMemoryParams {
            max_relationship_slots: 3,
            ..Default::default()
        });

        // Create 3 relationships with different strengths
        let weak_target = EntityId::new();
        let medium_target = EntityId::new();
        let strong_target = EntityId::new();

        // Weak: old, low intensity
        memory.record_encounter(weak_target, EventType::Transaction, 0.3, 0);

        // Medium: moderate
        memory.record_encounter(medium_target, EventType::AidReceived, 0.5, 500);

        // Strong: recent, high intensity
        memory.record_encounter(strong_target, EventType::AidReceived, 0.9, 1000);

        assert_eq!(memory.slots.len(), 3);

        // Now add a 4th that should trigger eviction
        let new_target = EntityId::new();
        memory.record_encounter(new_target, EventType::AidReceived, 0.8, 1500);

        // Should still have 3 slots
        assert_eq!(memory.slots.len(), 3);

        // The weak target should have been evicted
        assert!(memory.find_slot(weak_target).is_none());
        assert!(memory.find_slot(strong_target).is_some());
        assert!(memory.find_slot(new_target).is_some());
    }
}

#[cfg(test)]
mod critical_edge_case_tests {
    use super::*;
    
    /// Test that decay is recalculated from tick_created (not cumulative)
    /// This verifies the decay model semantics
    #[test]
    fn test_decay_is_age_based_not_cumulative() {
        let mut memory = RelationshipMemory::new(
            EventType::AidReceived,
            Valence::Positive,
            0.8,
            0, // created at tick 0
        );
        
        // Decay at tick 2000
        memory.apply_decay(2000, 0.02);
        let salience_at_2k = memory.salience;
        
        // Decay again at tick 1000 (earlier tick!)
        memory.apply_decay(1000, 0.02);
        let salience_at_1k = memory.salience;
        
        // Since decay is age-based, calling with earlier tick should give HIGHER salience
        // This is the current behavior - verify it's intentional
        assert!(salience_at_1k > salience_at_2k, 
            "Age-based decay: 1k ticks ({:.4}) should be higher than 2k ticks ({:.4})",
            salience_at_1k, salience_at_2k);
    }
    
    /// Test exact threshold boundaries
    #[test]
    fn test_threshold_boundary_at_exactly_floor() {
        let mut memory = SocialMemory::new();
        let target = EntityId::new();
        
        // importance_floor is 0.2
        // At exactly 0.2 should be stored (not < 0.2)
        memory.record_encounter(target, EventType::Observation, 0.2, 0);
        
        let stored = memory.encounter_buffer.iter().any(|e| e.target_id == target)
                     || memory.find_slot(target).is_some();
        assert!(stored, "Encounter at exactly floor (0.2) should be stored");
    }
    
    /// Test that slot promotion at exactly threshold works
    #[test]
    fn test_slot_promotion_at_exactly_threshold() {
        let mut memory = SocialMemory::new();
        let target = EntityId::new();
        
        // slot_allocation_threshold is 0.3
        // Single encounter at 0.3 should create slot immediately
        memory.record_encounter(target, EventType::AidReceived, 0.3, 0);
        
        assert!(memory.find_slot(target).is_some(), 
            "Encounter at exactly threshold (0.3) should create slot");
        assert!(memory.encounter_buffer.is_empty(),
            "Should not be in buffer if promoted to slot");
    }
    
    /// Test that slot just below threshold goes to buffer
    #[test]
    fn test_slot_buffer_just_below_threshold() {
        let mut memory = SocialMemory::new();
        let target = EntityId::new();
        
        // Just below threshold
        memory.record_encounter(target, EventType::Transaction, 0.29, 0);
        
        assert!(memory.find_slot(target).is_none(), 
            "0.29 should not create slot");
        assert!(memory.encounter_buffer.iter().any(|e| e.target_id == target),
            "0.29 should be in buffer");
    }
    
    /// Test memory reordering after decay
    #[test]
    fn test_memory_reordering_after_decay() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);
        
        // Add two memories: one old high-intensity, one recent lower-intensity
        let old_memory = RelationshipMemory::new(
            EventType::Betrayal, Valence::Negative, 0.9, 0  // old, high
        );
        let recent_memory = RelationshipMemory::new(
            EventType::Transaction, Valence::Positive, 0.4, 5000  // recent, low
        );
        
        slot.memories.push(old_memory);
        slot.memories.push(recent_memory);
        slot.memories.sort_by(|a, b| {
            b.weighted_importance()
                .partial_cmp(&a.weighted_importance())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Before decay, old high-intensity should be first
        assert!(slot.memories[0].intensity > 0.8, "High intensity should be first before decay");
        
        // Apply decay at tick 10000 (10 days later)
        slot.apply_decay(10000, 0.02);
        
        // After decay, the old memory has decayed more, recent memory decayed less
        // Recent memory might now be higher weighted_importance
        let first_importance = slot.memories[0].weighted_importance();
        let second_importance = slot.memories[1].weighted_importance();
        
        assert!(first_importance >= second_importance, 
            "Memories should be sorted by weighted_importance after decay");
    }
    
    /// Test disposition with all memories at salience floor
    #[test]
    fn test_disposition_with_floor_salience_memories() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);
        
        // Add a positive memory
        let mut memory = RelationshipMemory::new(
            EventType::AidReceived, Valence::Positive, 0.8, 0
        );
        
        // Decay to floor (salience = 0.01)
        memory.apply_decay(1000000, 0.5); // Massive decay
        assert!((memory.salience - 0.01).abs() < 0.001, "Should be at floor");
        
        slot.memories.push(memory);
        
        let disposition = slot.get_disposition();
        
        // weighted_importance = 0.8 * 0.01 = 0.008
        // This is very low, should be Neutral
        assert!(disposition == Disposition::Neutral || disposition == Disposition::Friendly,
            "Floor-salience memories should give low disposition: {:?}", disposition);
    }
    
    /// Test encounter buffer eviction (oldest removed)
    #[test]
    fn test_encounter_buffer_evicts_oldest() {
        let mut memory = SocialMemory::with_params(SocialMemoryParams {
            encounter_buffer_size: 3,
            slot_allocation_threshold: 10.0, // Very high so nothing promotes
            memory_importance_floor: 0.1,
            ..Default::default()
        });
        
        let t1 = EntityId::new();
        let t2 = EntityId::new();
        let t3 = EntityId::new();
        let t4 = EntityId::new();
        
        memory.record_encounter(t1, EventType::Transaction, 0.2, 100);  // oldest
        memory.record_encounter(t2, EventType::Transaction, 0.2, 200);
        memory.record_encounter(t3, EventType::Transaction, 0.2, 300);
        
        assert_eq!(memory.encounter_buffer.len(), 3);
        
        // Add 4th, should evict oldest (t1)
        memory.record_encounter(t4, EventType::Transaction, 0.2, 400);
        
        assert_eq!(memory.encounter_buffer.len(), 3);
        assert!(!memory.encounter_buffer.iter().any(|e| e.target_id == t1),
            "Oldest encounter (t1) should be evicted");
        assert!(memory.encounter_buffer.iter().any(|e| e.target_id == t4),
            "Newest encounter (t4) should be present");
    }
}

#[cfg(test)]
mod expectation_tests {
    use super::*;

    #[test]
    fn test_relationship_slot_expectations() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);

        assert!(slot.expectations.is_empty());

        // Add expectation
        let pattern = BehaviorPattern::new(
            PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting },
            100,
        );
        slot.add_expectation(pattern);

        assert_eq!(slot.expectations.len(), 1);

        // Find expectation
        let found = slot.find_expectation(&PatternType::ProvidesWhenAsked {
            service_type: ServiceType::Crafting
        });
        assert!(found.is_some());
    }

    #[test]
    fn test_expectations_bounded() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);

        // Add more than MAX_PATTERNS_PER_SLOT
        for i in 0..12 {
            let pattern = BehaviorPattern::new(
                PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Reliable },
                i as u64 * 100,
            );
            slot.add_expectation(pattern);
        }

        // Should be bounded
        assert!(slot.expectations.len() <= MAX_PATTERNS_PER_SLOT);
    }

    #[test]
    fn test_add_expectation_strengthens_existing() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);

        // Add initial expectation
        let pattern1 = BehaviorPattern::new(
            PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting },
            100,
        );
        slot.add_expectation(pattern1);

        let initial_obs = slot.expectations[0].observation_count;
        let initial_salience = slot.expectations[0].salience;

        // Add same type again - should strengthen, not add new
        let pattern2 = BehaviorPattern::new(
            PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting },
            200,
        );
        slot.add_expectation(pattern2);

        // Should still have just 1 expectation
        assert_eq!(slot.expectations.len(), 1);
        // But observation count should be higher
        assert!(slot.expectations[0].observation_count > initial_obs);
        // And salience should be higher
        assert!(slot.expectations[0].salience > initial_salience);
    }

    #[test]
    fn test_find_expectation_mut() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);

        let pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Generous },
            100,
        );
        slot.add_expectation(pattern);

        // Find and modify
        let found = slot.find_expectation_mut(&PatternType::BehavesWithTrait {
            trait_indicator: TraitIndicator::Generous
        });
        assert!(found.is_some());

        let pattern_mut = found.unwrap();
        pattern_mut.record_violation(200);

        // Verify the change persisted
        let found_again = slot.find_expectation(&PatternType::BehavesWithTrait {
            trait_indicator: TraitIndicator::Generous
        }).unwrap();
        assert_eq!(found_again.violation_count, 1);
    }

    #[test]
    fn test_pattern_matches_provides_when_asked() {
        // Same service type should match
        assert!(RelationshipSlot::pattern_matches(
            &PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting },
            &PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting }
        ));

        // Different service type should not match
        assert!(!RelationshipSlot::pattern_matches(
            &PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting },
            &PatternType::ProvidesWhenAsked { service_type: ServiceType::Trading }
        ));
    }

    #[test]
    fn test_pattern_matches_behaves_with_trait() {
        // Same trait should match
        assert!(RelationshipSlot::pattern_matches(
            &PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Generous },
            &PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Generous }
        ));

        // Different trait should not match
        assert!(!RelationshipSlot::pattern_matches(
            &PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Generous },
            &PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Aggressive }
        ));
    }

    #[test]
    fn test_pattern_matches_location_during() {
        let loc1 = EntityId::new();
        let loc2 = EntityId::new();

        // Same location and time should match
        assert!(RelationshipSlot::pattern_matches(
            &PatternType::LocationDuring { location_id: loc1, time_period: TimePeriod::Morning },
            &PatternType::LocationDuring { location_id: loc1, time_period: TimePeriod::Morning }
        ));

        // Different location should not match
        assert!(!RelationshipSlot::pattern_matches(
            &PatternType::LocationDuring { location_id: loc1, time_period: TimePeriod::Morning },
            &PatternType::LocationDuring { location_id: loc2, time_period: TimePeriod::Morning }
        ));

        // Different time period should not match
        assert!(!RelationshipSlot::pattern_matches(
            &PatternType::LocationDuring { location_id: loc1, time_period: TimePeriod::Morning },
            &PatternType::LocationDuring { location_id: loc1, time_period: TimePeriod::Evening }
        ));
    }

    #[test]
    fn test_pattern_matches_responds_to_event() {
        use crate::actions::catalog::ActionCategory;

        // Same event type should match even with different response
        assert!(RelationshipSlot::pattern_matches(
            &PatternType::RespondsToEvent {
                event_type: EventType::HarmReceived,
                typical_response: ActionCategory::Combat
            },
            &PatternType::RespondsToEvent {
                event_type: EventType::HarmReceived,
                typical_response: ActionCategory::Movement
            }
        ));

        // Different event type should not match
        assert!(!RelationshipSlot::pattern_matches(
            &PatternType::RespondsToEvent {
                event_type: EventType::HarmReceived,
                typical_response: ActionCategory::Combat
            },
            &PatternType::RespondsToEvent {
                event_type: EventType::AidReceived,
                typical_response: ActionCategory::Combat
            }
        ));
    }

    #[test]
    fn test_pattern_matches_different_variants() {
        // Different pattern variants should never match
        assert!(!RelationshipSlot::pattern_matches(
            &PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting },
            &PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Reliable }
        ));
    }

    #[test]
    fn test_decay_expectations() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);

        let pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Reliable },
            0,
        );
        slot.add_expectation(pattern);

        let initial_salience = slot.expectations[0].salience;

        // Apply decay
        slot.decay_expectations(0.1);

        // Salience should have decayed
        assert!(slot.expectations[0].salience < initial_salience);
    }

    #[test]
    fn test_decay_removes_stale_expectations() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);

        let pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Reliable },
            0,
        );
        slot.add_expectation(pattern);

        // Apply heavy decay multiple times to make it stale
        for _ in 0..50 {
            slot.decay_expectations(0.2);
        }

        // Expectation should have been removed due to falling below SALIENCE_FLOOR
        assert!(slot.expectations.is_empty());
    }

    #[test]
    fn test_eviction_removes_lowest_salience() {
        let target = EntityId::new();
        let mut slot = RelationshipSlot::new(target, 0);

        // Add MAX_PATTERNS_PER_SLOT patterns with varying salience
        // Note: They all start with same salience, so we need to modify them
        for i in 0..MAX_PATTERNS_PER_SLOT {
            let mut pattern = BehaviorPattern::new(
                PatternType::LocationDuring {
                    location_id: EntityId::new(),
                    time_period: TimePeriod::Morning
                },
                i as u64 * 100,
            );
            // Boost salience of later patterns by recording observations
            for _ in 0..i {
                pattern.record_observation((i * 100) as u64);
            }
            slot.expectations.push(pattern);
        }

        assert_eq!(slot.expectations.len(), MAX_PATTERNS_PER_SLOT);

        // Find the current minimum salience
        let min_salience_before = slot.expectations.iter()
            .map(|p| p.salience)
            .fold(f32::MAX, f32::min);

        // Add one more pattern - should evict lowest salience
        let new_pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Punctual },
            1000,
        );
        let new_pattern_salience = new_pattern.salience;
        slot.add_expectation(new_pattern);

        // Should still be bounded
        assert_eq!(slot.expectations.len(), MAX_PATTERNS_PER_SLOT);

        // The new minimum salience should be >= the old minimum
        // (because we evicted the lowest)
        let min_salience_after = slot.expectations.iter()
            .map(|p| p.salience)
            .fold(f32::MAX, f32::min);
        assert!(min_salience_after >= min_salience_before ||
                min_salience_after == new_pattern_salience,
            "Lowest salience pattern should have been evicted");
    }
}
