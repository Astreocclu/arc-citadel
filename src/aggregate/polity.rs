//! Polity - nation/tribe/hold/grove and species-specific state

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::core::types::{GovernmentType, PolityId, PolityTier, RulerId, Species};

/// A polity (nation, tribe, hold, grove, etc.)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Polity {
    pub id: PolityId,
    pub name: String,
    pub species: Species,
    pub polity_type: PolityType,

    // Hierarchy fields
    pub tier: PolityTier,
    pub government: GovernmentType,
    pub parent: Option<PolityId>,  // None = sovereign
    pub rulers: Vec<RulerId>,       // len=1 for autocracy, len=N for council
    pub council_roles: HashMap<CouncilRole, RulerId>,

    // Physical state (territory removed - Location.controller is source of truth)
    pub population: u32,
    pub capital: u32,  // Region ID
    pub military_strength: f32,
    pub economic_strength: f32,

    // Cultural drift from species baseline
    pub cultural_drift: CulturalDrift,

    // Relations with other polities (treaties, not opinions)
    pub relations: HashMap<u32, Relation>,

    // Species-specific state
    pub species_state: SpeciesState,

    // Alive status
    pub alive: bool,
}

/// Council roles for government
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CouncilRole {
    Chancellor,   // Diplomacy
    Marshal,      // Military
    Steward,      // Economy
    Spymaster,    // Intrigue
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolityType {
    // Human
    Kingdom,
    Tribe,
    CityState,
    // Dwarf
    Clan,
    Hold,
    // Elf
    Grove,
    Court,
    // Orc
    Warband,
    Horde,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CulturalDrift {
    pub primary_drift: Option<(String, f32)>,
    pub secondary_drift: Option<(String, f32)>,
    pub traditions: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Relation {
    pub opinion: i32,  // -100 to +100
    pub trust: i32,    // -100 to +100
    pub at_war: bool,
    pub alliance: bool,
    pub grudges: Vec<Grudge>,
    pub treaties: Vec<Treaty>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grudge {
    pub id: u32,
    pub against: u32,
    pub reason: GrudgeReason,
    pub severity: f32,
    pub year_incurred: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GrudgeReason {
    Betrayal,
    TerritoryLost(u32),
    HoldsAncestralSite(u32),
    OathBroken,
    KinSlain { count: u32 },
    InsultGiven,
    DebtUnpaid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Treaty {
    pub id: u32,
    pub parties: Vec<u32>,
    pub terms: TreatyTerms,
    pub year_signed: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TreatyTerms {
    Peace,
    Trade,
    MilitaryAccess,
    Tribute { from: u32, to: u32, amount: u32 },
    // Vassalage removed - now represented by Polity.parent field
}

/// Species-specific state - enum variants for static dispatch
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpeciesState {
    Human(HumanState),
    Dwarf(DwarfState),
    Elf(ElfState),
    Orc(OrcState),
    Kobold(KoboldState),
    Gnoll(GnollState),
    Lizardfolk(LizardfolkState),
    Hobgoblin(HobgoblinState),
    Ogre(OgreState),
    Harpy(HarpyState),
    Centaur(CentaurState),
    Minotaur(MinotaurState),
    Satyr(SatyrState),
    Dryad(DryadState),
    Goblin(GoblinState),
    Troll(TrollState),
    // CODEGEN: species_state_variants
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HumanState {
    pub expansion_pressure: f32,
    pub internal_cohesion: f32,
    pub reputation: f32,
    pub piety: f32,
    pub factions: Vec<Faction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Faction {
    pub id: u32,
    pub name: String,
    pub power: f32,
    pub ideology: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DwarfState {
    pub grudge_ledger: HashMap<u32, Vec<Grudge>>,
    pub oaths: Vec<Oath>,
    pub ancestral_sites: Vec<u32>,
    pub craft_focus: CraftType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Oath {
    pub id: u32,
    pub sworn_to: Option<u32>,
    pub oath_type: OathType,
    pub year_sworn: u32,
    pub fulfilled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OathType {
    MutualDefense,
    Vengeance { target: u32 },
    Service { duration_years: u32 },
    Silence,
    Crafting { item: String },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CraftType {
    #[default]
    Stone,
    Metal,
    Gems,
    Weapons,
    Armor,
    Architecture,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ElfState {
    pub memory: Vec<HistoricalMemory>,
    pub grief_level: f32,
    pub pending_decisions: Vec<PendingDecision>,
    pub core_territory: HashSet<u32>,
    pub pattern_assessment: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoricalMemory {
    pub event_id: u32,
    pub year: u32,
    pub emotional_weight: f32,
    pub lesson_learned: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingDecision {
    pub trigger_event: u32,
    pub deliberation_started: u32,
    pub deliberation_required: u32,
    pub decision_type: DecisionType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecisionType {
    War { target: u32 },
    Alliance { with: u32 },
    Isolation,
    PatternIntervention { situation: u32 },
    Migration,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OrcState {
    pub waaagh_level: f32,
    pub raid_targets: Vec<u32>,
    pub blood_feuds: Vec<u32>,
    pub tribal_strength: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KoboldState {
    pub trap_density: f32,
    pub tunnel_network: u32,
    pub dragon_worship: f32,
    pub grudge_targets: Vec<u32>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GnollState {
    pub pack_frenzy: f32,
    pub hunting_grounds: Vec<u32>,
    pub demon_taint: f32,
    pub slave_count: u32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LizardfolkState {
    pub spawning_pools: u32,
    pub food_stores: f32,
    pub tribal_memory: Vec<String>,
    pub alliance_pragmatism: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HobgoblinState {
    pub military_doctrine: f32,
    pub legion_strength: u32,
    pub conquered_territories: Vec<u32>,
    pub war_machine: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OgreState {
    pub meat_stores: f32,
    pub territory_size: u32,
    pub dominated_tribes: Vec<u32>,
    pub giant_blood: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HarpyState {
    pub nesting_sites: Vec<u32>,
    pub trinket_hoard: f32,
    pub cursed_ones: u32,
    pub flock_unity: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CentaurState {
    pub sacred_grounds: Vec<u32>,
    pub herd_bonds: f32,
    pub star_wisdom: f32,
    pub oaths_sworn: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MinotaurState {
    pub labyrinth_depth: u32,
    pub sacrifices_claimed: u32,
    pub cursed_bloodline: f32,
    pub territorial_markers: Vec<u32>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SatyrState {
    pub revelry_level: f32,
    pub wine_stores: f32,
    pub charmed_mortals: Vec<u32>,
    pub fey_connection: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DryadState {
    pub sacred_trees: u32,
    pub forest_health: f32,
    pub corrupted_lands: Vec<u32>,
    pub fey_pacts: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GoblinState {
    pub grudge_list: Vec<u32>,
    pub hoard_value: f32,
    pub raid_targets: Vec<u32>,
    pub war_exhaustion: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrollState {
    pub grudge_list: Vec<u32>,
    pub hoard_value: f32,
    pub war_exhaustion: f32,
}
// CODEGEN: species_state_structs

impl Polity {
    pub fn human_state(&self) -> Option<&HumanState> {
        match &self.species_state {
            SpeciesState::Human(s) => Some(s),
            _ => None,
        }
    }

    pub fn human_state_mut(&mut self) -> Option<&mut HumanState> {
        match &mut self.species_state {
            SpeciesState::Human(s) => Some(s),
            _ => None,
        }
    }

    pub fn dwarf_state(&self) -> Option<&DwarfState> {
        match &self.species_state {
            SpeciesState::Dwarf(s) => Some(s),
            _ => None,
        }
    }

    pub fn dwarf_state_mut(&mut self) -> Option<&mut DwarfState> {
        match &mut self.species_state {
            SpeciesState::Dwarf(s) => Some(s),
            _ => None,
        }
    }

    pub fn elf_state(&self) -> Option<&ElfState> {
        match &self.species_state {
            SpeciesState::Elf(s) => Some(s),
            _ => None,
        }
    }

    pub fn elf_state_mut(&mut self) -> Option<&mut ElfState> {
        match &mut self.species_state {
            SpeciesState::Elf(s) => Some(s),
            _ => None,
        }
    }


    pub fn orc_state(&self) -> Option<&OrcState> {
        match &self.species_state {
            SpeciesState::Orc(s) => Some(s),
            _ => None,
        }
    }

    pub fn orc_state_mut(&mut self) -> Option<&mut OrcState> {
        match &mut self.species_state {
            SpeciesState::Orc(s) => Some(s),
            _ => None,
        }
    }

    pub fn kobold_state(&self) -> Option<&KoboldState> {
        match &self.species_state {
            SpeciesState::Kobold(s) => Some(s),
            _ => None,
        }
    }

    pub fn kobold_state_mut(&mut self) -> Option<&mut KoboldState> {
        match &mut self.species_state {
            SpeciesState::Kobold(s) => Some(s),
            _ => None,
        }
    }

    pub fn gnoll_state(&self) -> Option<&GnollState> {
        match &self.species_state {
            SpeciesState::Gnoll(s) => Some(s),
            _ => None,
        }
    }

    pub fn gnoll_state_mut(&mut self) -> Option<&mut GnollState> {
        match &mut self.species_state {
            SpeciesState::Gnoll(s) => Some(s),
            _ => None,
        }
    }

    pub fn lizardfolk_state(&self) -> Option<&LizardfolkState> {
        match &self.species_state {
            SpeciesState::Lizardfolk(s) => Some(s),
            _ => None,
        }
    }

    pub fn lizardfolk_state_mut(&mut self) -> Option<&mut LizardfolkState> {
        match &mut self.species_state {
            SpeciesState::Lizardfolk(s) => Some(s),
            _ => None,
        }
    }

    pub fn hobgoblin_state(&self) -> Option<&HobgoblinState> {
        match &self.species_state {
            SpeciesState::Hobgoblin(s) => Some(s),
            _ => None,
        }
    }

    pub fn hobgoblin_state_mut(&mut self) -> Option<&mut HobgoblinState> {
        match &mut self.species_state {
            SpeciesState::Hobgoblin(s) => Some(s),
            _ => None,
        }
    }

    pub fn ogre_state(&self) -> Option<&OgreState> {
        match &self.species_state {
            SpeciesState::Ogre(s) => Some(s),
            _ => None,
        }
    }

    pub fn ogre_state_mut(&mut self) -> Option<&mut OgreState> {
        match &mut self.species_state {
            SpeciesState::Ogre(s) => Some(s),
            _ => None,
        }
    }

    pub fn harpy_state(&self) -> Option<&HarpyState> {
        match &self.species_state {
            SpeciesState::Harpy(s) => Some(s),
            _ => None,
        }
    }

    pub fn harpy_state_mut(&mut self) -> Option<&mut HarpyState> {
        match &mut self.species_state {
            SpeciesState::Harpy(s) => Some(s),
            _ => None,
        }
    }

    pub fn centaur_state(&self) -> Option<&CentaurState> {
        match &self.species_state {
            SpeciesState::Centaur(s) => Some(s),
            _ => None,
        }
    }

    pub fn centaur_state_mut(&mut self) -> Option<&mut CentaurState> {
        match &mut self.species_state {
            SpeciesState::Centaur(s) => Some(s),
            _ => None,
        }
    }

    pub fn minotaur_state(&self) -> Option<&MinotaurState> {
        match &self.species_state {
            SpeciesState::Minotaur(s) => Some(s),
            _ => None,
        }
    }

    pub fn minotaur_state_mut(&mut self) -> Option<&mut MinotaurState> {
        match &mut self.species_state {
            SpeciesState::Minotaur(s) => Some(s),
            _ => None,
        }
    }

    pub fn satyr_state(&self) -> Option<&SatyrState> {
        match &self.species_state {
            SpeciesState::Satyr(s) => Some(s),
            _ => None,
        }
    }

    pub fn satyr_state_mut(&mut self) -> Option<&mut SatyrState> {
        match &mut self.species_state {
            SpeciesState::Satyr(s) => Some(s),
            _ => None,
        }
    }

    pub fn dryad_state(&self) -> Option<&DryadState> {
        match &self.species_state {
            SpeciesState::Dryad(s) => Some(s),
            _ => None,
        }
    }

    pub fn dryad_state_mut(&mut self) -> Option<&mut DryadState> {
        match &mut self.species_state {
            SpeciesState::Dryad(s) => Some(s),
            _ => None,
        }
    }

    pub fn goblin_state(&self) -> Option<&GoblinState> {
        match &self.species_state {
            SpeciesState::Goblin(s) => Some(s),
            _ => None,
        }
    }

    pub fn goblin_state_mut(&mut self) -> Option<&mut GoblinState> {
        match &mut self.species_state {
            SpeciesState::Goblin(s) => Some(s),
            _ => None,
        }
    }

    pub fn troll_state(&self) -> Option<&TrollState> {
        match &self.species_state {
            SpeciesState::Troll(s) => Some(s),
            _ => None,
        }
    }

    pub fn troll_state_mut(&mut self) -> Option<&mut TrollState> {
        match &mut self.species_state {
            SpeciesState::Troll(s) => Some(s),
            _ => None,
        }
    }
    // CODEGEN: species_state_accessors

    /// Returns true if this polity has no parent (is sovereign)
    pub fn is_sovereign(&self) -> bool {
        self.parent.is_none()
    }

    /// Get the liege (immediate parent) if any
    pub fn liege(&self) -> Option<PolityId> {
        self.parent
    }

    /// Get the primary ruler (first in rulers list)
    pub fn primary_ruler(&self) -> Option<RulerId> {
        self.rulers.first().copied()
    }

    /// Check if a ruler is part of this polity's leadership
    pub fn has_ruler(&self, ruler: RulerId) -> bool {
        self.rulers.contains(&ruler)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{PolityId, RulerId, PolityTier, GovernmentType};

    #[test]
    fn test_polity_has_new_fields() {
        let polity = Polity {
            id: PolityId(1),
            name: "Kingdom of Aldoria".to_string(),
            species: Species::Human,
            polity_type: PolityType::Kingdom,
            tier: PolityTier::Kingdom,
            government: GovernmentType::Autocracy,
            parent: None, // Sovereign
            rulers: vec![RulerId(1)],
            council_roles: HashMap::new(),
            capital: 0,
            population: 10000,
            military_strength: 100.0,
            economic_strength: 100.0,
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state: SpeciesState::Human(HumanState::default()),
            alive: true,
        };

        assert!(polity.is_sovereign());
        assert_eq!(polity.tier, PolityTier::Kingdom);
        assert_eq!(polity.rulers.len(), 1);
    }

    #[test]
    fn test_polity_is_vassal() {
        let polity = Polity {
            id: PolityId(2),
            name: "Duchy of Valheim".to_string(),
            species: Species::Human,
            polity_type: PolityType::Kingdom, // Cultural type
            tier: PolityTier::Duchy,          // Hierarchy rank
            government: GovernmentType::Autocracy,
            parent: Some(PolityId(1)),        // Vassal of polity 1
            rulers: vec![RulerId(2)],
            council_roles: HashMap::new(),
            capital: 1,
            population: 5000,
            military_strength: 50.0,
            economic_strength: 50.0,
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state: SpeciesState::Human(HumanState::default()),
            alive: true,
        };

        assert!(!polity.is_sovereign());
        assert_eq!(polity.parent, Some(PolityId(1)));
    }
}
