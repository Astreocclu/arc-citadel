//! Experience-based chunk generation
//!
//! Entities accumulate life experiences that generate skill chunks.
//! A 45-year-old farmer has deeper farming chunks than a 20-year-old.
//! A soldier who saw combat has different chunks than one fresh from training.

use std::collections::HashMap;

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::skills::{ChunkId, ChunkLibrary, PersonalChunkState};

/// A period of an entity's life that generated skill chunks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifeExperience {
    /// What kind of activity
    pub activity: ActivityType,
    /// How long (in years)
    pub duration_years: f32,
    /// Intensity: full-time = 1.0, part-time = 0.5, casual = 0.2
    pub intensity: f32,
    /// Quality of training/environment (0.0 to 1.0)
    pub training_quality: f32,
}

/// Military unit types for training specialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    Infantry,
    Cavalry,
    Archer,
}

/// Current occupation (used to generate plausible history)
///
/// Role is a label for convenience. What matters is the generated history.
/// A Role::Farmer at age 50 has 40 years of farming experience.
/// A Role::Farmer at age 18 has 6 years.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Farmer,
    Miner,
    Craftsman(CraftSpecialty),
    Soldier,
    Guard,
    Noble,
    Merchant,
    Scholar,
    Priest,
    Servant,
    Child,
    Unemployed,
}

/// Craft specialization (kept from old system)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CraftSpecialty {
    Smithing,
    Carpentry,
    Masonry,
    Cooking,
    Tailoring,
    Leatherwork,
}

impl CraftSpecialty {
    /// Convert to the corresponding ActivityType
    pub fn to_activity(self) -> ActivityType {
        match self {
            Self::Smithing => ActivityType::Smithing,
            Self::Carpentry => ActivityType::Carpentry,
            Self::Masonry => ActivityType::Masonry,
            Self::Cooking => ActivityType::Cooking,
            Self::Tailoring => ActivityType::Tailoring,
            Self::Leatherwork => ActivityType::Leatherworking,
        }
    }
}

/// Activities that generate skill chunks over time
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActivityType {
    // === Physical Labor ===
    Farming,
    Mining,
    Construction,
    Hauling,

    // === Crafts ===
    Smithing,
    Carpentry,
    Masonry,
    Cooking,
    Tailoring,
    Leatherworking,
    Pottery,

    // === Combat ===
    MilitaryTraining { unit_type: UnitType },
    CombatExperience { battles_fought: u32 },
    GuardDuty,

    // === Social ===
    Trading,
    Diplomacy,
    CourtLife,
    PublicSpeaking,

    // === Leadership ===
    MilitaryCommand { soldiers_led: u32 },
    WorkforceManagement { workers_led: u32 },
    PoliticalOffice,

    // === Knowledge ===
    Literacy,
    FormalEducation,
    Apprenticeship { master_skill: f32 },
    Research,
    Teaching,

    // === Universal ===
    GeneralLife,
}

/// Calculate the contribution to encoding depth from one experience.
///
/// Uses logarithmic growth: fast early, slow later.
/// At full intensity/quality/growth (all 1.0):
/// 1 year -> ~0.13, 5 years -> ~0.43, 15 years -> ~0.69, 30 years -> ~0.82
pub fn calculate_experience_contribution(
    years: f32,
    intensity: f32,
    training_quality: f32,
    base_growth: f32,
) -> f32 {
    // Effective years = actual years * intensity
    let effective_years = years * intensity;

    // Logarithmic growth curve
    let base_depth = 1.0 - (1.0 / (1.0 + effective_years * 0.15));

    // Quality modifier: 0.5x to 1.0x based on training_quality
    let quality_mod = 0.5 + (training_quality * 0.5);

    (base_depth * base_growth * quality_mod).clamp(0.0, 0.95)
}

/// Combine two encoding depths with diminishing returns.
///
/// Two 0.5 experiences don't make 1.0, they make ~0.75.
pub fn combine_encoding(existing: f32, additional: f32) -> f32 {
    let combined = existing + additional * (1.0 - existing * 0.5);
    combined.clamp(0.0, 0.95)
}

/// Estimate repetition count from experience duration.
pub fn estimate_repetitions(years: f32, intensity: f32, _chunk_id: &ChunkId) -> u32 {
    // Assume ~100 meaningful reps per year at full intensity
    (years * intensity * 100.0) as u32
}

/// Constants for chunk generation
const FORMATION_TICK_MULTIPLIER: f32 = 10000.0;
const DEPTH_VARIANCE: f32 = 0.05;

/// Generate chunks from an entity's accumulated life experiences.
///
/// # Arguments
/// * `history` - The entity's life experiences
/// * `tick` - Current simulation tick
/// * `rng` - Random number generator for variance
///
/// # Returns
/// A ChunkLibrary with chunks derived from the history.
pub fn generate_chunks_from_history(
    history: &[LifeExperience],
    tick: u64,
    rng: &mut impl Rng,
) -> ChunkLibrary {
    let mut chunk_depths: HashMap<ChunkId, f32> = HashMap::new();
    let mut chunk_reps: HashMap<ChunkId, u32> = HashMap::new();

    // Process each life experience
    for experience in history {
        let chunk_contributions = get_chunks_for_activity(&experience.activity);

        for (chunk_id, base_growth_rate) in chunk_contributions {
            // Calculate encoding depth from this experience
            let contribution = calculate_experience_contribution(
                experience.duration_years,
                experience.intensity,
                experience.training_quality,
                base_growth_rate,
            );

            // Combine with existing depth (diminishing returns)
            let existing = *chunk_depths.get(&chunk_id).unwrap_or(&0.0);
            chunk_depths.insert(chunk_id, combine_encoding(existing, contribution));

            // Accumulate repetitions
            let reps = estimate_repetitions(
                experience.duration_years,
                experience.intensity,
                &chunk_id,
            );
            *chunk_reps.entry(chunk_id).or_insert(0) += reps;
        }
    }

    // Build the ChunkLibrary
    let mut library = ChunkLibrary::new();

    for (chunk_id, base_depth) in chunk_depths {
        // Apply variance for individual differences
        let variance = rng.gen_range(-DEPTH_VARIANCE..DEPTH_VARIANCE);
        let depth = (base_depth + variance).clamp(0.05, 0.95);

        let reps = chunk_reps.get(&chunk_id).copied().unwrap_or(0);

        library.set_chunk(
            chunk_id,
            PersonalChunkState {
                encoding_depth: depth,
                repetition_count: reps,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub((depth * FORMATION_TICK_MULTIPLIER) as u64),
            },
        );
    }

    library
}

/// Generate a plausible life history for an entity with given role and age.
///
/// This is a convenience function for mass spawning. For important NPCs,
/// construct history explicitly.
pub fn generate_history_for_role(
    role: Role,
    age: u32,
    rng: &mut impl Rng,
) -> Vec<LifeExperience> {
    let mut history = Vec::new();

    // Everyone has childhood (general life until age 12)
    let childhood_end = 12.min(age);
    if childhood_end > 0 {
        history.push(LifeExperience {
            activity: ActivityType::GeneralLife,
            duration_years: childhood_end as f32,
            intensity: 1.0,
            training_quality: 0.5 + rng.gen_range(-0.2..0.2),
        });
    }

    if age <= 12 {
        return history; // Just a child
    }

    // Role-specific history from age 12+
    match role {
        Role::Child => {
            // Already handled above
        }

        Role::Farmer => {
            let farming_years = (age - 10) as f32;
            history.push(LifeExperience {
                activity: ActivityType::Farming,
                duration_years: farming_years,
                intensity: 0.9 + rng.gen_range(-0.1..0.1),
                training_quality: 0.5 + rng.gen_range(-0.2..0.2),
            });
        }

        Role::Miner => {
            let mining_years = (age - 14) as f32;
            if mining_years > 0.0 {
                history.push(LifeExperience {
                    activity: ActivityType::Mining,
                    duration_years: mining_years,
                    intensity: 0.9,
                    training_quality: 0.4 + rng.gen_range(-0.1..0.1),
                });
            }
        }

        Role::Craftsman(specialty) => {
            // Some adolescent labor
            history.push(LifeExperience {
                activity: ActivityType::Hauling,
                duration_years: rng.gen_range(2.0_f32..5.0).min((age - 12) as f32),
                intensity: 0.7,
                training_quality: 0.5,
            });

            // Apprenticeship starts around 14
            let apprentice_years: f32 = rng.gen_range(4.0..8.0);
            let craft_start = 14;

            if age > craft_start {
                let years_in_craft = (age - craft_start) as f32;
                let training_years = apprentice_years.min(years_in_craft);
                let journeyman_years = (years_in_craft - training_years).max(0.0);

                // Apprenticeship (good training quality)
                if training_years > 0.0 {
                    history.push(LifeExperience {
                        activity: specialty.to_activity(),
                        duration_years: training_years,
                        intensity: 1.0,
                        training_quality: rng.gen_range(0.6..0.9),
                    });
                }

                // Journeyman/master years (self-directed)
                if journeyman_years > 0.0 {
                    history.push(LifeExperience {
                        activity: specialty.to_activity(),
                        duration_years: journeyman_years,
                        intensity: 1.0,
                        training_quality: 0.5 + rng.gen_range(-0.1..0.1),
                    });
                }
            }
        }

        Role::Soldier => {
            // Background before military
            let military_start = rng.gen_range(16..22);

            if age > 12 {
                // Pre-military labor
                let labor_years = ((military_start.min(age) - 12) as f32).max(0.0);
                if labor_years > 0.0 {
                    history.push(LifeExperience {
                        activity: if rng.gen_bool(0.7) {
                            ActivityType::Farming
                        } else {
                            ActivityType::Construction
                        },
                        duration_years: labor_years,
                        intensity: 0.8,
                        training_quality: 0.5,
                    });
                }
            }

            if age > military_start {
                let training_years: f32 = rng.gen_range(0.5..2.0);
                let service_years = (age - military_start) as f32;

                // Military training
                history.push(LifeExperience {
                    activity: ActivityType::MilitaryTraining {
                        unit_type: UnitType::Infantry,
                    },
                    duration_years: training_years.min(service_years),
                    intensity: 1.0,
                    training_quality: rng.gen_range(0.5..0.9),
                });

                // Combat experience (maybe)
                let post_training = service_years - training_years;
                if post_training > 0.0 {
                    let battles = (post_training * rng.gen_range(0.0..0.5)) as u32;
                    if battles > 0 {
                        history.push(LifeExperience {
                            activity: ActivityType::CombatExperience { battles_fought: battles },
                            duration_years: post_training,
                            intensity: 0.3, // Combat is intermittent
                            training_quality: 1.0,
                        });
                    }

                    // Guard duty between combat
                    history.push(LifeExperience {
                        activity: ActivityType::GuardDuty,
                        duration_years: post_training,
                        intensity: 0.6,
                        training_quality: 0.4,
                    });
                }
            }
        }

        Role::Guard => {
            // Similar to soldier but more guard duty, less combat
            let guard_start = rng.gen_range(18..25);

            if age > 12 {
                let labor_years = ((guard_start.min(age) - 12) as f32).max(0.0);
                if labor_years > 0.0 {
                    history.push(LifeExperience {
                        activity: ActivityType::Construction,
                        duration_years: labor_years,
                        intensity: 0.8,
                        training_quality: 0.5,
                    });
                }
            }

            if age > guard_start {
                history.push(LifeExperience {
                    activity: ActivityType::MilitaryTraining {
                        unit_type: UnitType::Infantry,
                    },
                    duration_years: rng.gen_range(0.3_f32..1.0),
                    intensity: 0.8,
                    training_quality: 0.5,
                });

                history.push(LifeExperience {
                    activity: ActivityType::GuardDuty,
                    duration_years: (age - guard_start) as f32,
                    intensity: 0.9,
                    training_quality: 0.4,
                });
            }
        }

        Role::Noble => {
            // Education starts early
            history.push(LifeExperience {
                activity: ActivityType::FormalEducation,
                duration_years: rng.gen_range(5.0_f32..10.0).min((age - 6) as f32),
                intensity: 0.8,
                training_quality: 0.8,
            });

            if age > 16 {
                // Court life
                history.push(LifeExperience {
                    activity: ActivityType::CourtLife,
                    duration_years: (age - 16) as f32,
                    intensity: 0.7,
                    training_quality: 0.6,
                });
            }

            // Maybe military training
            if rng.gen_bool(0.4) && age > 18 {
                history.push(LifeExperience {
                    activity: ActivityType::MilitaryTraining {
                        unit_type: if rng.gen_bool(0.6) {
                            UnitType::Cavalry
                        } else {
                            UnitType::Infantry
                        },
                    },
                    duration_years: rng.gen_range(1.0_f32..4.0),
                    intensity: 0.7,
                    training_quality: 0.8,
                });

                // Command experience if older
                if age > 25 {
                    history.push(LifeExperience {
                        activity: ActivityType::MilitaryCommand {
                            soldiers_led: rng.gen_range(20..200),
                        },
                        duration_years: (age - 25) as f32 * 0.3,
                        intensity: 0.5,
                        training_quality: 0.5,
                    });
                }
            }
        }

        Role::Merchant => {
            // Some education
            history.push(LifeExperience {
                activity: ActivityType::FormalEducation,
                duration_years: rng.gen_range(3.0_f32..6.0).min((age - 8) as f32),
                intensity: 0.7,
                training_quality: 0.6,
            });

            // Trading from adolescence
            if age > 14 {
                history.push(LifeExperience {
                    activity: ActivityType::Trading,
                    duration_years: (age - 14) as f32,
                    intensity: 0.9,
                    training_quality: 0.5 + rng.gen_range(-0.2..0.3),
                });
            }
        }

        Role::Scholar => {
            // Extensive education
            history.push(LifeExperience {
                activity: ActivityType::FormalEducation,
                duration_years: rng.gen_range(8.0_f32..15.0).min((age - 6) as f32),
                intensity: 0.9,
                training_quality: 0.7 + rng.gen_range(-0.1..0.2),
            });

            // Research if older
            if age > 25 {
                history.push(LifeExperience {
                    activity: ActivityType::Research,
                    duration_years: (age - 25) as f32,
                    intensity: 0.8,
                    training_quality: 0.6,
                });
            }

            // Maybe teaching
            if age > 30 && rng.gen_bool(0.5) {
                history.push(LifeExperience {
                    activity: ActivityType::Teaching,
                    duration_years: (age - 30) as f32 * 0.5,
                    intensity: 0.5,
                    training_quality: 0.5,
                });
            }
        }

        Role::Priest => {
            // Education
            history.push(LifeExperience {
                activity: ActivityType::FormalEducation,
                duration_years: rng.gen_range(5.0_f32..10.0).min((age - 8) as f32),
                intensity: 0.8,
                training_quality: 0.7,
            });

            // Public speaking/preaching
            if age > 20 {
                history.push(LifeExperience {
                    activity: ActivityType::PublicSpeaking,
                    duration_years: (age - 20) as f32,
                    intensity: 0.6,
                    training_quality: 0.5,
                });
            }
        }

        Role::Servant => {
            // General labor
            if age > 10 {
                history.push(LifeExperience {
                    activity: ActivityType::Hauling,
                    duration_years: (age - 10) as f32,
                    intensity: 0.8,
                    training_quality: 0.3,
                });
            }
        }

        Role::Unemployed => {
            // Just general life, no specialized activities
        }
    }

    history
}

/// Get chunks generated by an activity with their growth rates.
/// Growth rate affects how quickly this activity builds the chunk (0.0 to 1.0).
pub fn get_chunks_for_activity(activity: &ActivityType) -> Vec<(ChunkId, f32)> {
    match activity {
        ActivityType::GeneralLife => vec![
            (ChunkId::PhysEfficientGait, 1.0),
            (ChunkId::PhysQuietMovement, 0.5),
            (ChunkId::PhysPowerStance, 0.3),
        ],

        ActivityType::Farming => vec![
            (ChunkId::PhysSustainedLabor, 1.0),
            (ChunkId::PhysHeavyLifting, 0.7),
            (ChunkId::PhysRoughTerrainTravel, 0.5),
            (ChunkId::CraftBasicCut, 0.3),
            (ChunkId::SocialActiveListening, 0.2),
        ],

        ActivityType::Mining => vec![
            (ChunkId::PhysSustainedLabor, 1.0),
            (ChunkId::PhysHeavyLifting, 0.9),
            (ChunkId::CraftBasicCut, 0.6),
            (ChunkId::PhysPowerStance, 0.5),
        ],

        ActivityType::Construction => vec![
            (ChunkId::PhysSustainedLabor, 0.9),
            (ChunkId::PhysHeavyLifting, 0.8),
            (ChunkId::CraftBasicMeasure, 0.7),
            (ChunkId::CraftBasicCut, 0.6),
            (ChunkId::CraftBasicJoin, 0.5),
        ],

        ActivityType::Hauling => vec![
            (ChunkId::PhysSustainedLabor, 1.0),
            (ChunkId::PhysHeavyLifting, 1.0),
            (ChunkId::PhysPowerStance, 0.6),
        ],

        ActivityType::Smithing => vec![
            (ChunkId::CraftBasicHeatCycle, 1.0),
            (ChunkId::CraftBasicHammerWork, 1.0),
            (ChunkId::CraftDrawOutMetal, 0.8),
            (ChunkId::CraftUpsetMetal, 0.7),
            (ChunkId::CraftBasicWeld, 0.6),
            (ChunkId::CraftForgeKnife, 0.5),
            (ChunkId::CraftForgeToolHead, 0.4),
            (ChunkId::CraftForgeSword, 0.3),
            (ChunkId::PhysSustainedLabor, 0.4),
        ],

        ActivityType::Carpentry => vec![
            (ChunkId::CraftBasicMeasure, 1.0),
            (ChunkId::CraftBasicCut, 1.0),
            (ChunkId::CraftShapeWood, 0.9),
            (ChunkId::CraftBasicJoin, 0.8),
            (ChunkId::CraftFinishSurface, 0.6),
            (ChunkId::CraftBuildFurniture, 0.5),
            (ChunkId::CraftBuildStructure, 0.3),
        ],

        ActivityType::Masonry => vec![
            (ChunkId::CraftBasicMeasure, 0.9),
            (ChunkId::CraftBasicCut, 0.7),
            (ChunkId::PhysHeavyLifting, 0.8),
            (ChunkId::CraftBuildStructure, 0.5),
        ],

        ActivityType::Cooking => vec![
            (ChunkId::CraftBasicHeatCycle, 0.8),
            (ChunkId::CraftBasicCut, 0.9),
            (ChunkId::CraftBasicMeasure, 0.5),
        ],

        ActivityType::Tailoring => vec![
            (ChunkId::CraftBasicMeasure, 0.9),
            (ChunkId::CraftBasicCut, 0.8),
            (ChunkId::CraftSewGarment, 1.0),
            (ChunkId::CraftFinishSurface, 0.4),
        ],

        ActivityType::Leatherworking => vec![
            (ChunkId::CraftBasicMeasure, 0.7),
            (ChunkId::CraftBasicCut, 1.0),
            (ChunkId::CraftSewGarment, 0.6),
        ],

        ActivityType::Pottery => vec![
            (ChunkId::CraftBasicMeasure, 0.6),
            (ChunkId::CraftBasicHeatCycle, 0.7),
            (ChunkId::CraftFinishSurface, 0.8),
        ],

        ActivityType::MilitaryTraining { unit_type } => {
            let mut chunks = vec![
                (ChunkId::BasicStance, 1.0),
                (ChunkId::BasicSwing, 0.9),
                (ChunkId::BasicBlock, 0.9),
                (ChunkId::PhysDistanceRunning, 0.7),
                (ChunkId::PhysSustainedLabor, 0.4),
            ];

            match unit_type {
                UnitType::Infantry => {
                    chunks.push((ChunkId::AttackSequence, 0.5));
                    chunks.push((ChunkId::DefendSequence, 0.4));
                }
                UnitType::Cavalry => {
                    chunks.push((ChunkId::PhysHorseControl, 1.0));
                    chunks.push((ChunkId::PhysCavalryRiding, 0.6));
                    chunks.push((ChunkId::PhysMountedCombat, 0.4));
                }
                UnitType::Archer => {
                    chunks.push((ChunkId::DrawBow, 1.0));
                    chunks.push((ChunkId::BasicAim, 1.0));
                    chunks.push((ChunkId::LooseArrow, 0.8));
                    chunks.push((ChunkId::SnapShot, 0.5));
                }
            }

            chunks
        }

        ActivityType::CombatExperience { battles_fought } => {
            let intensity = (*battles_fought as f32 / 10.0).min(1.0);
            vec![
                (ChunkId::EngageMelee, intensity),
                (ChunkId::HandleFlanking, 0.6 * intensity),
                (ChunkId::Riposte, 0.7 * intensity),
            ]
        }

        ActivityType::GuardDuty => vec![
            (ChunkId::BasicStance, 0.5),
            (ChunkId::LeadSituationalRead, 0.4),
            (ChunkId::SocialProjectAuthority, 0.3),
        ],

        ActivityType::Trading => vec![
            (ChunkId::SocialBuildRapport, 0.9),
            (ChunkId::SocialReadReaction, 1.0),
            (ChunkId::SocialNegotiateTerms, 1.0),
            (ChunkId::KnowArithmetic, 0.5),
            (ChunkId::SocialDeceive, 0.3),
        ],

        ActivityType::Diplomacy => vec![
            (ChunkId::SocialBuildRapport, 1.0),
            (ChunkId::SocialReadReaction, 0.9),
            (ChunkId::SocialNegotiateTerms, 0.8),
            (ChunkId::SocialPersuade, 0.7),
            (ChunkId::SocialMediateConflict, 0.5),
        ],

        ActivityType::CourtLife => vec![
            (ChunkId::SocialProjectConfidence, 1.0),
            (ChunkId::SocialReadReaction, 0.9),
            (ChunkId::SocialBuildRapport, 0.8),
            (ChunkId::SocialDeflectInquiry, 0.6),
            (ChunkId::SocialPoliticalManeuver, 0.4),
            (ChunkId::SocialDeceive, 0.3),
        ],

        ActivityType::PublicSpeaking => vec![
            (ChunkId::SocialProjectConfidence, 1.0),
            (ChunkId::SocialInspire, 0.7),
            (ChunkId::SocialPersuade, 0.6),
            (ChunkId::SocialEmotionalAppeal, 0.5),
        ],

        ActivityType::MilitaryCommand { soldiers_led } => {
            let scale = (*soldiers_led as f32).max(1.0).log10() / 4.0;
            let scale = scale.clamp(0.1, 1.0);
            vec![
                (ChunkId::LeadCommandPresence, scale),
                (ChunkId::LeadIssueCommand, scale),
                (ChunkId::LeadAssessUnitState, 0.7 * scale),
                (ChunkId::LeadCoordinateUnits, 0.5 * scale),
                (ChunkId::LeadBattleManagement, 0.3 * scale),
            ]
        }

        ActivityType::WorkforceManagement { workers_led } => {
            let scale = (*workers_led as f32).max(1.0).log10() / 3.0;
            let scale = scale.clamp(0.1, 1.0);
            vec![
                (ChunkId::LeadDelegateTask, scale),
                (ChunkId::SocialProjectAuthority, 0.7 * scale),
                (ChunkId::LeadAssessUnitState, 0.5 * scale),
            ]
        }

        ActivityType::PoliticalOffice => vec![
            (ChunkId::SocialPoliticalManeuver, 1.0),
            (ChunkId::SocialWorkRoom, 0.8),
            (ChunkId::SocialLeadGroup, 0.7),
            (ChunkId::LeadDelegateTask, 0.6),
        ],

        ActivityType::Literacy => vec![
            (ChunkId::KnowFluentReading, 1.0),
            (ChunkId::KnowFluentWriting, 0.8),
        ],

        ActivityType::FormalEducation => vec![
            (ChunkId::KnowFluentReading, 1.0),
            (ChunkId::KnowFluentWriting, 0.9),
            (ChunkId::KnowMemorization, 0.8),
            (ChunkId::KnowArithmetic, 0.6),
            (ChunkId::KnowResearchSource, 0.3),
        ],

        ActivityType::Apprenticeship { master_skill: _ } => vec![
            (ChunkId::SocialActiveListening, 0.8),
        ],

        ActivityType::Research => vec![
            (ChunkId::KnowResearchSource, 1.0),
            (ChunkId::KnowAnalyzeText, 0.8),
            (ChunkId::KnowSynthesizeSources, 0.6),
            (ChunkId::KnowOriginalResearch, 0.4),
        ],

        ActivityType::Teaching => vec![
            (ChunkId::KnowTeachConcept, 1.0),
            (ChunkId::KnowInstructStudent, 0.7),
            (ChunkId::SocialReadReaction, 0.5),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::ChunkId;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_life_experience_creation() {
        let exp = LifeExperience {
            activity: ActivityType::Farming,
            duration_years: 10.0,
            intensity: 0.9,
            training_quality: 0.5,
        };
        assert_eq!(exp.duration_years, 10.0);
        assert_eq!(exp.intensity, 0.9);
    }

    #[test]
    fn test_military_training_variants() {
        let infantry = ActivityType::MilitaryTraining { unit_type: UnitType::Infantry };
        let cavalry = ActivityType::MilitaryTraining { unit_type: UnitType::Cavalry };
        assert_ne!(infantry, cavalry);
    }

    #[test]
    fn test_craft_specialty_to_activity() {
        assert_eq!(
            CraftSpecialty::Smithing.to_activity(),
            ActivityType::Smithing
        );
        assert_eq!(
            CraftSpecialty::Carpentry.to_activity(),
            ActivityType::Carpentry
        );
    }

    #[test]
    fn test_role_with_specialty() {
        let smith = Role::Craftsman(CraftSpecialty::Smithing);
        let carpenter = Role::Craftsman(CraftSpecialty::Carpentry);
        assert_ne!(smith, carpenter);
    }

    #[test]
    fn test_farming_produces_labor_chunks() {
        let chunks = get_chunks_for_activity(&ActivityType::Farming);
        assert!(chunks.iter().any(|(id, _)| *id == ChunkId::PhysSustainedLabor));
        assert!(chunks.iter().any(|(id, _)| *id == ChunkId::PhysHeavyLifting));
    }

    #[test]
    fn test_smithing_produces_craft_chunks() {
        let chunks = get_chunks_for_activity(&ActivityType::Smithing);
        assert!(chunks.iter().any(|(id, _)| *id == ChunkId::CraftBasicHeatCycle));
        assert!(chunks.iter().any(|(id, _)| *id == ChunkId::CraftBasicHammerWork));
    }

    #[test]
    fn test_military_training_varies_by_unit_type() {
        let infantry = get_chunks_for_activity(&ActivityType::MilitaryTraining {
            unit_type: UnitType::Infantry
        });
        let cavalry = get_chunks_for_activity(&ActivityType::MilitaryTraining {
            unit_type: UnitType::Cavalry
        });

        assert!(infantry.iter().any(|(id, _)| *id == ChunkId::AttackSequence));
        assert!(cavalry.iter().any(|(id, _)| *id == ChunkId::PhysHorseControl));
        assert!(!infantry.iter().any(|(id, _)| *id == ChunkId::PhysHorseControl));
    }

    #[test]
    fn test_combat_experience_scales_with_battles() {
        let few_battles = get_chunks_for_activity(&ActivityType::CombatExperience {
            battles_fought: 2
        });
        let many_battles = get_chunks_for_activity(&ActivityType::CombatExperience {
            battles_fought: 20
        });

        let few_rate = few_battles.iter()
            .find(|(id, _)| *id == ChunkId::EngageMelee)
            .map(|(_, r)| *r).unwrap();
        let many_rate = many_battles.iter()
            .find(|(id, _)| *id == ChunkId::EngageMelee)
            .map(|(_, r)| *r).unwrap();

        assert!(many_rate > few_rate);
    }

    #[test]
    fn test_experience_contribution_scales_with_years() {
        let one_year = calculate_experience_contribution(1.0, 1.0, 0.5, 1.0);
        let five_years = calculate_experience_contribution(5.0, 1.0, 0.5, 1.0);
        let fifteen_years = calculate_experience_contribution(15.0, 1.0, 0.5, 1.0);

        assert!(one_year < five_years);
        assert!(five_years < fifteen_years);
        assert!(fifteen_years < 0.95); // Never reaches max
    }

    #[test]
    fn test_intensity_affects_contribution() {
        let full_time = calculate_experience_contribution(10.0, 1.0, 0.5, 1.0);
        let part_time = calculate_experience_contribution(10.0, 0.5, 0.5, 1.0);

        assert!(full_time > part_time);
    }

    #[test]
    fn test_training_quality_affects_contribution() {
        let good_training = calculate_experience_contribution(10.0, 1.0, 1.0, 1.0);
        let poor_training = calculate_experience_contribution(10.0, 1.0, 0.0, 1.0);

        assert!(good_training > poor_training);
    }

    #[test]
    fn test_combine_encoding_diminishing_returns() {
        let combined = combine_encoding(0.5, 0.5);

        // Should be less than 1.0 due to diminishing returns
        assert!(combined < 1.0);
        assert!(combined > 0.5); // But more than either alone
    }

    #[test]
    fn test_combine_encoding_caps_at_095() {
        let combined = combine_encoding(0.9, 0.9);
        assert!(combined <= 0.95);
    }

    #[test]
    fn test_generate_chunks_from_farming_history() {
        let history = vec![
            LifeExperience {
                activity: ActivityType::GeneralLife,
                duration_years: 12.0,
                intensity: 1.0,
                training_quality: 0.5,
            },
            LifeExperience {
                activity: ActivityType::Farming,
                duration_years: 30.0,
                intensity: 0.9,
                training_quality: 0.5,
            },
        ];

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let library = generate_chunks_from_history(&history, 1000, &mut rng);

        // Should have farming chunks
        assert!(library.has_chunk(ChunkId::PhysSustainedLabor));
        assert!(library.has_chunk(ChunkId::PhysHeavyLifting));

        // Should have general life chunks
        assert!(library.has_chunk(ChunkId::PhysEfficientGait));
    }

    #[test]
    fn test_longer_experience_deeper_chunks() {
        let short_history = vec![
            LifeExperience {
                activity: ActivityType::Smithing,
                duration_years: 2.0,
                intensity: 1.0,
                training_quality: 0.7,
            },
        ];

        let long_history = vec![
            LifeExperience {
                activity: ActivityType::Smithing,
                duration_years: 20.0,
                intensity: 1.0,
                training_quality: 0.7,
            },
        ];

        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);

        let short_chunks = generate_chunks_from_history(&short_history, 0, &mut rng1);
        let long_chunks = generate_chunks_from_history(&long_history, 0, &mut rng2);

        let short_depth = short_chunks.get_chunk(ChunkId::CraftBasicHammerWork)
            .unwrap().encoding_depth;
        let long_depth = long_chunks.get_chunk(ChunkId::CraftBasicHammerWork)
            .unwrap().encoding_depth;

        assert!(long_depth > short_depth + 0.1);
    }

    #[test]
    fn test_mixed_history_produces_mixed_skills() {
        let history = vec![
            LifeExperience {
                activity: ActivityType::Farming,
                duration_years: 10.0,
                intensity: 1.0,
                training_quality: 0.5,
            },
            LifeExperience {
                activity: ActivityType::MilitaryTraining { unit_type: UnitType::Infantry },
                duration_years: 3.0,
                intensity: 1.0,
                training_quality: 0.7,
            },
        ];

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let library = generate_chunks_from_history(&history, 0, &mut rng);

        // Should have BOTH farming and combat chunks
        assert!(library.has_chunk(ChunkId::PhysSustainedLabor));
        assert!(library.has_chunk(ChunkId::BasicStance));
    }

    #[test]
    fn test_generate_history_for_farmer() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let history = generate_history_for_role(Role::Farmer, 30, &mut rng);

        // Should have general life and farming
        assert!(history.iter().any(|e| matches!(e.activity, ActivityType::GeneralLife)));
        assert!(history.iter().any(|e| matches!(e.activity, ActivityType::Farming)));
    }

    #[test]
    fn test_generate_history_for_soldier() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let history = generate_history_for_role(Role::Soldier, 30, &mut rng);

        // Should have military training
        assert!(history.iter().any(|e| matches!(
            e.activity,
            ActivityType::MilitaryTraining { .. }
        )));
    }

    #[test]
    fn test_child_has_minimal_history() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let history = generate_history_for_role(Role::Child, 8, &mut rng);

        // Only general life
        assert_eq!(history.len(), 1);
        assert!(matches!(history[0].activity, ActivityType::GeneralLife));
    }

    #[test]
    fn test_same_role_different_ages() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);

        let young_history = generate_history_for_role(Role::Farmer, 20, &mut rng1);
        let old_history = generate_history_for_role(Role::Farmer, 50, &mut rng2);

        let young_chunks = generate_chunks_from_history(&young_history, 0, &mut rng1);
        let old_chunks = generate_chunks_from_history(&old_history, 0, &mut rng2);

        let young_farming = young_chunks.get_chunk(ChunkId::PhysSustainedLabor)
            .unwrap().encoding_depth;
        let old_farming = old_chunks.get_chunk(ChunkId::PhysSustainedLabor)
            .unwrap().encoding_depth;

        assert!(old_farming > young_farming);
    }
}
