//! Events and history logging

use serde::{Deserialize, Serialize};

use crate::aggregate::polity::{GrudgeReason, TreatyTerms, DecisionType};
use crate::aggregate::world::WarCause;
use crate::core::types::PolityId;

/// A historical event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: u32,
    pub year: u32,
    pub event_type: EventType,
    pub participants: Vec<u32>,
    pub location: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventType {
    // Wars
    WarDeclared { aggressor: u32, defender: u32, cause: WarCause },
    Battle { war_id: u32, location: u32, winner: u32, casualties: (u32, u32) },
    Siege { war_id: u32, target: u32, successful: bool },
    WarEnded { war_id: u32, victor: Option<u32> },

    // Diplomacy
    AllianceFormed { members: Vec<u32> },
    AllianceBroken { breaker: u32 },
    Treaty { parties: Vec<u32>, terms: TreatyTerms },
    Betrayal { betrayer: u32, victim: u32 },

    // Territory
    Expansion { polity: u32, region: u32 },
    RegionLost { loser: u32, winner: u32, region: u32 },
    Settlement { polity: u32, region: u32, name: String },

    // Internal
    CivilWar { polity: u32, faction_ids: Vec<u32> },
    PolityCollapsed { polity: u32, successor_states: Vec<u32> },
    PolityMerged { absorbed: u32, absorber: u32 },

    // Cultural
    TraditionAdopted { polity: u32, tradition: String },
    CulturalDrift { polity: u32, value: String, direction: f32 },

    // Disasters
    Plague { affected: Vec<u32>, severity: f32 },
    Famine { affected: Vec<u32> },

    // Dwarf-specific
    GrudgeDeclared { polity: u32, against: u32, reason: GrudgeReason },
    GrudgeSettled { polity: u32, against: u32 },
    OathSworn { polity: u32, oath_id: u32 },
    OathBroken { polity: u32, oath_id: u32 },

    // Elf-specific
    GriefEvent { polity: u32, intensity: f32 },
    DeliberationComplete { polity: u32, decision: DecisionType },
    Isolation { polity: u32 },

    // Gnoll-specific
    RaidLaunched { attacker: PolityId, target: PolityId },
    CorruptionSpreads { polity: PolityId, intensity: f32 },
    BattleWon { polity: PolityId },
    BattleLost { polity: PolityId },

    // Vampire-specific
    InfiltrationAttempt { infiltrator: PolityId, target: PolityId },
    InfiltrationSuccess { infiltrator: PolityId, target: PolityId },
    TributeDemanded { from: PolityId, amount: u32 },
    TributePaid { to: PolityId, amount: u32 },

    // Kobold-specific
    TrapConstruction { polity: PolityId, trap_count: u32 },
    SpiteRaid { attacker: PolityId, target: PolityId },
    DragonTributeOffered { polity: PolityId },
    TrapTriggered { polity: PolityId, casualties: u32 },
}

/// The complete history log
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HistoryLog {
    pub events: Vec<Event>,
    next_event_id: u32,
}

impl HistoryLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event(&mut self, event_type: EventType, year: u32, participants: Vec<u32>, location: Option<u32>) -> u32 {
        let id = self.next_event_id;
        self.next_event_id += 1;

        self.events.push(Event {
            id,
            year,
            event_type,
            participants,
            location,
        });

        id
    }

    pub fn events_for_year(&self, year: u32) -> impl Iterator<Item = &Event> {
        self.events.iter().filter(move |e| e.year == year)
    }

    pub fn events_for_polity(&self, polity_id: u32) -> impl Iterator<Item = &Event> {
        self.events.iter().filter(move |e| e.participants.contains(&polity_id))
    }
}
