//! LLM Gateway
//!
//! Adapters for converting between Battle State and LLM Text/JSON.
//! Defines the Schema for LLM inputs (PromptContext) and outputs (LlmResponse).

use crate::battle::ai::DecisionContext;
use crate::battle::courier::{Order, OrderTarget, OrderType};
use crate::battle::hex::BattleHexCoord;
use crate::battle::units::UnitId;
use serde::{Deserialize, Serialize};

// =========================================================================
//  INPUT SCHEMA (Rust -> LLM)
// =========================================================================

/// Structured context for the LLM prompt
#[derive(Serialize)]
pub struct PromptContext {
    pub turn: u64,
    pub situation_summary: String,
    pub my_units: Vec<UnitSummary>,
    pub visible_enemies: Vec<EnemySummary>,
    pub current_objectives: Vec<String>,
}

#[derive(Serialize)]
pub struct UnitSummary {
    pub id: String,
    pub type_name: String,
    pub strength_pct: u8,
    pub morale_status: String, // "Steady", "Shaken", "Broken"
    pub position: String,      // "x,y"
    pub engaged_with: Option<String>,
}

#[derive(Serialize)]
pub struct EnemySummary {
    pub id: String,
    pub estimated_strength: String, // "Strong", "Weak"
    pub position: String,
    pub distance: u32,
}

impl PromptContext {
    /// Convert runtime DecisionContext into serializable PromptContext
    pub fn from_decision_context(dc: &DecisionContext) -> Self {
        let my_units = dc.own_units().iter().map(|u| {
            let total = u.elements.iter().map(|e| e.strength()).sum::<usize>() as f32;
            let effective = u.effective_strength() as f32;
            let strength_pct = if total > 0.0 { ((effective / total) * 100.0) as u8 } else { 0 };
            UnitSummary {
                id: u.id.0.to_string(),
                type_name: format!("{:?}", u.unit_type),
                strength_pct,
                morale_status: if u.is_broken() { "Broken".into() } else { "Steady".into() },
                position: format!("{},{}", u.position.q, u.position.r),
                engaged_with: None, // TODO: Check engagement status
            }
        }).collect();

        let visible_enemies = dc.visible_enemy_units().iter().map(|u| {
             EnemySummary {
                id: u.id.0.to_string(),
                estimated_strength: if u.effective_strength() > 50 { "Strong".into() } else { "Weak".into() },
                position: format!("{},{}", u.position.q, u.position.r),
                distance: 0, // Calculated relative to HQ or center mass
            }
        }).collect();

        Self {
            turn: dc.current_tick,
            situation_summary: "Battle in progress.".to_string(),
            my_units,
            visible_enemies,
            current_objectives: vec!["Defeat Enemy".to_string()],
        }
    }
}

// =========================================================================
//  OUTPUT SCHEMA (LLM -> Rust)
// =========================================================================

/// The expected JSON output from the LLM
#[derive(Serialize, Deserialize, Debug)]
pub struct LlmResponse {
    /// Chain-of-thought reasoning
    pub reasoning: String,
    /// List of orders to issue
    pub orders: Vec<LlmOrder>,
}

/// A single order from the LLM
#[derive(Serialize, Deserialize, Debug)]
pub struct LlmOrder {
    /// Target Unit ID (or "ALL")
    pub unit_id: String,
    /// Action type: MOVE, ATTACK, DEFEND, RETREAT
    pub action: String,
    /// Target: "x,y" for Move/Defend, UnitID for Attack
    pub target: String,
}

impl LlmOrder {
    /// Convert LLM order to Engine Order
    pub fn to_engine_order(&self) -> Result<Order, String> {
        let unit_id = UnitId::parse_str(&self.unit_id)
            .ok_or_else(|| format!("Invalid Unit ID: {}", self.unit_id))?;

        match self.action.to_uppercase().as_str() {
            "MOVE" => {
                let coord = parse_coord(&self.target)?;
                Ok(Order::move_to(unit_id, coord))
            },
            "ATTACK" => {
                let target_id = UnitId::parse_str(&self.target)
                    .ok_or_else(|| format!("Invalid Target ID: {}", self.target))?;
                Ok(Order::attack(unit_id, target_id))
            },
            "DEFEND" => {
                let coord = parse_coord(&self.target)?;
                Ok(Order {
                    order_type: OrderType::Defend(coord),
                    target: OrderTarget::Unit(unit_id),
                    issued_at: 0,
                })
            },
            "RETREAT" => {
                // Simplified retreat to single point for now
                let coord = parse_coord(&self.target)?;
                Ok(Order::retreat(unit_id, vec![coord]))
            },
            _ => Err(format!("Unknown action: {}", self.action)),
        }
    }
}

fn parse_coord(s: &str) -> Result<BattleHexCoord, String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid coordinate format: {}", s));
    }
    let q = parts[0].trim().parse().map_err(|_| "Invalid x")?;
    let r = parts[1].trim().parse().map_err(|_| "Invalid y")?;
    Ok(BattleHexCoord::new(q, r))
}

// Helper to make UnitId parsable from string
trait ParseableId {
    fn parse_str(s: &str) -> Option<UnitId>;
}

impl ParseableId for UnitId {
    fn parse_str(s: &str) -> Option<UnitId> {
        uuid::Uuid::parse_str(s).ok().map(UnitId)
    }
}
