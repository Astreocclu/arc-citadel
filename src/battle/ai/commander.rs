//! AI Commander - main battle AI implementation
//!
//! Evaluates battle state and issues orders through courier system.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::battle::ai::decision_context::DecisionContext;
use crate::battle::ai::personality::AiPersonality;
use crate::battle::ai::phase_plans::PhasePlanManager;
use crate::battle::ai::BattleAI;
use crate::battle::courier::Order;
use crate::battle::execution::BattleEventLog;
use crate::battle::units::{BattleUnit, UnitId, UnitStance};
use crate::core::types::Tick;

/// AI Commander implementing BattleAI trait
pub struct AiCommander {
    personality: AiPersonality,
    phase_manager: PhasePlanManager,
    last_evaluation_tick: Option<Tick>,
    rng: StdRng,
    /// Track which units have pending orders (to avoid spamming)
    pending_orders: Vec<UnitId>,
}

impl AiCommander {
    /// Create a new AI commander with default seed
    pub fn new(personality: AiPersonality) -> Self {
        Self {
            personality,
            phase_manager: PhasePlanManager::new(),
            last_evaluation_tick: None,
            rng: StdRng::seed_from_u64(42), // Deterministic for testing
            pending_orders: Vec::new(),
        }
    }

    /// Create with specific RNG seed for deterministic behavior
    pub fn with_seed(personality: AiPersonality, seed: u64) -> Self {
        Self {
            personality,
            phase_manager: PhasePlanManager::new(),
            last_evaluation_tick: None,
            rng: StdRng::seed_from_u64(seed),
            pending_orders: Vec::new(),
        }
    }

    /// Set the phase plan manager
    pub fn set_phase_manager(&mut self, manager: PhasePlanManager) {
        self.phase_manager = manager;
    }

    /// Should we re-evaluate this tick?
    fn should_evaluate(&self, current_tick: Tick) -> bool {
        match self.last_evaluation_tick {
            None => true, // First evaluation always happens
            Some(last) => {
                let interval = self.personality.preferences.re_evaluation_interval;
                current_tick >= last + interval
            }
        }
    }

    /// Roll for making a mistake based on difficulty settings
    fn makes_mistake(&mut self) -> bool {
        self.rng.gen::<f32>() < self.personality.difficulty.mistake_chance
    }

    /// Evaluate tactical situation and decide orders
    fn evaluate_tactical(&mut self, context: &DecisionContext) -> Vec<Order> {
        let mut orders = Vec::new();

        // Clear pending orders from last evaluation cycle
        // Previous orders should have been delivered by now (re-eval interval > courier travel)
        self.pending_orders.clear();

        // Get effective aggression (base + phase modifier)
        let phase = self.phase_manager.current_phase();
        let effective_aggression =
            (self.personality.behavior.aggression + phase.aggression_modifier).clamp(0.0, 1.0);

        // Check if we should retreat
        if self.should_retreat(context) {
            return self.generate_retreat_orders(context);
        }

        // Get idle units that need orders
        let idle_units: Vec<&BattleUnit> = context
            .own_units()
            .into_iter()
            .filter(|u| self.unit_needs_orders(u))
            .collect();

        for unit in idle_units {
            if self.pending_orders.contains(&unit.id) {
                continue;
            }

            if let Some(order) = self.decide_unit_order(unit, context, effective_aggression) {
                // Apply mistake chance - skip order randomly
                if !self.makes_mistake() {
                    self.pending_orders.push(unit.id);
                    orders.push(order);
                }
            }
        }

        orders
    }

    /// Check if unit needs new orders
    /// Units that are Formed, Alert, or Moving can receive new orders.
    /// Moving units may need to be redirected if the tactical situation changes.
    fn unit_needs_orders(&self, unit: &BattleUnit) -> bool {
        matches!(
            unit.stance,
            UnitStance::Formed | UnitStance::Alert | UnitStance::Moving
        ) && unit.can_fight()
    }

    /// Decide what order to give a specific unit
    fn decide_unit_order(
        &mut self,
        unit: &BattleUnit,
        context: &DecisionContext,
        aggression: f32,
    ) -> Option<Order> {
        let visible_enemies = context.visible_enemy_units();

        if visible_enemies.is_empty() {
            // No visible enemies - but we know there's a battle happening.
            // Advance toward enemy HQ to find them.
            // Commanders know roughly where the enemy came from (pre-battle intel).
            let enemy_hq = context.enemy_hq_position();
            return Some(Order::move_to(unit.id, enemy_hq));
        }

        // Find best target based on personality weights
        let target = self.select_target(unit, &visible_enemies, context)?;

        // Decide action based on aggression
        if aggression > 0.5 {
            // Aggressive: attack
            Some(Order::attack(unit.id, target.id))
        } else {
            // Defensive: move towards but hold
            let halfway = unit.position.lerp(&target.position, 0.5);
            Some(Order::move_to(unit.id, halfway))
        }
    }

    /// Select best target for a unit based on personality weights
    fn select_target<'a>(
        &self,
        unit: &BattleUnit,
        enemies: &[&'a BattleUnit],
        _context: &DecisionContext,
    ) -> Option<&'a BattleUnit> {
        if enemies.is_empty() {
            return None;
        }

        let weights = &self.personality.weights;

        // Score each enemy
        let mut best_score = f32::MIN;
        let mut best_target = None;

        for enemy in enemies {
            let mut score = 0.0;

            // Weak targets are attractive
            let weakness = 1.0 - (enemy.effective_strength() as f32 / 100.0).min(1.0);
            score += weakness * weights.attack_value;

            // Close targets are easier
            let distance = unit.position.distance(&enemy.position) as f32;
            let closeness = 1.0 / (1.0 + distance * 0.1);
            score += closeness * 0.5;

            // Flanking opportunities (simplified: check if enemy is engaged)
            if enemy.is_engaged() {
                score += weights.flanking_value * 0.5;
            }

            // Routing enemies are priority
            if enemy.is_broken() {
                score += 1.0;
            }

            if score > best_score {
                best_score = score;
                best_target = Some(*enemy);
            }
        }

        best_target
    }

    /// Check if we should retreat based on strength ratio and casualties
    fn should_retreat(&self, context: &DecisionContext) -> bool {
        let ratio = context.strength_ratio();
        let casualties = context.own_casualty_percentage();

        ratio < self.personality.weights.retreat_threshold
            || casualties > self.personality.weights.casualty_threshold
    }

    /// Generate retreat orders for all units to HQ
    fn generate_retreat_orders(&self, context: &DecisionContext) -> Vec<Order> {
        let hq = context.hq_position();

        context
            .own_units()
            .iter()
            .filter(|u| u.can_fight() && !u.is_broken())
            .map(|u| Order::move_to(u.id, hq))
            .collect()
    }

    /// Clear pending orders for units that have received them
    pub fn clear_pending(&mut self, delivered_unit_ids: &[UnitId]) {
        self.pending_orders
            .retain(|id| !delivered_unit_ids.contains(id));
    }
}

impl BattleAI for AiCommander {
    fn process_tick(
        &mut self,
        context: &DecisionContext,
        current_tick: Tick,
        _events: &mut BattleEventLog,
    ) -> Vec<Order> {
        // Update phase transitions
        self.phase_manager.update(
            current_tick,
            context.strength_ratio(),
            context.own_casualty_percentage(),
        );

        // Only evaluate at intervals
        if !self.should_evaluate(current_tick) {
            return Vec::new();
        }

        self.last_evaluation_tick = Some(current_tick);

        // Evaluate and return orders
        self.evaluate_tactical(context)
    }

    fn personality(&self) -> &AiPersonality {
        &self.personality
    }

    fn ignores_fog_of_war(&self) -> bool {
        self.personality.difficulty.ignores_fog_of_war
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::ai::personality::AiPersonality;
    use crate::battle::courier::OrderType;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{
        Army, ArmyId, BattleFormation, BattleUnit, Element, FormationId, UnitId, UnitStance,
    };
    use crate::battle::visibility::ArmyVisibility;
    use crate::core::types::EntityId;

    fn create_test_unit_at(pos: BattleHexCoord, stance: UnitStance) -> BattleUnit {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = pos;
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stance = stance;
        unit
    }

    #[test]
    fn test_commander_creation() {
        let personality = AiPersonality::default();
        let commander = AiCommander::new(personality);
        assert!(!commander.ignores_fog_of_war());
    }

    #[test]
    fn test_commander_with_seed() {
        let personality = AiPersonality::default();
        let commander = AiCommander::with_seed(personality, 12345);
        assert!(!commander.ignores_fog_of_war());
    }

    #[test]
    fn test_commander_process_tick_returns_orders() {
        let personality = AiPersonality::default();
        let mut commander = AiCommander::new(personality);

        let own_army = Army::new(ArmyId::new(), EntityId::new());
        let enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, false);
        let mut events = BattleEventLog::new();

        let orders = commander.process_tick(&context, 0, &mut events);

        // With no units, should return no orders
        assert!(orders.is_empty());
    }

    #[test]
    fn test_commander_attacks_visible_enemy() {
        let mut personality = AiPersonality::default();
        personality.behavior.aggression = 0.8; // Aggressive
        personality.preferences.re_evaluation_interval = 1;
        personality.difficulty.mistake_chance = 0.0; // No mistakes for deterministic test

        let mut commander = AiCommander::new(personality);

        // Setup armies
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());
        let mut own_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        own_formation.units.push(create_test_unit_at(
            BattleHexCoord::new(5, 5),
            UnitStance::Formed,
        ));
        own_army.formations.push(own_formation);

        let mut enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        enemy_formation.units.push(create_test_unit_at(
            BattleHexCoord::new(10, 5),
            UnitStance::Formed,
        ));
        enemy_army.formations.push(enemy_formation);

        // Full visibility (ignores fog)
        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, true);
        let mut events = BattleEventLog::new();

        let orders = commander.process_tick(&context, 0, &mut events);

        assert!(!orders.is_empty(), "Should generate attack order");
        assert!(matches!(orders[0].order_type, OrderType::Attack(_)));
    }

    #[test]
    fn test_commander_retreats_when_outnumbered() {
        let mut personality = AiPersonality::default();
        personality.weights.retreat_threshold = 0.5;
        personality.preferences.re_evaluation_interval = 1;
        personality.difficulty.mistake_chance = 0.0; // No mistakes for deterministic test

        let mut commander = AiCommander::new(personality);

        // Small own army
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());
        own_army.hq_position = BattleHexCoord::new(0, 0);
        let mut own_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut small_unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        small_unit.position = BattleHexCoord::new(10, 10);
        small_unit
            .elements
            .push(Element::new(vec![EntityId::new(); 20]));
        small_unit.stance = UnitStance::Formed;
        own_formation.units.push(small_unit);
        own_army.formations.push(own_formation);

        // Large enemy army
        let mut enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut large_unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        large_unit.position = BattleHexCoord::new(12, 10);
        large_unit
            .elements
            .push(Element::new(vec![EntityId::new(); 200]));
        large_unit.stance = UnitStance::Formed;
        enemy_formation.units.push(large_unit);
        enemy_army.formations.push(enemy_formation);

        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, true);
        let mut events = BattleEventLog::new();

        let orders = commander.process_tick(&context, 0, &mut events);

        // Should retreat to HQ
        assert!(!orders.is_empty(), "Should generate retreat orders");
        if let OrderType::MoveTo(dest) = &orders[0].order_type {
            assert_eq!(*dest, BattleHexCoord::new(0, 0));
        } else {
            panic!("Expected MoveTo order for retreat");
        }
    }

    #[test]
    fn test_commander_respects_evaluation_interval() {
        let mut personality = AiPersonality::default();
        personality.preferences.re_evaluation_interval = 10;
        personality.behavior.aggression = 0.8;
        personality.difficulty.mistake_chance = 0.0; // No mistakes for deterministic test

        let mut commander = AiCommander::new(personality);

        let mut own_army = Army::new(ArmyId::new(), EntityId::new());
        let mut own_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        own_formation.units.push(create_test_unit_at(
            BattleHexCoord::new(5, 5),
            UnitStance::Formed,
        ));
        own_army.formations.push(own_formation);

        let mut enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        enemy_formation.units.push(create_test_unit_at(
            BattleHexCoord::new(10, 5),
            UnitStance::Formed,
        ));
        enemy_army.formations.push(enemy_formation);

        let visibility = ArmyVisibility::new();
        let mut events = BattleEventLog::new();

        // First tick should evaluate
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, true);
        let orders1 = commander.process_tick(&context, 0, &mut events);
        assert!(!orders1.is_empty());

        // Tick 5 should not evaluate (interval is 10)
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 5, true);
        let orders2 = commander.process_tick(&context, 5, &mut events);
        assert!(orders2.is_empty());

        // Tick 10 should evaluate
        commander.pending_orders.clear(); // Clear pending for re-test
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 10, true);
        let orders3 = commander.process_tick(&context, 10, &mut events);
        assert!(!orders3.is_empty());
    }

    #[test]
    fn test_clear_pending() {
        let personality = AiPersonality::default();
        let mut commander = AiCommander::new(personality);

        let unit1 = UnitId::new();
        let unit2 = UnitId::new();
        let unit3 = UnitId::new();

        commander.pending_orders.push(unit1);
        commander.pending_orders.push(unit2);
        commander.pending_orders.push(unit3);

        commander.clear_pending(&[unit1, unit3]);

        assert_eq!(commander.pending_orders.len(), 1);
        assert!(commander.pending_orders.contains(&unit2));
    }

    #[test]
    fn test_set_phase_manager() {
        use crate::battle::ai::phase_plans::{PhasePlan, PhasePlanManager, PhaseTransition};

        let personality = AiPersonality::default();
        let mut commander = AiCommander::new(personality);

        let mut manager = PhasePlanManager::new();
        manager.add_phase(PhasePlan {
            name: "Opening".to_string(),
            priority_targets: vec![],
            reserve_commitment: 0.0,
            aggression_modifier: -0.2,
            transition: PhaseTransition::TimeElapsed(10),
        });

        commander.set_phase_manager(manager);

        assert_eq!(commander.phase_manager.current_phase().name, "Opening");
    }

    #[test]
    fn test_defensive_commander_moves_towards_enemy() {
        let mut personality = AiPersonality::default();
        personality.behavior.aggression = 0.3; // Defensive (< 0.5)
        personality.preferences.re_evaluation_interval = 1;
        personality.difficulty.mistake_chance = 0.0; // No mistakes for deterministic test

        let mut commander = AiCommander::new(personality);

        // Setup armies
        let mut own_army = Army::new(ArmyId::new(), EntityId::new());
        let mut own_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        own_formation.units.push(create_test_unit_at(
            BattleHexCoord::new(0, 0),
            UnitStance::Formed,
        ));
        own_army.formations.push(own_formation);

        let mut enemy_army = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        enemy_formation.units.push(create_test_unit_at(
            BattleHexCoord::new(10, 0),
            UnitStance::Formed,
        ));
        enemy_army.formations.push(enemy_formation);

        let visibility = ArmyVisibility::new();
        let context = DecisionContext::new(&own_army, &enemy_army, &visibility, 0, true);
        let mut events = BattleEventLog::new();

        let orders = commander.process_tick(&context, 0, &mut events);

        assert!(!orders.is_empty(), "Should generate move order");
        // Defensive commander should move to halfway point
        assert!(matches!(orders[0].order_type, OrderType::MoveTo(_)));
    }
}
