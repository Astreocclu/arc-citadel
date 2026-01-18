# 12-BATTLE-PLANNING-TERRAIN-SPEC
> Battle planning AI and terrain effects for tactical combat

## Overview

This specification defines how battles are planned by AI commanders and how terrain affects tactical combat. The battle planning system creates emergent tactical behavior through the interaction of terrain properties, unit capabilities, and commander decision-making.

---

## Terrain System

### Terrain Types

```rust
/// Battle map terrain types with physical properties
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BattleTerrain {
    // Open terrain
    Grass,
    Dirt,
    Sand,
    Mud,

    // Elevated terrain
    Hill,
    Ridge,
    Cliff,

    // Obstructed terrain
    Forest,
    DenseForest,
    Rocks,
    Rubble,

    // Water terrain
    ShallowWater,
    DeepWater,
    Marsh,

    // Artificial terrain
    Road,
    Bridge,
    Wall,
    Building,
    Trench,

    // Special terrain
    Lava,
    Ice,
    MagicField,
}

/// Physical properties of terrain that affect gameplay
#[derive(Clone, Debug)]
pub struct TerrainProperties {
    /// Movement cost multiplier (1.0 = normal, 2.0 = half speed)
    pub movement_cost: f32,

    /// Cover value (0.0 = none, 1.0 = full cover)
    pub cover: f32,

    /// Concealment (0.0 = visible, 1.0 = hidden)
    pub concealment: f32,

    /// Elevation in meters
    pub elevation: f32,

    /// Whether units can stand on this terrain
    pub passable: bool,

    /// Whether projectiles can pass through
    pub blocks_projectiles: bool,

    /// Whether line of sight is blocked
    pub blocks_los: bool,

    /// Fatigue accumulation rate modifier
    pub fatigue_modifier: f32,

    /// Combat stance restrictions
    pub allows_mounted: bool,
    pub allows_formation: bool,
}

impl BattleTerrain {
    pub fn properties(&self) -> TerrainProperties {
        match self {
            Self::Grass => TerrainProperties {
                movement_cost: 1.0,
                cover: 0.0,
                concealment: 0.1,
                elevation: 0.0,
                passable: true,
                blocks_projectiles: false,
                blocks_los: false,
                fatigue_modifier: 1.0,
                allows_mounted: true,
                allows_formation: true,
            },
            Self::Hill => TerrainProperties {
                movement_cost: 1.5,
                cover: 0.2,
                concealment: 0.0,
                elevation: 5.0,
                passable: true,
                blocks_projectiles: false,
                blocks_los: false,  // From higher ground
                fatigue_modifier: 1.3,
                allows_mounted: true,
                allows_formation: true,
            },
            Self::Forest => TerrainProperties {
                movement_cost: 1.5,
                cover: 0.4,
                concealment: 0.6,
                elevation: 0.0,
                passable: true,
                blocks_projectiles: true,  // Partial
                blocks_los: true,          // Partial
                fatigue_modifier: 1.2,
                allows_mounted: false,
                allows_formation: false,
            },
            Self::DenseForest => TerrainProperties {
                movement_cost: 2.5,
                cover: 0.7,
                concealment: 0.9,
                elevation: 0.0,
                passable: true,
                blocks_projectiles: true,
                blocks_los: true,
                fatigue_modifier: 1.5,
                allows_mounted: false,
                allows_formation: false,
            },
            Self::ShallowWater => TerrainProperties {
                movement_cost: 2.0,
                cover: 0.0,
                concealment: 0.0,
                elevation: -0.5,
                passable: true,
                blocks_projectiles: false,
                blocks_los: false,
                fatigue_modifier: 2.0,
                allows_mounted: true,
                allows_formation: false,
            },
            Self::DeepWater => TerrainProperties {
                movement_cost: 5.0,
                cover: 0.0,
                concealment: 0.0,
                elevation: -2.0,
                passable: false,  // Unless swimming
                blocks_projectiles: false,
                blocks_los: false,
                fatigue_modifier: 3.0,
                allows_mounted: false,
                allows_formation: false,
            },
            Self::Wall => TerrainProperties {
                movement_cost: f32::INFINITY,
                cover: 1.0,
                concealment: 1.0,
                elevation: 3.0,
                passable: false,
                blocks_projectiles: true,
                blocks_los: true,
                fatigue_modifier: 1.0,
                allows_mounted: false,
                allows_formation: false,
            },
            Self::Road => TerrainProperties {
                movement_cost: 0.75,  // Faster than grass
                cover: 0.0,
                concealment: 0.0,
                elevation: 0.0,
                passable: true,
                blocks_projectiles: false,
                blocks_los: false,
                fatigue_modifier: 0.8,
                allows_mounted: true,
                allows_formation: true,
            },
            Self::Trench => TerrainProperties {
                movement_cost: 1.2,
                cover: 0.8,
                concealment: 0.3,
                elevation: -1.5,
                passable: true,
                blocks_projectiles: false,  // Only from front
                blocks_los: false,
                fatigue_modifier: 1.0,
                allows_mounted: false,
                allows_formation: false,
            },
            // ... other terrain types follow same pattern
            _ => TerrainProperties::default(),
        }
    }
}
```

### Elevation Effects

Elevation creates tactical advantages through physics-based mechanics:

```rust
/// Calculate combat modifiers based on elevation difference
pub fn elevation_effects(attacker_elevation: f32, defender_elevation: f32) -> ElevationModifiers {
    let height_diff = attacker_elevation - defender_elevation;

    ElevationModifiers {
        // Higher ground increases projectile range (gravity assists)
        range_modifier: 1.0 + (height_diff * 0.05).max(-0.3),

        // Higher ground makes targets easier to see
        visibility_modifier: 1.0 + (height_diff * 0.1).max(-0.5),

        // Downhill charges gain momentum
        charge_momentum: if height_diff > 0.0 {
            1.0 + height_diff * 0.1
        } else {
            1.0  // No penalty, just no bonus
        },

        // Uphill melee is harder (fighting gravity)
        melee_penalty: if height_diff < 0.0 {
            height_diff.abs() * 0.05  // 5% per meter of elevation
        } else {
            0.0
        },
    }
}

#[derive(Debug)]
pub struct ElevationModifiers {
    pub range_modifier: f32,
    pub visibility_modifier: f32,
    pub charge_momentum: f32,
    pub melee_penalty: f32,
}
```

### Line of Sight

```rust
/// Line of sight calculation with terrain occlusion
pub fn calculate_los(
    map: &BattleMap,
    from: Vec2,
    from_elevation: f32,
    to: Vec2,
    to_elevation: f32,
) -> LineOfSightResult {
    let direction = (to - from).normalize();
    let distance = (to - from).length();
    let steps = (distance / 0.5).ceil() as usize;  // Sample every 0.5 units

    let mut total_occlusion = 0.0;
    let mut blocking_terrain = Vec::new();

    for i in 1..steps {
        let t = i as f32 / steps as f32;
        let sample_pos = from + direction * (distance * t);
        let expected_height = from_elevation + (to_elevation - from_elevation) * t;

        let terrain = map.terrain_at(sample_pos);
        let terrain_props = terrain.properties();

        // Check if terrain blocks the sight line
        if terrain_props.blocks_los && terrain_props.elevation > expected_height {
            total_occlusion += 1.0;
            blocking_terrain.push((sample_pos, terrain));
        } else if terrain_props.concealment > 0.0 {
            // Partial occlusion from concealment
            total_occlusion += terrain_props.concealment * 0.3;
        }
    }

    LineOfSightResult {
        visible: total_occlusion < 1.0,
        visibility: (1.0 - total_occlusion).max(0.0),
        blockers: blocking_terrain,
        distance,
    }
}

#[derive(Debug)]
pub struct LineOfSightResult {
    pub visible: bool,
    pub visibility: f32,  // 0.0 = fully blocked, 1.0 = clear
    pub blockers: Vec<(Vec2, BattleTerrain)>,
    pub distance: f32,
}
```

---

## Battle Map

### Map Structure

```rust
/// A tactical battle map
pub struct BattleMap {
    /// Terrain grid
    terrain: Grid<BattleTerrain>,

    /// Pre-computed elevation map
    elevation: Grid<f32>,

    /// Dynamic features (can change during battle)
    features: Vec<MapFeature>,

    /// Zones of control
    zones: Vec<ControlZone>,

    /// Weather affecting the battle
    weather: Weather,

    /// Time of day (affects visibility)
    time_of_day: TimeOfDay,

    /// Map dimensions
    width: u32,
    height: u32,
}

/// Dynamic features on the map
#[derive(Clone, Debug)]
pub struct MapFeature {
    pub position: Vec2,
    pub feature_type: FeatureType,
    pub health: f32,  // Destructible features
}

#[derive(Clone, Copy, Debug)]
pub enum FeatureType {
    // Destructible
    Barricade,
    Palisade,
    Tower,
    Gate,

    // Interactive
    Bridge,
    Ford,
    Ladder,

    // Hazards
    Fire,
    Smoke,
    PoisonCloud,
}

/// Zones of tactical importance
#[derive(Clone, Debug)]
pub struct ControlZone {
    pub center: Vec2,
    pub radius: f32,
    pub importance: f32,
    pub zone_type: ZoneType,
    pub controlled_by: Option<Faction>,
}

#[derive(Clone, Copy, Debug)]
pub enum ZoneType {
    HighGround,      // Elevated position
    Chokepoint,      // Narrow passage
    Objective,       // Victory point
    SupplyCache,     // Resources
    Fortification,   // Defensive structure
    FlankingRoute,   // Side approach
}
```

### Map Analysis

```rust
impl BattleMap {
    /// Analyze map for tactical features
    pub fn analyze(&self) -> MapAnalysis {
        MapAnalysis {
            high_ground: self.find_high_ground(),
            chokepoints: self.find_chokepoints(),
            cover_positions: self.find_cover(),
            flanking_routes: self.find_flanking_routes(),
            kill_zones: self.identify_kill_zones(),
            defensible_positions: self.find_defensible_positions(),
        }
    }

    /// Find elevated positions that provide tactical advantage
    fn find_high_ground(&self) -> Vec<TacticalPosition> {
        let mut positions = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let elevation = self.elevation.get(x, y);
                let avg_neighbor_elevation = self.average_neighbor_elevation(x, y);

                // Position is high ground if significantly above surroundings
                if elevation - avg_neighbor_elevation > 2.0 {
                    positions.push(TacticalPosition {
                        center: Vec2::new(x as f32, y as f32),
                        value: elevation - avg_neighbor_elevation,
                        position_type: PositionType::HighGround,
                    });
                }
            }
        }

        // Sort by tactical value
        positions.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
        positions
    }

    /// Find narrow passages that restrict movement
    fn find_chokepoints(&self) -> Vec<TacticalPosition> {
        let mut chokepoints = Vec::new();

        // Use pathfinding analysis to find mandatory passage points
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let passable_neighbors = self.count_passable_neighbors(x, y);
                let terrain = self.terrain.get(x, y);

                // Chokepoint: passable with few passable neighbors
                if terrain.properties().passable && passable_neighbors <= 2 {
                    chokepoints.push(TacticalPosition {
                        center: Vec2::new(x as f32, y as f32),
                        value: 1.0 / (passable_neighbors as f32 + 0.1),
                        position_type: PositionType::Chokepoint,
                    });
                }
            }
        }

        chokepoints
    }

    /// Identify areas where defenders have significant advantage
    fn identify_kill_zones(&self) -> Vec<KillZone> {
        let mut kill_zones = Vec::new();

        // A kill zone is an area visible from multiple defensive positions
        // with limited cover for attackers

        for chokepoint in self.find_chokepoints() {
            let high_grounds = self.find_high_ground();
            let mut overwatch_positions = Vec::new();

            for hg in &high_grounds {
                let los = calculate_los(
                    self,
                    hg.center,
                    self.elevation.get(hg.center.x as u32, hg.center.y as u32),
                    chokepoint.center,
                    self.elevation.get(chokepoint.center.x as u32, chokepoint.center.y as u32),
                );

                if los.visible && los.distance < 50.0 {
                    overwatch_positions.push(hg.center);
                }
            }

            if overwatch_positions.len() >= 2 {
                kill_zones.push(KillZone {
                    center: chokepoint.center,
                    radius: 5.0,
                    overwatch_positions,
                    lethality: overwatch_positions.len() as f32 * 0.3,
                });
            }
        }

        kill_zones
    }
}

#[derive(Debug)]
pub struct TacticalPosition {
    pub center: Vec2,
    pub value: f32,
    pub position_type: PositionType,
}

#[derive(Debug)]
pub enum PositionType {
    HighGround,
    Chokepoint,
    Cover,
    Flanking,
    Defensible,
}

#[derive(Debug)]
pub struct KillZone {
    pub center: Vec2,
    pub radius: f32,
    pub overwatch_positions: Vec<Vec2>,
    pub lethality: f32,
}

#[derive(Debug)]
pub struct MapAnalysis {
    pub high_ground: Vec<TacticalPosition>,
    pub chokepoints: Vec<TacticalPosition>,
    pub cover_positions: Vec<TacticalPosition>,
    pub flanking_routes: Vec<Vec<Vec2>>,
    pub kill_zones: Vec<KillZone>,
    pub defensible_positions: Vec<TacticalPosition>,
}
```

---

## Battle Planning AI

### Commander Decision System

```rust
/// AI commander that plans and directs battles
pub struct BattleCommander {
    /// Faction this commander leads
    faction: Faction,

    /// Commander's tactical tendencies (from personality)
    tendencies: CommanderTendencies,

    /// Current battle plan
    plan: Option<BattlePlan>,

    /// Knowledge of enemy (updated by scouting)
    enemy_assessment: EnemyAssessment,

    /// Analysis of the battlefield
    map_analysis: MapAnalysis,
}

/// Commander personality affects tactical decisions
#[derive(Clone, Debug)]
pub struct CommanderTendencies {
    /// Aggressive (0.0) vs Cautious (1.0)
    pub caution: f32,

    /// Direct assault (0.0) vs Maneuver (1.0)
    pub maneuver_preference: f32,

    /// Hold ground (0.0) vs Seek initiative (1.0)
    pub initiative: f32,

    /// Individual heroics (0.0) vs Combined arms (1.0)
    pub coordination: f32,

    /// Preserve forces (0.0) vs Accept losses (1.0)
    pub risk_tolerance: f32,
}

impl BattleCommander {
    /// Create a battle plan based on situation analysis
    pub fn create_plan(
        &mut self,
        own_forces: &[Unit],
        known_enemies: &[Unit],
        map: &BattleMap,
        objectives: &[Objective],
    ) -> BattlePlan {
        // Analyze the situation
        self.map_analysis = map.analyze();
        self.enemy_assessment = self.assess_enemy(known_enemies);

        // Determine overall strategy
        let strategy = self.select_strategy(own_forces, objectives);

        // Generate tactical phases
        let phases = self.plan_phases(&strategy, own_forces, map, objectives);

        // Assign units to roles
        let unit_assignments = self.assign_units(own_forces, &phases);

        BattlePlan {
            strategy,
            phases,
            unit_assignments,
            contingencies: self.plan_contingencies(&strategy),
        }
    }

    /// Select overall battle strategy
    fn select_strategy(
        &self,
        own_forces: &[Unit],
        objectives: &[Objective],
    ) -> BattleStrategy {
        let force_ratio = self.calculate_force_ratio(own_forces);

        // Strategy selection based on situation and tendencies
        if force_ratio > 1.5 && self.tendencies.caution < 0.3 {
            // Strong advantage + aggressive = direct assault
            BattleStrategy::DirectAssault
        } else if force_ratio < 0.7 {
            // Outnumbered = defensive
            if self.tendencies.maneuver_preference > 0.6 {
                BattleStrategy::FightingWithdrawal
            } else {
                BattleStrategy::Defense
            }
        } else if self.tendencies.maneuver_preference > 0.5 {
            // Prefer maneuver = flanking
            if self.map_analysis.flanking_routes.len() >= 2 {
                BattleStrategy::DoubleEnvelopment
            } else {
                BattleStrategy::SingleEnvelopment
            }
        } else {
            // Default to methodical advance
            BattleStrategy::MethodicalAdvance
        }
    }

    /// Plan tactical phases for the battle
    fn plan_phases(
        &self,
        strategy: &BattleStrategy,
        own_forces: &[Unit],
        map: &BattleMap,
        objectives: &[Objective],
    ) -> Vec<BattlePhase> {
        match strategy {
            BattleStrategy::DirectAssault => vec![
                BattlePhase::Preparation {
                    duration: 30,  // Ticks
                    actions: vec![
                        PhaseAction::FormUp { formation: FormationType::Line },
                        PhaseAction::RangedSuppression { target_zone: objectives[0].position },
                    ],
                },
                BattlePhase::Advance {
                    objective: objectives[0].position,
                    formation: FormationType::Wedge,
                    speed: AdvanceSpeed::Fast,
                },
                BattlePhase::Assault {
                    target: objectives[0].position,
                    commitment: 0.8,  // Commit 80% of forces
                },
            ],

            BattleStrategy::SingleEnvelopment => {
                let flank_route = self.map_analysis.flanking_routes
                    .first()
                    .cloned()
                    .unwrap_or_default();

                vec![
                    BattlePhase::Preparation {
                        duration: 20,
                        actions: vec![
                            PhaseAction::DetachFlankingForce {
                                size_ratio: 0.3,
                                route: flank_route.clone(),
                            },
                        ],
                    },
                    BattlePhase::Feint {
                        direction: objectives[0].position,
                        commitment: 0.4,  // Light pressure
                    },
                    BattlePhase::FlankingManeuver {
                        route: flank_route,
                        timing: PhaseTiming::WhenMainEngaged,
                    },
                    BattlePhase::CombinedAssault {
                        main_target: objectives[0].position,
                        flank_target: self.enemy_assessment.rear_position,
                    },
                ]
            },

            BattleStrategy::Defense => {
                let defensive_positions = &self.map_analysis.defensible_positions;

                vec![
                    BattlePhase::OccupyPositions {
                        positions: defensive_positions.iter()
                            .take(3)
                            .map(|p| p.center)
                            .collect(),
                    },
                    BattlePhase::PrepareDefenses {
                        actions: vec![
                            PhaseAction::Entrench,
                            PhaseAction::EstablishFireLanes,
                        ],
                    },
                    BattlePhase::HoldLine {
                        fallback_threshold: 0.3,  // Retreat if 30% casualties
                        fallback_positions: defensive_positions.iter()
                            .skip(3)
                            .take(3)
                            .map(|p| p.center)
                            .collect(),
                    },
                ]
            },

            // ... other strategies
            _ => vec![],
        }
    }

    /// Assign units to roles in the plan
    fn assign_units(
        &self,
        units: &[Unit],
        phases: &[BattlePhase],
    ) -> Vec<UnitAssignment> {
        let mut assignments = Vec::new();

        // Categorize units by type
        let mut infantry = Vec::new();
        let mut cavalry = Vec::new();
        let mut ranged = Vec::new();
        let mut elite = Vec::new();

        for unit in units {
            match unit.unit_type {
                UnitType::Infantry => infantry.push(unit.id),
                UnitType::Cavalry => cavalry.push(unit.id),
                UnitType::Archer | UnitType::Crossbow => ranged.push(unit.id),
                UnitType::Elite => elite.push(unit.id),
                _ => infantry.push(unit.id),
            }
        }

        // Assign based on roles needed
        for phase in phases {
            match phase {
                BattlePhase::Advance { .. } => {
                    // Infantry forms the main line
                    for id in &infantry {
                        assignments.push(UnitAssignment {
                            unit_id: *id,
                            role: UnitRole::MainLine,
                            phase_orders: PhaseOrders::AdvanceWithFormation,
                        });
                    }
                },
                BattlePhase::FlankingManeuver { .. } => {
                    // Cavalry does flanking
                    for id in &cavalry {
                        assignments.push(UnitAssignment {
                            unit_id: *id,
                            role: UnitRole::Flanker,
                            phase_orders: PhaseOrders::FlankAndCharge,
                        });
                    }
                },
                // ... other phase assignments
                _ => {},
            }
        }

        // Ranged units provide support
        for id in &ranged {
            assignments.push(UnitAssignment {
                unit_id: *id,
                role: UnitRole::FireSupport,
                phase_orders: PhaseOrders::ProvideSupport,
            });
        }

        assignments
    }
}
```

### Battle Plan Structure

```rust
/// Complete battle plan
#[derive(Clone, Debug)]
pub struct BattlePlan {
    pub strategy: BattleStrategy,
    pub phases: Vec<BattlePhase>,
    pub unit_assignments: Vec<UnitAssignment>,
    pub contingencies: Vec<Contingency>,
}

#[derive(Clone, Copy, Debug)]
pub enum BattleStrategy {
    DirectAssault,
    MethodicalAdvance,
    SingleEnvelopment,
    DoubleEnvelopment,
    Defense,
    DelayingAction,
    FightingWithdrawal,
    Ambush,
    Siege,
}

#[derive(Clone, Debug)]
pub enum BattlePhase {
    Preparation {
        duration: u32,
        actions: Vec<PhaseAction>,
    },
    Advance {
        objective: Vec2,
        formation: FormationType,
        speed: AdvanceSpeed,
    },
    Assault {
        target: Vec2,
        commitment: f32,
    },
    Feint {
        direction: Vec2,
        commitment: f32,
    },
    FlankingManeuver {
        route: Vec<Vec2>,
        timing: PhaseTiming,
    },
    CombinedAssault {
        main_target: Vec2,
        flank_target: Vec2,
    },
    OccupyPositions {
        positions: Vec<Vec2>,
    },
    PrepareDefenses {
        actions: Vec<PhaseAction>,
    },
    HoldLine {
        fallback_threshold: f32,
        fallback_positions: Vec<Vec2>,
    },
}

#[derive(Clone, Debug)]
pub enum PhaseAction {
    FormUp { formation: FormationType },
    RangedSuppression { target_zone: Vec2 },
    DetachFlankingForce { size_ratio: f32, route: Vec<Vec2> },
    Entrench,
    EstablishFireLanes,
    ScoutAhead { range: f32 },
}

#[derive(Clone, Copy, Debug)]
pub enum FormationType {
    Line,
    Column,
    Wedge,
    Square,
    Crescent,
    Skirmish,
}

#[derive(Clone, Copy, Debug)]
pub enum AdvanceSpeed {
    Cautious,  // Maintain formation, slow
    Normal,    // Standard pace
    Fast,      // Quick advance, some disorder
    Charge,    // Full speed, formation breaks
}

#[derive(Clone, Copy, Debug)]
pub enum PhaseTiming {
    Immediate,
    WhenMainEngaged,
    OnSignal,
    AfterDuration(u32),
}
```

### Plan Execution and Adaptation

```rust
/// Monitors plan execution and adapts to changing conditions
pub struct PlanExecutor {
    plan: BattlePlan,
    current_phase_index: usize,
    phase_progress: f32,
    adaptations: Vec<PlanAdaptation>,
}

impl PlanExecutor {
    /// Update plan execution based on battle state
    pub fn tick(
        &mut self,
        battle_state: &BattleState,
        commander: &BattleCommander,
    ) -> Vec<UnitOrder> {
        // Check if current phase is complete
        if self.is_phase_complete(battle_state) {
            self.advance_phase();
        }

        // Check for conditions requiring adaptation
        if let Some(adaptation) = self.check_contingencies(battle_state) {
            self.apply_adaptation(adaptation);
        }

        // Generate orders for current phase
        self.generate_phase_orders(battle_state)
    }

    /// Check if plan needs adaptation
    fn check_contingencies(&self, state: &BattleState) -> Option<PlanAdaptation> {
        for contingency in &self.plan.contingencies {
            if contingency.condition.evaluate(state) {
                return Some(PlanAdaptation {
                    reason: contingency.trigger_reason.clone(),
                    new_orders: contingency.response.clone(),
                });
            }
        }
        None
    }

    /// Generate orders for units based on current phase
    fn generate_phase_orders(&self, state: &BattleState) -> Vec<UnitOrder> {
        let mut orders = Vec::new();
        let phase = &self.plan.phases[self.current_phase_index];

        for assignment in &self.plan.unit_assignments {
            let order = match (&phase, &assignment.role) {
                (BattlePhase::Advance { objective, formation, speed }, UnitRole::MainLine) => {
                    UnitOrder::Move {
                        unit_id: assignment.unit_id,
                        destination: *objective,
                        formation: Some(*formation),
                        speed: *speed,
                    }
                },
                (BattlePhase::Assault { target, .. }, UnitRole::MainLine) => {
                    UnitOrder::Attack {
                        unit_id: assignment.unit_id,
                        target: AttackTarget::Position(*target),
                        aggression: Aggression::High,
                    }
                },
                (BattlePhase::FlankingManeuver { route, .. }, UnitRole::Flanker) => {
                    UnitOrder::FollowPath {
                        unit_id: assignment.unit_id,
                        waypoints: route.clone(),
                        then: Box::new(UnitOrder::ChargeNearestEnemy {
                            unit_id: assignment.unit_id,
                        }),
                    }
                },
                (_, UnitRole::FireSupport) => {
                    // Ranged units always look for targets
                    UnitOrder::EngageTargetsOfOpportunity {
                        unit_id: assignment.unit_id,
                        priority: TargetPriority::Threatening,
                    }
                },
                _ => continue,
            };
            orders.push(order);
        }

        orders
    }
}

/// Contingency plans for when things go wrong
#[derive(Clone, Debug)]
pub struct Contingency {
    pub condition: ContingencyCondition,
    pub trigger_reason: String,
    pub response: Vec<UnitOrder>,
}

#[derive(Clone, Debug)]
pub enum ContingencyCondition {
    CasualtiesExceed(f32),
    FlankThreatened,
    ObjectiveLost(Vec2),
    CommanderDown,
    RouteBroken,
    AmmunitionLow,
    ReinforcementsArrived(Faction),
}

impl ContingencyCondition {
    pub fn evaluate(&self, state: &BattleState) -> bool {
        match self {
            Self::CasualtiesExceed(threshold) => {
                state.casualty_ratio() > *threshold
            },
            Self::FlankThreatened => {
                state.enemies_on_flank().len() > 0
            },
            Self::ObjectiveLost(pos) => {
                state.objective_controller(*pos) != state.own_faction
            },
            Self::CommanderDown => {
                !state.commander_alive()
            },
            Self::RouteBroken => {
                state.units_routing() > state.total_units() / 4
            },
            _ => false,
        }
    }
}
```

---

## Weather and Environmental Effects

```rust
/// Weather conditions affecting battle
#[derive(Clone, Debug)]
pub struct Weather {
    pub precipitation: Precipitation,
    pub wind_speed: f32,      // m/s
    pub wind_direction: f32,  // radians
    pub visibility: f32,      // 0.0-1.0
    pub temperature: f32,     // Celsius
}

#[derive(Clone, Copy, Debug)]
pub enum Precipitation {
    None,
    LightRain,
    HeavyRain,
    Snow,
    Sleet,
    Fog,
}

impl Weather {
    /// Effects on ranged combat
    pub fn ranged_modifiers(&self) -> RangedModifiers {
        RangedModifiers {
            // Wind affects arrow flight
            accuracy_penalty: self.wind_speed * 0.02,
            range_modifier: 1.0 - (self.wind_speed * 0.01),

            // Rain affects visibility and bowstrings
            visibility_penalty: match self.precipitation {
                Precipitation::None => 0.0,
                Precipitation::LightRain => 0.1,
                Precipitation::HeavyRain => 0.3,
                Precipitation::Fog => 0.5,
                Precipitation::Snow => 0.2,
                Precipitation::Sleet => 0.4,
            },

            // Wet bowstrings lose power
            power_modifier: match self.precipitation {
                Precipitation::HeavyRain | Precipitation::Sleet => 0.7,
                Precipitation::LightRain | Precipitation::Snow => 0.9,
                _ => 1.0,
            },
        }
    }

    /// Effects on movement
    pub fn movement_modifiers(&self) -> MovementModifiers {
        MovementModifiers {
            // Mud from rain
            terrain_penalty: match self.precipitation {
                Precipitation::HeavyRain => 0.3,
                Precipitation::LightRain => 0.1,
                Precipitation::Snow => 0.2,
                _ => 0.0,
            },

            // Cold exhausts troops faster
            fatigue_modifier: if self.temperature < 0.0 {
                1.0 + (self.temperature.abs() * 0.02)
            } else if self.temperature > 30.0 {
                1.0 + ((self.temperature - 30.0) * 0.03)
            } else {
                1.0
            },
        }
    }
}

/// Time of day affects visibility and fatigue
#[derive(Clone, Copy, Debug)]
pub enum TimeOfDay {
    Dawn,
    Morning,
    Midday,
    Afternoon,
    Dusk,
    Night,
}

impl TimeOfDay {
    pub fn visibility_modifier(&self) -> f32 {
        match self {
            Self::Midday | Self::Morning | Self::Afternoon => 1.0,
            Self::Dawn | Self::Dusk => 0.7,
            Self::Night => 0.3,
        }
    }

    pub fn morale_modifier(&self) -> f32 {
        match self {
            Self::Dawn => 1.1,  // Fresh troops
            Self::Night => 0.9, // Uncertainty
            _ => 1.0,
        }
    }
}
```

---

## Integration with Combat System

```rust
/// Bridge between battle planning and combat resolution
impl BattleMap {
    /// Get terrain effects for a combat at position
    pub fn combat_terrain_effects(&self, position: Vec2) -> CombatTerrainEffects {
        let terrain = self.terrain_at(position);
        let props = terrain.properties();

        CombatTerrainEffects {
            defender_cover: props.cover,
            movement_penalty: props.movement_cost - 1.0,
            elevation: props.elevation,
            allows_charge: props.allows_mounted && props.movement_cost < 1.5,
            allows_formation: props.allows_formation,
        }
    }

    /// Calculate advantage from relative positions
    pub fn positional_advantage(
        &self,
        attacker_pos: Vec2,
        defender_pos: Vec2,
    ) -> PositionalAdvantage {
        let attacker_terrain = self.terrain_at(attacker_pos);
        let defender_terrain = self.terrain_at(defender_pos);

        let attacker_elev = attacker_terrain.properties().elevation;
        let defender_elev = defender_terrain.properties().elevation;

        let elev_effects = elevation_effects(attacker_elev, defender_elev);

        PositionalAdvantage {
            attacker_elevation_bonus: elev_effects.charge_momentum,
            defender_cover: defender_terrain.properties().cover,
            melee_penalty: elev_effects.melee_penalty,
            charge_possible: (attacker_pos - defender_pos).length() > 5.0
                && attacker_terrain.properties().allows_mounted,
        }
    }
}

#[derive(Debug)]
pub struct CombatTerrainEffects {
    pub defender_cover: f32,
    pub movement_penalty: f32,
    pub elevation: f32,
    pub allows_charge: bool,
    pub allows_formation: bool,
}

#[derive(Debug)]
pub struct PositionalAdvantage {
    pub attacker_elevation_bonus: f32,
    pub defender_cover: f32,
    pub melee_penalty: f32,
    pub charge_possible: bool,
}
```

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [05-BATTLE-SYSTEM-SPEC](05-BATTLE-SYSTEM-SPEC.md) | Combat resolution using terrain effects |
| [06-BALANCE-COMPOSITION-SPEC](06-BALANCE-COMPOSITION-SPEC.md) | Physical property interactions |
| [07-CAMPAIGN-MAP-SPEC](07-CAMPAIGN-MAP-SPEC.md) | Strategic layer that spawns battles |
| [14-PHYSICS-COMBAT-SPEC](14-PHYSICS-COMBAT-SPEC.md) | Physics formulas for combat |
| [10-PERFORMANCE-ARCHITECTURE-SPEC](10-PERFORMANCE-ARCHITECTURE-SPEC.md) | Map analysis optimization |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
