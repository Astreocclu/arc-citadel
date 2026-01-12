//! Integration tests for spawn loadouts using experience-based history

use arc_citadel::actions::catalog::ActionId;
use arc_citadel::skills::{
    generate_chunks_from_history, generate_history_for_role, skill_check, spend_attention,
    ActivityType, CraftSpecialty, LifeExperience, Role,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[test]
fn test_farmer_builds_with_skill_check() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let history = generate_history_for_role(Role::Farmer, 25, &mut rng);
    let mut library = generate_chunks_from_history(&history, 0, &mut rng);
    library.attention_budget = 1.0;

    let result = skill_check(&library, ActionId::Build);

    assert!(result.can_execute);
    assert!(result.attention_cost > 0.0);
    assert!(result.skill_modifier >= 0.1);
    assert!(result.skill_modifier < 1.0);
}

#[test]
fn test_carpenter_builds_better_than_farmer() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let farmer_history = generate_history_for_role(Role::Farmer, 30, &mut rng);
    let mut farmer = generate_chunks_from_history(&farmer_history, 0, &mut rng);
    farmer.attention_budget = 1.0;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let carpenter_history =
        generate_history_for_role(Role::Craftsman(CraftSpecialty::Carpentry), 30, &mut rng);
    let mut carpenter = generate_chunks_from_history(&carpenter_history, 0, &mut rng);
    carpenter.attention_budget = 1.0;

    let farmer_result = skill_check(&farmer, ActionId::Build);
    let carpenter_result = skill_check(&carpenter, ActionId::Build);

    assert!(
        carpenter_result.skill_modifier > farmer_result.skill_modifier,
        "Carpenter {} should build better than farmer {}",
        carpenter_result.skill_modifier,
        farmer_result.skill_modifier
    );
}

#[test]
fn test_veteran_smith_vs_apprentice() {
    let apprentice_history = vec![
        LifeExperience {
            activity: ActivityType::GeneralLife,
            duration_years: 12.0,
            intensity: 1.0,
            training_quality: 0.5,
        },
        LifeExperience {
            activity: ActivityType::Smithing,
            duration_years: 2.0,
            intensity: 1.0,
            training_quality: 0.7,
        },
    ];

    let veteran_history = vec![
        LifeExperience {
            activity: ActivityType::GeneralLife,
            duration_years: 12.0,
            intensity: 1.0,
            training_quality: 0.5,
        },
        LifeExperience {
            activity: ActivityType::Smithing,
            duration_years: 25.0,
            intensity: 1.0,
            training_quality: 0.7,
        },
    ];

    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);

    let apprentice = generate_chunks_from_history(&apprentice_history, 0, &mut rng1);
    let veteran = generate_chunks_from_history(&veteran_history, 0, &mut rng2);

    let apprentice_depth = apprentice
        .get_chunk(arc_citadel::skills::ChunkId::CraftBasicHammerWork)
        .unwrap()
        .encoding_depth;
    let veteran_depth = veteran
        .get_chunk(arc_citadel::skills::ChunkId::CraftBasicHammerWork)
        .unwrap()
        .encoding_depth;

    assert!(veteran_depth > apprentice_depth + 0.2);
}

#[test]
fn test_farmer_turned_soldier() {
    let mixed_history = vec![
        LifeExperience {
            activity: ActivityType::GeneralLife,
            duration_years: 12.0,
            intensity: 1.0,
            training_quality: 0.5,
        },
        LifeExperience {
            activity: ActivityType::Farming,
            duration_years: 8.0,
            intensity: 1.0,
            training_quality: 0.5,
        },
        LifeExperience {
            activity: ActivityType::MilitaryTraining {
                unit_type: arc_citadel::skills::UnitType::Infantry,
            },
            duration_years: 2.0,
            intensity: 1.0,
            training_quality: 0.7,
        },
        LifeExperience {
            activity: ActivityType::CombatExperience { battles_fought: 3 },
            duration_years: 5.0,
            intensity: 0.3,
            training_quality: 1.0,
        },
    ];

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let library = generate_chunks_from_history(&mixed_history, 0, &mut rng);

    // Has both farming AND combat skills
    assert!(library.has_chunk(arc_citadel::skills::ChunkId::PhysSustainedLabor));
    assert!(library.has_chunk(arc_citadel::skills::ChunkId::BasicStance));
    assert!(library.has_chunk(arc_citadel::skills::ChunkId::EngageMelee));
}

#[test]
fn test_noble_without_combat_training() {
    // A noble who never trained
    let non_military_noble = vec![
        LifeExperience {
            activity: ActivityType::GeneralLife,
            duration_years: 12.0,
            intensity: 1.0,
            training_quality: 0.5,
        },
        LifeExperience {
            activity: ActivityType::FormalEducation,
            duration_years: 8.0,
            intensity: 0.8,
            training_quality: 0.8,
        },
        LifeExperience {
            activity: ActivityType::CourtLife,
            duration_years: 10.0,
            intensity: 0.7,
            training_quality: 0.6,
        },
    ];

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let library = generate_chunks_from_history(&non_military_noble, 0, &mut rng);

    // Has social skills
    assert!(library.has_chunk(arc_citadel::skills::ChunkId::SocialProjectConfidence));
    assert!(library.has_chunk(arc_citadel::skills::ChunkId::KnowFluentReading));

    // Does NOT have combat skills (never trained!)
    assert!(!library.has_chunk(arc_citadel::skills::ChunkId::BasicStance));
    assert!(!library.has_chunk(arc_citadel::skills::ChunkId::BasicSwing));
}

#[test]
fn test_chunking_affects_actions() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let history = generate_history_for_role(Role::Farmer, 25, &mut rng);
    let mut library = generate_chunks_from_history(&history, 0, &mut rng);
    library.attention_budget = 1.0;
    library.attention_spent = 0.0;

    let result = skill_check(&library, ActionId::Build);
    assert!(result.can_execute);

    spend_attention(&mut library, result.attention_cost);

    assert!(library.attention_spent > 0.0, "Should have spent attention");
}
