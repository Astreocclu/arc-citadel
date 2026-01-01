use serde::{Deserialize, Serialize};
use crate::core::types::EntityId;
use super::memory::RelationshipMemory;
use super::event_types::{EventType, Valence};

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
