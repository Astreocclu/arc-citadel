//! Learning and chunk formation
//!
//! Entities develop chunks through practice. Encoding depth increases
//! logarithmically with repetitions. Unused chunks rust over time.

use crate::skills::{get_chunk_definition, ChunkId, ChunkLibrary, PersonalChunkState, CHUNK_LIBRARY};

/// Learning rate constant (higher = faster learning)
const LEARNING_RATE: f32 = 0.01;

/// Ticks until rust starts (unused chunks decay)
const RUST_THRESHOLD: u64 = 10000;

/// Rust decay rate per tick past threshold
const RUST_RATE: f32 = 0.0001;

/// Minimum encoding depth (chunks never fully forgotten)
const MIN_ENCODING: f32 = 0.1;

/// Maximum encoding depth
const MAX_ENCODING: f32 = 0.99;

/// Calculate encoding depth from repetition count
///
/// Uses logarithmic curve: fast early gains, slow mastery
pub fn calculate_encoding_depth(repetitions: u32) -> f32 {
    // depth = 1.0 - (1.0 / (1.0 + count * rate))
    let depth = 1.0 - (1.0 / (1.0 + repetitions as f32 * LEARNING_RATE));
    depth.clamp(MIN_ENCODING, MAX_ENCODING)
}

/// Process learning for an entity
///
/// - Consolidates pending experiences into encoding depth
/// - Checks for new chunk formation
/// - Applies rust decay to unused chunks
pub fn process_learning(library: &mut ChunkLibrary, tick: u64) {
    // 1. Consolidate experiences
    for exp in library.pending_experiences().to_vec() {
        if let Some(state) = library.get_chunk_mut(exp.chunk_id) {
            // Only successful executions increase repetition count and encoding depth
            if exp.success {
                state.repetition_count += 1;
                state.encoding_depth = calculate_encoding_depth(state.repetition_count);
            }
            // Failures still update last_used_tick (prevents rust) but don't teach
            state.last_used_tick = exp.tick;
        } else {
            // Experience with un-owned chunk - check if we can form it
            check_chunk_formation(library, exp.chunk_id, tick);
        }
    }

    library.clear_experiences();

    // 2. Check for new chunk formation from prerequisites
    check_all_formations(library, tick);

    // 3. Apply rust decay
    apply_rust_decay(library, tick);
}

/// Check if prerequisites are met to form a new chunk
fn check_chunk_formation(library: &mut ChunkLibrary, chunk_id: ChunkId, tick: u64) {
    if library.has_chunk(chunk_id) {
        return;
    }

    let Some(def) = get_chunk_definition(chunk_id) else {
        return;
    };

    // Check all prerequisites met with sufficient depth
    let prereqs_met = def.prerequisite_chunks.iter().all(|prereq| {
        library.get_chunk(*prereq).map_or(false, |s| s.encoding_depth > 0.3)
    });

    if prereqs_met {
        library.set_chunk(chunk_id, PersonalChunkState::new(tick));
    }
}

/// Check all potential chunk formations
fn check_all_formations(library: &mut ChunkLibrary, tick: u64) {
    // Get chunks we might be able to form
    let candidate_chunks: Vec<ChunkId> = CHUNK_LIBRARY
        .iter()
        .filter(|def| !library.has_chunk(def.id))
        .map(|def| def.id)
        .collect();

    for chunk_id in candidate_chunks {
        check_chunk_formation(library, chunk_id, tick);
    }
}

/// Apply rust decay to unused chunks
fn apply_rust_decay(library: &mut ChunkLibrary, tick: u64) {
    for state in library.chunks_mut().values_mut() {
        let ticks_since_use = tick.saturating_sub(state.last_used_tick);

        if ticks_since_use > RUST_THRESHOLD {
            let decay_ticks = ticks_since_use - RUST_THRESHOLD;
            let decay = decay_ticks as f32 * RUST_RATE;
            state.encoding_depth = (state.encoding_depth - decay).max(MIN_ENCODING);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::Experience;

    #[test]
    fn test_encoding_curve_starts_low() {
        assert!(calculate_encoding_depth(1) < 0.15);
    }

    #[test]
    fn test_encoding_curve_grows() {
        let depth_10 = calculate_encoding_depth(10);
        let depth_50 = calculate_encoding_depth(50);
        let depth_200 = calculate_encoding_depth(200);

        assert!(depth_10 < depth_50);
        assert!(depth_50 < depth_200);
    }

    #[test]
    fn test_encoding_curve_plateaus() {
        let depth_1000 = calculate_encoding_depth(1000);
        let depth_5000 = calculate_encoding_depth(5000);

        // Should be close to max
        assert!(depth_1000 > 0.9);
        // Marginal gains at high counts
        assert!((depth_5000 - depth_1000) < 0.1);
    }

    #[test]
    fn test_experience_increases_depth() {
        let mut lib = ChunkLibrary::new();
        // Use consistent values: encoding_depth should match what calculate_encoding_depth(100) returns
        let initial_reps = 100;
        let initial_depth = calculate_encoding_depth(initial_reps);
        lib.set_chunk(ChunkId::BasicSwing, PersonalChunkState {
            encoding_depth: initial_depth,
            repetition_count: initial_reps,
            last_used_tick: 0,
            formation_tick: 0,
        });

        let old_depth = lib.get_chunk(ChunkId::BasicSwing).unwrap().encoding_depth;

        lib.record_experience(Experience {
            chunk_id: ChunkId::BasicSwing,
            success: true,
            tick: 100,
        });

        process_learning(&mut lib, 100);
        let new_depth = lib.get_chunk(ChunkId::BasicSwing).unwrap().encoding_depth;

        assert!(new_depth > old_depth);
    }

    #[test]
    fn test_chunk_formation_from_prerequisites() {
        let mut lib = ChunkLibrary::new();

        // Add prerequisites with sufficient depth
        lib.set_chunk(ChunkId::BasicStance, PersonalChunkState {
            encoding_depth: 0.5,
            repetition_count: 50,
            last_used_tick: 0,
            formation_tick: 0,
        });
        lib.set_chunk(ChunkId::BasicSwing, PersonalChunkState {
            encoding_depth: 0.5,
            repetition_count: 50,
            last_used_tick: 0,
            formation_tick: 0,
        });

        // Should not have AttackSequence yet
        assert!(!lib.has_chunk(ChunkId::AttackSequence));

        process_learning(&mut lib, 1000);

        // Should now have formed AttackSequence
        assert!(lib.has_chunk(ChunkId::AttackSequence));
    }

    #[test]
    fn test_rust_decay() {
        let mut lib = ChunkLibrary::new();
        lib.set_chunk(ChunkId::BasicSwing, PersonalChunkState {
            encoding_depth: 0.8,
            repetition_count: 200,
            last_used_tick: 0,
            formation_tick: 0,
        });

        // Advance time past rust threshold
        process_learning(&mut lib, RUST_THRESHOLD + 5000);

        let depth = lib.get_chunk(ChunkId::BasicSwing).unwrap().encoding_depth;
        assert!(depth < 0.8);
        assert!(depth >= MIN_ENCODING);
    }
}
