//! Chunk identifiers for the hierarchical skill system

use serde::{Deserialize, Serialize};

use crate::skills::ChunkDomain;

/// Unique identifier for a skill chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkId {
    // === MELEE ===
    // Level 1 - Micro-chunks (first learning)
    BasicSwing,
    BasicBlock,
    BasicStance,

    // Level 2 - Action chunks (competent soldier)
    AttackSequence,
    DefendSequence,
    Riposte,

    // Level 3 - Tactical chunks (veteran)
    EngageMelee,
    HandleFlanking,

    // === RANGED ===
    // Level 1 - Micro-chunks
    DrawBow,      // Physical act of drawing bowstring
    LoadCrossbow, // Spanning/winding crossbow mechanism
    BasicAim,     // Visual focus on target
    BasicThrow,   // Throwing motion fundamentals

    // Level 2 - Action chunks
    LooseArrow,   // Draw + Aim + Release (standard bow shot)
    CrossbowShot, // Aim + Trigger (crossbow shot when loaded)
    AimedThrow,   // Aim + Throw (accurate thrown weapon)
    SnapShot,     // Quick bow shot, less accurate

    // Level 3 - Tactical chunks
    RapidFire,   // Multiple arrows in quick succession
    SniperShot,  // Maximum precision, high cost
    VolleyFire,  // Coordinated area fire
    PartingShot, // Fire while retreating (horse archers)

    // === CRAFT DOMAIN ===
    // Level 1 - Micro-chunks
    CraftBasicHeatCycle,  // Heat metal to working temperature
    CraftBasicHammerWork, // Basic hammer strikes for shaping
    CraftBasicMeasure,    // Measure and mark materials
    CraftBasicCut,        // Cut materials (wood, leather, cloth)
    CraftBasicJoin,       // Join pieces (nails, stitches, glue)

    // Level 2 - Technique chunks
    CraftDrawOutMetal,  // Lengthen and thin metal through hammering
    CraftUpsetMetal,    // Thicken and shorten metal
    CraftBasicWeld,     // Forge-weld two pieces of metal
    CraftShapeWood,     // Shape wood through carving/planing
    CraftFinishSurface, // Sand, polish, apply finish

    // Level 3 - Product chunks
    CraftForgeKnife,     // Create a basic knife
    CraftForgeToolHead,  // Create tool heads (axe, hammer, etc.)
    CraftBuildFurniture, // Build basic furniture
    CraftSewGarment,     // Sew a complete garment

    // Level 4 - Complex product chunks
    CraftForgeSword,     // Create a sword
    CraftForgeArmor,     // Create armor pieces
    CraftBuildStructure, // Build structural elements
    CraftPatternWeld,    // Create pattern-welded steel

    // Level 5 - Mastery chunks
    CraftAssessAndExecute,    // Assess problem, choose approach, execute
    CraftForgeMasterwork,     // Create masterwork quality items
    CraftInnovativeTechnique, // Develop new techniques

    // === SOCIAL DOMAIN ===
    // Level 1 - Micro-chunks
    SocialActiveListening,   // Focus on speaker, absorb content
    SocialProjectConfidence, // Body language, voice tone, presence
    SocialEmpathicMirror,    // Match emotional state, build connection
    SocialCreateTension,     // Introduce discomfort, silence, pressure

    // Level 2 - Technique chunks
    SocialBuildRapport,     // Establish trust and common ground
    SocialProjectAuthority, // Command presence and respect
    SocialReadReaction,     // Assess response, adjust approach
    SocialDeflectInquiry,   // Redirect questions, avoid commitments
    SocialEmotionalAppeal,  // Invoke emotions to persuade

    // Level 3 - Tactical chunks
    SocialNegotiateTerms, // Reach mutually acceptable agreements
    SocialIntimidate,     // Apply pressure through fear
    SocialPersuade,       // Change minds through argument
    SocialDeceive,        // Mislead while appearing truthful
    SocialInspire,        // Motivate through vision and charisma

    // Level 4 - Strategic chunks
    SocialWorkRoom,          // Manage multiple relationships at once
    SocialPoliticalManeuver, // Navigate power structures
    SocialLeadGroup,         // Guide collective decision-making
    SocialMediateConflict,   // Resolve disputes between parties

    // Level 5 - Mastery chunks
    SocialOmniscience,        // Read entire room's dynamics instantly
    SocialManipulateDynamics, // Shape group behavior subtly
    SocialCultOfPersonality,  // Build devoted following

    // === MEDICINE DOMAIN ===
    // Level 1 - Micro-chunks
    MedWoundAssessment, // Evaluate wound severity and type
    MedBasicCleaning,   // Clean wounds to prevent infection
    MedBasicSuture,     // Close wounds with needle and thread
    MedVitalCheck,      // Assess pulse, breathing, consciousness

    // Level 2 - Technique chunks
    MedTreatLaceration, // Complete laceration treatment
    MedSetFracture,     // Align and immobilize broken bones
    MedPreparePoultice, // Create herbal healing preparations
    MedDiagnoseIllness, // Identify illness from symptoms
    MedPainManagement,  // Administer pain relief techniques

    // Level 3 - Treatment chunks
    MedFieldSurgery,   // Perform surgery in non-ideal conditions
    MedTreatInfection, // Combat established infections
    MedDeliverBaby,    // Assist with childbirth
    MedAmputation,     // Remove damaged limbs to save patient

    // Level 4 - Complex treatment chunks
    MedBattlefieldTriage, // Rapidly prioritize multiple casualties
    MedComplexSurgery,    // Major internal surgery
    MedEpidemicResponse,  // Manage disease outbreaks

    // Level 5 - Mastery chunks
    MedDiagnosticIntuition, // Instantly recognize obscure conditions
    MedSurgicalExcellence,  // Perfect surgical technique
    MedHolisticTreatment,   // Treat body, mind, and spirit together

    // === LEADERSHIP DOMAIN ===
    // Level 1 - Micro-chunks
    LeadCommandPresence, // Project authority through bearing
    LeadClearOrder,      // Articulate unambiguous commands
    LeadSituationalRead, // Quickly assess tactical situation

    // Level 2 - Technique chunks
    LeadIssueCommand,    // Deliver orders with proper timing
    LeadAssessUnitState, // Evaluate unit morale and capability
    LeadDelegateTask,    // Assign tasks to appropriate subordinates
    LeadMaintainCalm,    // Stay composed under pressure

    // Level 3 - Tactical chunks
    LeadDirectFormation, // Guide unit positioning and movement
    LeadRespondToCrisis, // React decisively to sudden changes
    LeadRallyWavering,   // Restore morale to shaken troops
    LeadCoordinateUnits, // Synchronize multiple units' actions

    // Level 4 - Strategic chunks
    LeadBattleManagement,     // Orchestrate entire battle
    LeadCampaignPlanning,     // Plan long-term military operations
    LeadOrganizationBuilding, // Build and maintain command structure

    // Level 5 - Mastery chunks
    LeadReadBattleFlow,     // Intuitive grasp of battle dynamics
    LeadInspireArmy,        // Motivate entire force through presence
    LeadStrategicIntuition, // Instant recognition of strategic opportunity

    // === PHYSICAL DOMAIN ===
    // Level 1 - Micro-chunks
    PhysEfficientGait, // Energy-efficient walking form
    PhysQuietMovement, // Move without making noise
    PhysPowerStance,   // Leverage body weight for lifting
    PhysClimbGrip,     // Grip technique for climbing

    // Level 2 - Technique chunks
    PhysDistanceRunning, // Sustained running pace
    PhysHeavyLifting,    // Lift and carry heavy loads
    PhysSilentApproach,  // Approach targets undetected
    PhysRockClimbing,    // Scale rock faces and walls
    PhysHorseControl,    // Basic mounted movement

    // Level 3 - Application chunks
    PhysSustainedLabor,     // Work for extended periods
    PhysInfiltration,       // Move through guarded areas
    PhysRoughTerrainTravel, // Navigate difficult terrain
    PhysCavalryRiding,      // Combat-ready mounted movement
    PhysSwimming,           // Swim in various conditions

    // Level 4 - Expert chunks
    PhysLaborLeadership, // Organize and lead work crews
    PhysScoutMission,    // Extended reconnaissance operations
    PhysMountedCombat,   // Fight effectively while mounted
    PhysSurvivalTravel,  // Travel through hostile environments

    // Level 5 - Mastery chunks
    PhysTirelessEndurance, // Extreme sustained physical output
    PhysShadowMovement,    // Near-invisible movement
    PhysCentaurUnity,      // Rider-mount perfect fusion

    // === KNOWLEDGE DOMAIN ===
    // Level 1 - Micro-chunks
    KnowFluentReading, // Read text smoothly with comprehension
    KnowFluentWriting, // Write text clearly and legibly
    KnowArithmetic,    // Basic mathematical operations
    KnowMemorization,  // Commit information to memory

    // Level 2 - Technique chunks
    KnowResearchSource,    // Find and evaluate sources
    KnowComposeDocument,   // Write formal documents
    KnowMathematicalProof, // Construct logical proofs
    KnowTeachConcept,      // Explain ideas to learners
    KnowTranslateText,     // Convert between languages

    // Level 3 - Application chunks
    KnowAnalyzeText,       // Deep textual analysis
    KnowSynthesizeSources, // Combine multiple sources
    KnowFormalArgument,    // Construct rigorous arguments
    KnowInstructStudent,   // Guide student development

    // Level 4 - Expert chunks
    KnowOriginalResearch,      // Conduct novel research
    KnowComprehensiveTreatise, // Write comprehensive works
    KnowCurriculumDesign,      // Design educational programs

    // Level 5 - Mastery chunks
    KnowParadigmIntegration, // Integrate multiple paradigms
    KnowIntellectualLegacy,  // Create lasting intellectual contributions
}

impl ChunkId {
    /// Get the domain this chunk belongs to
    pub const fn domain(&self) -> ChunkDomain {
        match self {
            // Combat domain - all melee and ranged chunks
            Self::BasicSwing
            | Self::BasicBlock
            | Self::BasicStance
            | Self::AttackSequence
            | Self::DefendSequence
            | Self::Riposte
            | Self::EngageMelee
            | Self::HandleFlanking
            | Self::DrawBow
            | Self::LoadCrossbow
            | Self::BasicAim
            | Self::BasicThrow
            | Self::LooseArrow
            | Self::CrossbowShot
            | Self::AimedThrow
            | Self::SnapShot
            | Self::RapidFire
            | Self::SniperShot
            | Self::VolleyFire
            | Self::PartingShot => ChunkDomain::Combat,

            // Craft domain - all crafting chunks
            Self::CraftBasicHeatCycle
            | Self::CraftBasicHammerWork
            | Self::CraftBasicMeasure
            | Self::CraftBasicCut
            | Self::CraftBasicJoin
            | Self::CraftDrawOutMetal
            | Self::CraftUpsetMetal
            | Self::CraftBasicWeld
            | Self::CraftShapeWood
            | Self::CraftFinishSurface
            | Self::CraftForgeKnife
            | Self::CraftForgeToolHead
            | Self::CraftBuildFurniture
            | Self::CraftSewGarment
            | Self::CraftForgeSword
            | Self::CraftForgeArmor
            | Self::CraftBuildStructure
            | Self::CraftPatternWeld
            | Self::CraftAssessAndExecute
            | Self::CraftForgeMasterwork
            | Self::CraftInnovativeTechnique => ChunkDomain::Craft,

            // Social domain - all social interaction chunks
            Self::SocialActiveListening
            | Self::SocialProjectConfidence
            | Self::SocialEmpathicMirror
            | Self::SocialCreateTension
            | Self::SocialBuildRapport
            | Self::SocialProjectAuthority
            | Self::SocialReadReaction
            | Self::SocialDeflectInquiry
            | Self::SocialEmotionalAppeal
            | Self::SocialNegotiateTerms
            | Self::SocialIntimidate
            | Self::SocialPersuade
            | Self::SocialDeceive
            | Self::SocialInspire
            | Self::SocialWorkRoom
            | Self::SocialPoliticalManeuver
            | Self::SocialLeadGroup
            | Self::SocialMediateConflict
            | Self::SocialOmniscience
            | Self::SocialManipulateDynamics
            | Self::SocialCultOfPersonality => ChunkDomain::Social,

            // Medicine domain - all medical treatment chunks
            Self::MedWoundAssessment
            | Self::MedBasicCleaning
            | Self::MedBasicSuture
            | Self::MedVitalCheck
            | Self::MedTreatLaceration
            | Self::MedSetFracture
            | Self::MedPreparePoultice
            | Self::MedDiagnoseIllness
            | Self::MedPainManagement
            | Self::MedFieldSurgery
            | Self::MedTreatInfection
            | Self::MedDeliverBaby
            | Self::MedAmputation
            | Self::MedBattlefieldTriage
            | Self::MedComplexSurgery
            | Self::MedEpidemicResponse
            | Self::MedDiagnosticIntuition
            | Self::MedSurgicalExcellence
            | Self::MedHolisticTreatment => ChunkDomain::Medicine,

            // Leadership domain - all command and tactics chunks
            Self::LeadCommandPresence
            | Self::LeadClearOrder
            | Self::LeadSituationalRead
            | Self::LeadIssueCommand
            | Self::LeadAssessUnitState
            | Self::LeadDelegateTask
            | Self::LeadMaintainCalm
            | Self::LeadDirectFormation
            | Self::LeadRespondToCrisis
            | Self::LeadRallyWavering
            | Self::LeadCoordinateUnits
            | Self::LeadBattleManagement
            | Self::LeadCampaignPlanning
            | Self::LeadOrganizationBuilding
            | Self::LeadReadBattleFlow
            | Self::LeadInspireArmy
            | Self::LeadStrategicIntuition => ChunkDomain::Leadership,

            // Knowledge domain - all scholarly and academic chunks
            Self::KnowFluentReading
            | Self::KnowFluentWriting
            | Self::KnowArithmetic
            | Self::KnowMemorization
            | Self::KnowResearchSource
            | Self::KnowComposeDocument
            | Self::KnowMathematicalProof
            | Self::KnowTeachConcept
            | Self::KnowTranslateText
            | Self::KnowAnalyzeText
            | Self::KnowSynthesizeSources
            | Self::KnowFormalArgument
            | Self::KnowInstructStudent
            | Self::KnowOriginalResearch
            | Self::KnowComprehensiveTreatise
            | Self::KnowCurriculumDesign
            | Self::KnowParadigmIntegration
            | Self::KnowIntellectualLegacy => ChunkDomain::Knowledge,

            // Physical domain - athletics, stealth, climbing, mounted
            Self::PhysEfficientGait
            | Self::PhysQuietMovement
            | Self::PhysPowerStance
            | Self::PhysClimbGrip
            | Self::PhysDistanceRunning
            | Self::PhysHeavyLifting
            | Self::PhysSilentApproach
            | Self::PhysRockClimbing
            | Self::PhysHorseControl
            | Self::PhysSustainedLabor
            | Self::PhysInfiltration
            | Self::PhysRoughTerrainTravel
            | Self::PhysCavalryRiding
            | Self::PhysSwimming
            | Self::PhysLaborLeadership
            | Self::PhysScoutMission
            | Self::PhysMountedCombat
            | Self::PhysSurvivalTravel
            | Self::PhysTirelessEndurance
            | Self::PhysShadowMovement
            | Self::PhysCentaurUnity => ChunkDomain::Physical,
        }
    }

    /// Get the hierarchy level of this chunk (1-5)
    pub fn level(&self) -> u8 {
        match self {
            // Melee Level 1
            Self::BasicSwing | Self::BasicBlock | Self::BasicStance => 1,
            // Ranged Level 1
            Self::DrawBow | Self::LoadCrossbow | Self::BasicAim | Self::BasicThrow => 1,

            // Melee Level 2
            Self::AttackSequence | Self::DefendSequence | Self::Riposte => 2,
            // Ranged Level 2
            Self::LooseArrow | Self::CrossbowShot | Self::AimedThrow | Self::SnapShot => 2,

            // Melee Level 3
            Self::EngageMelee | Self::HandleFlanking => 3,
            // Ranged Level 3
            Self::RapidFire | Self::SniperShot | Self::VolleyFire | Self::PartingShot => 3,

            // Craft Level 1
            Self::CraftBasicHeatCycle
            | Self::CraftBasicHammerWork
            | Self::CraftBasicMeasure
            | Self::CraftBasicCut
            | Self::CraftBasicJoin => 1,

            // Craft Level 2
            Self::CraftDrawOutMetal
            | Self::CraftUpsetMetal
            | Self::CraftBasicWeld
            | Self::CraftShapeWood
            | Self::CraftFinishSurface => 2,

            // Craft Level 3
            Self::CraftForgeKnife
            | Self::CraftForgeToolHead
            | Self::CraftBuildFurniture
            | Self::CraftSewGarment => 3,

            // Craft Level 4
            Self::CraftForgeSword
            | Self::CraftForgeArmor
            | Self::CraftBuildStructure
            | Self::CraftPatternWeld => 4,

            // Craft Level 5
            Self::CraftAssessAndExecute
            | Self::CraftForgeMasterwork
            | Self::CraftInnovativeTechnique => 5,

            // Social Level 1
            Self::SocialActiveListening
            | Self::SocialProjectConfidence
            | Self::SocialEmpathicMirror
            | Self::SocialCreateTension => 1,

            // Social Level 2
            Self::SocialBuildRapport
            | Self::SocialProjectAuthority
            | Self::SocialReadReaction
            | Self::SocialDeflectInquiry
            | Self::SocialEmotionalAppeal => 2,

            // Social Level 3
            Self::SocialNegotiateTerms
            | Self::SocialIntimidate
            | Self::SocialPersuade
            | Self::SocialDeceive
            | Self::SocialInspire => 3,

            // Social Level 4
            Self::SocialWorkRoom
            | Self::SocialPoliticalManeuver
            | Self::SocialLeadGroup
            | Self::SocialMediateConflict => 4,

            // Social Level 5
            Self::SocialOmniscience
            | Self::SocialManipulateDynamics
            | Self::SocialCultOfPersonality => 5,

            // Medicine Level 1
            Self::MedWoundAssessment
            | Self::MedBasicCleaning
            | Self::MedBasicSuture
            | Self::MedVitalCheck => 1,

            // Medicine Level 2
            Self::MedTreatLaceration
            | Self::MedSetFracture
            | Self::MedPreparePoultice
            | Self::MedDiagnoseIllness
            | Self::MedPainManagement => 2,

            // Medicine Level 3
            Self::MedFieldSurgery
            | Self::MedTreatInfection
            | Self::MedDeliverBaby
            | Self::MedAmputation => 3,

            // Medicine Level 4
            Self::MedBattlefieldTriage | Self::MedComplexSurgery | Self::MedEpidemicResponse => 4,

            // Medicine Level 5
            Self::MedDiagnosticIntuition
            | Self::MedSurgicalExcellence
            | Self::MedHolisticTreatment => 5,

            // Leadership Level 1
            Self::LeadCommandPresence | Self::LeadClearOrder | Self::LeadSituationalRead => 1,

            // Leadership Level 2
            Self::LeadIssueCommand
            | Self::LeadAssessUnitState
            | Self::LeadDelegateTask
            | Self::LeadMaintainCalm => 2,

            // Leadership Level 3
            Self::LeadDirectFormation
            | Self::LeadRespondToCrisis
            | Self::LeadRallyWavering
            | Self::LeadCoordinateUnits => 3,

            // Leadership Level 4
            Self::LeadBattleManagement
            | Self::LeadCampaignPlanning
            | Self::LeadOrganizationBuilding => 4,

            // Leadership Level 5
            Self::LeadReadBattleFlow | Self::LeadInspireArmy | Self::LeadStrategicIntuition => 5,

            // Knowledge Level 1
            Self::KnowFluentReading
            | Self::KnowFluentWriting
            | Self::KnowArithmetic
            | Self::KnowMemorization => 1,

            // Knowledge Level 2
            Self::KnowResearchSource
            | Self::KnowComposeDocument
            | Self::KnowMathematicalProof
            | Self::KnowTeachConcept
            | Self::KnowTranslateText => 2,

            // Knowledge Level 3
            Self::KnowAnalyzeText
            | Self::KnowSynthesizeSources
            | Self::KnowFormalArgument
            | Self::KnowInstructStudent => 3,

            // Knowledge Level 4
            Self::KnowOriginalResearch
            | Self::KnowComprehensiveTreatise
            | Self::KnowCurriculumDesign => 4,

            // Knowledge Level 5
            Self::KnowParadigmIntegration | Self::KnowIntellectualLegacy => 5,

            // Physical Level 1
            Self::PhysEfficientGait
            | Self::PhysQuietMovement
            | Self::PhysPowerStance
            | Self::PhysClimbGrip => 1,

            // Physical Level 2
            Self::PhysDistanceRunning
            | Self::PhysHeavyLifting
            | Self::PhysSilentApproach
            | Self::PhysRockClimbing
            | Self::PhysHorseControl => 2,

            // Physical Level 3
            Self::PhysSustainedLabor
            | Self::PhysInfiltration
            | Self::PhysRoughTerrainTravel
            | Self::PhysCavalryRiding
            | Self::PhysSwimming => 3,

            // Physical Level 4
            Self::PhysLaborLeadership
            | Self::PhysScoutMission
            | Self::PhysMountedCombat
            | Self::PhysSurvivalTravel => 4,

            // Physical Level 5
            Self::PhysTirelessEndurance | Self::PhysShadowMovement | Self::PhysCentaurUnity => 5,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            // Melee
            Self::BasicSwing => "Basic Swing",
            Self::BasicBlock => "Basic Block",
            Self::BasicStance => "Basic Stance",
            Self::AttackSequence => "Attack Sequence",
            Self::DefendSequence => "Defend Sequence",
            Self::Riposte => "Riposte",
            Self::EngageMelee => "Engage Melee",
            Self::HandleFlanking => "Handle Flanking",
            // Ranged
            Self::DrawBow => "Draw Bow",
            Self::LoadCrossbow => "Load Crossbow",
            Self::BasicAim => "Basic Aim",
            Self::BasicThrow => "Basic Throw",
            Self::LooseArrow => "Loose Arrow",
            Self::CrossbowShot => "Crossbow Shot",
            Self::AimedThrow => "Aimed Throw",
            Self::SnapShot => "Snap Shot",
            Self::RapidFire => "Rapid Fire",
            Self::SniperShot => "Sniper Shot",
            Self::VolleyFire => "Volley Fire",
            Self::PartingShot => "Parting Shot",
            // Craft Level 1
            Self::CraftBasicHeatCycle => "Basic Heat Cycle",
            Self::CraftBasicHammerWork => "Basic Hammer Work",
            Self::CraftBasicMeasure => "Basic Measure",
            Self::CraftBasicCut => "Basic Cut",
            Self::CraftBasicJoin => "Basic Join",
            // Craft Level 2
            Self::CraftDrawOutMetal => "Draw Out Metal",
            Self::CraftUpsetMetal => "Upset Metal",
            Self::CraftBasicWeld => "Basic Weld",
            Self::CraftShapeWood => "Shape Wood",
            Self::CraftFinishSurface => "Finish Surface",
            // Craft Level 3
            Self::CraftForgeKnife => "Forge Knife",
            Self::CraftForgeToolHead => "Forge Tool Head",
            Self::CraftBuildFurniture => "Build Furniture",
            Self::CraftSewGarment => "Sew Garment",
            // Craft Level 4
            Self::CraftForgeSword => "Forge Sword",
            Self::CraftForgeArmor => "Forge Armor",
            Self::CraftBuildStructure => "Build Structure",
            Self::CraftPatternWeld => "Pattern Weld",
            // Craft Level 5
            Self::CraftAssessAndExecute => "Assess and Execute",
            Self::CraftForgeMasterwork => "Forge Masterwork",
            Self::CraftInnovativeTechnique => "Innovative Technique",
            // Social Level 1
            Self::SocialActiveListening => "Active Listening",
            Self::SocialProjectConfidence => "Project Confidence",
            Self::SocialEmpathicMirror => "Empathic Mirror",
            Self::SocialCreateTension => "Create Tension",
            // Social Level 2
            Self::SocialBuildRapport => "Build Rapport",
            Self::SocialProjectAuthority => "Project Authority",
            Self::SocialReadReaction => "Read Reaction",
            Self::SocialDeflectInquiry => "Deflect Inquiry",
            Self::SocialEmotionalAppeal => "Emotional Appeal",
            // Social Level 3
            Self::SocialNegotiateTerms => "Negotiate Terms",
            Self::SocialIntimidate => "Intimidate",
            Self::SocialPersuade => "Persuade",
            Self::SocialDeceive => "Deceive",
            Self::SocialInspire => "Inspire",
            // Social Level 4
            Self::SocialWorkRoom => "Work Room",
            Self::SocialPoliticalManeuver => "Political Maneuver",
            Self::SocialLeadGroup => "Lead Group",
            Self::SocialMediateConflict => "Mediate Conflict",
            // Social Level 5
            Self::SocialOmniscience => "Social Omniscience",
            Self::SocialManipulateDynamics => "Manipulate Dynamics",
            Self::SocialCultOfPersonality => "Cult of Personality",
            // Medicine Level 1
            Self::MedWoundAssessment => "Wound Assessment",
            Self::MedBasicCleaning => "Basic Cleaning",
            Self::MedBasicSuture => "Basic Suture",
            Self::MedVitalCheck => "Vital Check",
            // Medicine Level 2
            Self::MedTreatLaceration => "Treat Laceration",
            Self::MedSetFracture => "Set Fracture",
            Self::MedPreparePoultice => "Prepare Poultice",
            Self::MedDiagnoseIllness => "Diagnose Illness",
            Self::MedPainManagement => "Pain Management",
            // Medicine Level 3
            Self::MedFieldSurgery => "Field Surgery",
            Self::MedTreatInfection => "Treat Infection",
            Self::MedDeliverBaby => "Deliver Baby",
            Self::MedAmputation => "Amputation",
            // Medicine Level 4
            Self::MedBattlefieldTriage => "Battlefield Triage",
            Self::MedComplexSurgery => "Complex Surgery",
            Self::MedEpidemicResponse => "Epidemic Response",
            // Medicine Level 5
            Self::MedDiagnosticIntuition => "Diagnostic Intuition",
            Self::MedSurgicalExcellence => "Surgical Excellence",
            Self::MedHolisticTreatment => "Holistic Treatment",
            // Leadership Level 1
            Self::LeadCommandPresence => "Command Presence",
            Self::LeadClearOrder => "Clear Order",
            Self::LeadSituationalRead => "Situational Read",
            // Leadership Level 2
            Self::LeadIssueCommand => "Issue Command",
            Self::LeadAssessUnitState => "Assess Unit State",
            Self::LeadDelegateTask => "Delegate Task",
            Self::LeadMaintainCalm => "Maintain Calm",
            // Leadership Level 3
            Self::LeadDirectFormation => "Direct Formation",
            Self::LeadRespondToCrisis => "Respond to Crisis",
            Self::LeadRallyWavering => "Rally Wavering",
            Self::LeadCoordinateUnits => "Coordinate Units",
            // Leadership Level 4
            Self::LeadBattleManagement => "Battle Management",
            Self::LeadCampaignPlanning => "Campaign Planning",
            Self::LeadOrganizationBuilding => "Organization Building",
            // Leadership Level 5
            Self::LeadReadBattleFlow => "Read Battle Flow",
            Self::LeadInspireArmy => "Inspire Army",
            Self::LeadStrategicIntuition => "Strategic Intuition",
            // Knowledge Level 1
            Self::KnowFluentReading => "Fluent Reading",
            Self::KnowFluentWriting => "Fluent Writing",
            Self::KnowArithmetic => "Arithmetic",
            Self::KnowMemorization => "Memorization",
            // Knowledge Level 2
            Self::KnowResearchSource => "Research Source",
            Self::KnowComposeDocument => "Compose Document",
            Self::KnowMathematicalProof => "Mathematical Proof",
            Self::KnowTeachConcept => "Teach Concept",
            Self::KnowTranslateText => "Translate Text",
            // Knowledge Level 3
            Self::KnowAnalyzeText => "Analyze Text",
            Self::KnowSynthesizeSources => "Synthesize Sources",
            Self::KnowFormalArgument => "Formal Argument",
            Self::KnowInstructStudent => "Instruct Student",
            // Knowledge Level 4
            Self::KnowOriginalResearch => "Original Research",
            Self::KnowComprehensiveTreatise => "Comprehensive Treatise",
            Self::KnowCurriculumDesign => "Curriculum Design",
            // Knowledge Level 5
            Self::KnowParadigmIntegration => "Paradigm Integration",
            Self::KnowIntellectualLegacy => "Intellectual Legacy",
            // Physical Level 1
            Self::PhysEfficientGait => "Efficient Gait",
            Self::PhysQuietMovement => "Quiet Movement",
            Self::PhysPowerStance => "Power Stance",
            Self::PhysClimbGrip => "Climb Grip",
            // Physical Level 2
            Self::PhysDistanceRunning => "Distance Running",
            Self::PhysHeavyLifting => "Heavy Lifting",
            Self::PhysSilentApproach => "Silent Approach",
            Self::PhysRockClimbing => "Rock Climbing",
            Self::PhysHorseControl => "Horse Control",
            // Physical Level 3
            Self::PhysSustainedLabor => "Sustained Labor",
            Self::PhysInfiltration => "Infiltration",
            Self::PhysRoughTerrainTravel => "Rough Terrain Travel",
            Self::PhysCavalryRiding => "Cavalry Riding",
            Self::PhysSwimming => "Swimming",
            // Physical Level 4
            Self::PhysLaborLeadership => "Labor Leadership",
            Self::PhysScoutMission => "Scout Mission",
            Self::PhysMountedCombat => "Mounted Combat",
            Self::PhysSurvivalTravel => "Survival Travel",
            // Physical Level 5
            Self::PhysTirelessEndurance => "Tireless Endurance",
            Self::PhysShadowMovement => "Shadow Movement",
            Self::PhysCentaurUnity => "Centaur Unity",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_levels() {
        assert_eq!(ChunkId::BasicSwing.level(), 1);
        assert_eq!(ChunkId::AttackSequence.level(), 2);
        assert_eq!(ChunkId::EngageMelee.level(), 3);
    }

    #[test]
    fn test_chunk_names() {
        assert_eq!(ChunkId::BasicSwing.name(), "Basic Swing");
        assert_eq!(ChunkId::Riposte.name(), "Riposte");
    }

    #[test]
    fn test_ranged_chunk_levels() {
        // Level 1 ranged
        assert_eq!(ChunkId::DrawBow.level(), 1);
        assert_eq!(ChunkId::LoadCrossbow.level(), 1);
        assert_eq!(ChunkId::BasicAim.level(), 1);
        assert_eq!(ChunkId::BasicThrow.level(), 1);

        // Level 2 ranged
        assert_eq!(ChunkId::LooseArrow.level(), 2);
        assert_eq!(ChunkId::CrossbowShot.level(), 2);
        assert_eq!(ChunkId::AimedThrow.level(), 2);

        // Level 3 ranged
        assert_eq!(ChunkId::RapidFire.level(), 3);
        assert_eq!(ChunkId::SniperShot.level(), 3);
        assert_eq!(ChunkId::VolleyFire.level(), 3);
    }

    #[test]
    fn test_ranged_chunk_names() {
        assert_eq!(ChunkId::DrawBow.name(), "Draw Bow");
        assert_eq!(ChunkId::LooseArrow.name(), "Loose Arrow");
        assert_eq!(ChunkId::RapidFire.name(), "Rapid Fire");
    }

    #[test]
    fn test_chunk_domains() {
        use crate::skills::ChunkDomain;

        // Combat chunks - melee
        assert_eq!(ChunkId::BasicSwing.domain(), ChunkDomain::Combat);
        assert_eq!(ChunkId::HandleFlanking.domain(), ChunkDomain::Combat);

        // Combat chunks - ranged
        assert_eq!(ChunkId::DrawBow.domain(), ChunkDomain::Combat);
        assert_eq!(ChunkId::RapidFire.domain(), ChunkDomain::Combat);
    }

    #[test]
    fn test_craft_chunks_exist() {
        use crate::skills::ChunkDomain;

        // Level 1 craft chunks
        assert_eq!(ChunkId::CraftBasicHeatCycle.domain(), ChunkDomain::Craft);
        assert_eq!(ChunkId::CraftBasicHammerWork.domain(), ChunkDomain::Craft);
        assert_eq!(ChunkId::CraftBasicMeasure.domain(), ChunkDomain::Craft);

        // Level 2
        assert_eq!(ChunkId::CraftDrawOutMetal.domain(), ChunkDomain::Craft);

        // Level 3
        assert_eq!(ChunkId::CraftForgeKnife.domain(), ChunkDomain::Craft);

        // Level 4
        assert_eq!(ChunkId::CraftForgeSword.domain(), ChunkDomain::Craft);

        // Level 5
        assert_eq!(ChunkId::CraftForgeMasterwork.domain(), ChunkDomain::Craft);
    }

    #[test]
    fn test_social_chunks_exist() {
        use crate::skills::ChunkDomain;

        assert_eq!(ChunkId::SocialActiveListening.domain(), ChunkDomain::Social);
        assert_eq!(ChunkId::SocialBuildRapport.domain(), ChunkDomain::Social);
        assert_eq!(ChunkId::SocialNegotiateTerms.domain(), ChunkDomain::Social);
        assert_eq!(ChunkId::SocialWorkRoom.domain(), ChunkDomain::Social);
        assert_eq!(
            ChunkId::SocialManipulateDynamics.domain(),
            ChunkDomain::Social
        );
    }

    #[test]
    fn test_medicine_chunks_exist() {
        use crate::skills::ChunkDomain;

        assert_eq!(ChunkId::MedWoundAssessment.domain(), ChunkDomain::Medicine);
        assert_eq!(ChunkId::MedTreatLaceration.domain(), ChunkDomain::Medicine);
        assert_eq!(ChunkId::MedFieldSurgery.domain(), ChunkDomain::Medicine);
        assert_eq!(
            ChunkId::MedBattlefieldTriage.domain(),
            ChunkDomain::Medicine
        );
        assert_eq!(
            ChunkId::MedDiagnosticIntuition.domain(),
            ChunkDomain::Medicine
        );
    }

    #[test]
    fn test_leadership_chunks_exist() {
        use crate::skills::ChunkDomain;

        assert_eq!(
            ChunkId::LeadCommandPresence.domain(),
            ChunkDomain::Leadership
        );
        assert_eq!(ChunkId::LeadIssueCommand.domain(), ChunkDomain::Leadership);
        assert_eq!(
            ChunkId::LeadDirectFormation.domain(),
            ChunkDomain::Leadership
        );
        assert_eq!(
            ChunkId::LeadBattleManagement.domain(),
            ChunkDomain::Leadership
        );
        assert_eq!(
            ChunkId::LeadStrategicIntuition.domain(),
            ChunkDomain::Leadership
        );
    }

    #[test]
    fn test_knowledge_chunks_exist() {
        use crate::skills::ChunkDomain;

        assert_eq!(ChunkId::KnowFluentReading.domain(), ChunkDomain::Knowledge);
        assert_eq!(ChunkId::KnowResearchSource.domain(), ChunkDomain::Knowledge);
        assert_eq!(ChunkId::KnowAnalyzeText.domain(), ChunkDomain::Knowledge);
        assert_eq!(
            ChunkId::KnowOriginalResearch.domain(),
            ChunkDomain::Knowledge
        );
        assert_eq!(
            ChunkId::KnowIntellectualLegacy.domain(),
            ChunkDomain::Knowledge
        );
    }

    #[test]
    fn test_physical_chunks_exist() {
        use crate::skills::ChunkDomain;

        assert_eq!(ChunkId::PhysEfficientGait.domain(), ChunkDomain::Physical);
        assert_eq!(ChunkId::PhysDistanceRunning.domain(), ChunkDomain::Physical);
        assert_eq!(ChunkId::PhysSustainedLabor.domain(), ChunkDomain::Physical);
        assert_eq!(ChunkId::PhysScoutMission.domain(), ChunkDomain::Physical);
        assert_eq!(
            ChunkId::PhysTirelessEndurance.domain(),
            ChunkDomain::Physical
        );
    }
}
