# 07-CAMPAIGN-MAP-SPEC
> Strategic hex map, army movement, scouting, and supply lines

## Overview

The Campaign Layer provides strategic-scale gameplay between the Stronghold Layer (settlement management) and Battle Layer (tactical combat). Players deploy units across a hex-based map, manage supply lines, gather intelligence, and position forces for tactical engagements.

---

## Campaign Map Structure

### Hex Grid

```rust
pub struct CampaignMap {
    pub hexes: HashMap<HexCoord, HexTile>,
    pub fog_of_war: HashMap<EntityId, HashSet<HexCoord>>,  // Per-faction visibility
    pub supply_routes: Vec<SupplyRoute>,
    pub weather: WeatherState,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,  // Column (axial coordinates)
    pub r: i32,  // Row
}

impl HexCoord {
    /// Get all 6 adjacent hexes
    pub fn neighbors(&self) -> [HexCoord; 6] {
        [
            HexCoord { q: self.q + 1, r: self.r },
            HexCoord { q: self.q + 1, r: self.r - 1 },
            HexCoord { q: self.q, r: self.r - 1 },
            HexCoord { q: self.q - 1, r: self.r },
            HexCoord { q: self.q - 1, r: self.r + 1 },
            HexCoord { q: self.q, r: self.r + 1 },
        ]
    }

    /// Distance in hex steps
    pub fn distance(&self, other: &HexCoord) -> i32 {
        let dq = (self.q - other.q).abs();
        let dr = (self.r - other.r).abs();
        let ds = ((self.q + self.r) - (other.q + other.r)).abs();
        (dq + dr + ds) / 2
    }
}
```

### Hex Tiles

```rust
pub struct HexTile {
    pub terrain: CampaignTerrain,
    pub elevation: i32,           // Affects visibility, movement
    pub features: Vec<HexFeature>,
    pub controller: Option<FactionId>,
    pub contested: bool,
    pub resources: Vec<ResourceDeposit>,
}

#[derive(Clone, Copy)]
pub enum CampaignTerrain {
    Plains,
    Forest,
    Hills,
    Mountains,
    Swamp,
    Desert,
    Tundra,
    River,       // Crosses hex edges
    Coast,
    Ocean,
}

impl CampaignTerrain {
    /// Movement cost in days per hex
    pub fn movement_cost(&self) -> f32 {
        match self {
            Self::Plains => 1.0,
            Self::Forest => 2.0,
            Self::Hills => 2.0,
            Self::Mountains => 4.0,
            Self::Swamp => 3.0,
            Self::Desert => 2.0,
            Self::Tundra => 2.0,
            Self::River => 0.5,  // Along river = fast
            Self::Coast => 1.5,
            Self::Ocean => 0.0,  // Impassable without ships
        }
    }

    /// Visibility range modifier
    pub fn visibility_modifier(&self) -> f32 {
        match self {
            Self::Plains => 1.0,
            Self::Forest => 0.3,  // Hard to see through
            Self::Hills => 1.2,   // Can see from
            Self::Mountains => 1.5,
            Self::Swamp => 0.5,
            Self::Desert => 1.3,
            Self::Tundra => 1.2,
            Self::River => 1.0,
            Self::Coast => 1.0,
            Self::Ocean => 1.5,
        }
    }

    /// Supply cost multiplier
    pub fn supply_modifier(&self) -> f32 {
        match self {
            Self::Plains => 1.0,
            Self::Forest => 1.5,
            Self::Hills => 1.5,
            Self::Mountains => 3.0,
            Self::Swamp => 2.0,
            Self::Desert => 2.5,  // Water scarcity
            Self::Tundra => 2.0,
            Self::River => 0.5,   // River transport
            Self::Coast => 0.8,
            Self::Ocean => 0.3,   // Ship transport
        }
    }
}
```

### Hex Features

```rust
pub enum HexFeature {
    Settlement(SettlementRef),
    Fortress(FortressRef),
    Bridge,
    Ford,
    Pass,           // Through mountains
    Ruins,
    ResourceSite(ResourceType),
    Landmark(String),
}
```

---

## Army Management

### Army Structure

```rust
pub struct Army {
    pub id: ArmyId,
    pub faction: FactionId,
    pub position: HexCoord,
    pub units: Vec<UnitRef>,
    pub commander: Option<EntityId>,
    pub supply_level: f32,        // 0.0-1.0
    pub morale: f32,              // 0.0-1.0
    pub stance: ArmyStance,
    pub orders: Option<ArmyOrder>,
    pub movement_points: f32,
}

#[derive(Clone, Copy)]
pub enum ArmyStance {
    Aggressive,    // Engage enemies
    Defensive,     // Hold position
    Evasive,       // Avoid combat
    Raiding,       // Target supplies
}

pub enum ArmyOrder {
    MoveTo(HexCoord),
    Patrol(Vec<HexCoord>),
    Guard(HexCoord),
    Intercept(ArmyId),
    Retreat(HexCoord),
    Siege(HexCoord),
    Forage,
}
```

### Movement

```rust
impl Army {
    /// Calculate movement cost to adjacent hex
    pub fn movement_cost(&self, from: &HexTile, to: &HexTile) -> f32 {
        let base_cost = to.terrain.movement_cost();

        // Elevation change affects cost
        let elevation_change = (to.elevation - from.elevation).abs() as f32;
        let elevation_cost = 1.0 + (elevation_change * 0.2);

        // Army size affects speed
        let size_penalty = 1.0 + (self.units.len() as f32 * 0.01);

        // Supply affects movement
        let supply_penalty = if self.supply_level < 0.3 {
            1.5  // Low supply slows movement
        } else {
            1.0
        };

        base_cost * elevation_cost * size_penalty * supply_penalty
    }

    /// Execute movement order
    pub fn execute_movement(&mut self, map: &CampaignMap, dt_days: f32) -> MovementResult {
        let Some(ArmyOrder::MoveTo(target)) = &self.orders else {
            return MovementResult::NoOrders;
        };

        if self.position == *target {
            self.orders = None;
            return MovementResult::Arrived;
        }

        // Find path
        let path = pathfind(&self.position, target, map, |from, to| {
            self.movement_cost(from, to)
        });

        if path.is_empty() {
            return MovementResult::NoPath;
        }

        // Consume movement points
        let next_hex = path[0];
        let from_tile = &map.hexes[&self.position];
        let to_tile = &map.hexes[&next_hex];
        let cost = self.movement_cost(from_tile, to_tile);

        self.movement_points += dt_days;

        if self.movement_points >= cost {
            self.movement_points -= cost;
            self.position = next_hex;

            // Check for interception
            if let Some(enemy) = map.enemy_army_at(&next_hex, self.faction) {
                return MovementResult::Intercepted(enemy);
            }

            MovementResult::Moved
        } else {
            MovementResult::Moving
        }
    }
}

pub enum MovementResult {
    NoOrders,
    NoPath,
    Moving,
    Moved,
    Arrived,
    Intercepted(ArmyId),
}
```

---

## Supply System

### Supply Lines

```rust
pub struct SupplyRoute {
    pub source: HexCoord,      // Settlement with supply depot
    pub path: Vec<HexCoord>,
    pub capacity: f32,         // Units of supply per day
    pub security: f32,         // 0.0-1.0 (risk of raiding)
}

pub struct SupplySystem;

impl SupplySystem {
    /// Calculate supply reaching an army
    pub fn calculate_supply(
        army: &Army,
        routes: &[SupplyRoute],
        map: &CampaignMap,
    ) -> f32 {
        let mut total_supply = 0.0;

        for route in routes {
            // Check if route reaches army
            if !route.path.contains(&army.position) {
                continue;
            }

            // Calculate supply loss over distance
            let distance = route.path.iter()
                .position(|&h| h == army.position)
                .unwrap_or(0);

            let mut supply = route.capacity;

            // Supply degrades over distance
            for hex in route.path.iter().take(distance) {
                let tile = &map.hexes[hex];
                supply *= 1.0 - (tile.terrain.supply_modifier() * 0.1);
            }

            // Security affects delivery
            supply *= route.security;

            total_supply += supply;
        }

        total_supply
    }

    /// Update army supply status
    pub fn update_army_supply(army: &mut Army, supply_available: f32, dt_days: f32) {
        // Supply consumption based on army size
        let consumption = army.units.len() as f32 * 0.1 * dt_days;

        let net_supply = supply_available * dt_days - consumption;

        army.supply_level = (army.supply_level + net_supply / 10.0).clamp(0.0, 1.0);

        // Low supply affects morale
        if army.supply_level < 0.2 {
            army.morale -= 0.05 * dt_days;
        }
    }
}
```

### Foraging

```rust
impl Army {
    /// Attempt to forage from local terrain
    pub fn forage(&mut self, tile: &HexTile, season: Season) -> f32 {
        let base_forage = match tile.terrain {
            CampaignTerrain::Plains => 0.3,
            CampaignTerrain::Forest => 0.4,
            CampaignTerrain::Hills => 0.2,
            CampaignTerrain::Swamp => 0.1,
            CampaignTerrain::Desert => 0.05,
            CampaignTerrain::Tundra => 0.1,
            _ => 0.1,
        };

        let season_modifier = match season {
            Season::Spring => 1.0,
            Season::Summer => 1.2,
            Season::Autumn => 1.5,
            Season::Winter => 0.3,
        };

        base_forage * season_modifier
    }
}
```

---

## Intelligence & Fog of War

### Visibility System

```rust
pub struct IntelSystem;

impl IntelSystem {
    /// Calculate visible hexes for a faction
    pub fn calculate_visibility(
        map: &CampaignMap,
        faction: FactionId,
        armies: &[Army],
        settlements: &[Settlement],
    ) -> HashSet<HexCoord> {
        let mut visible = HashSet::new();

        // Armies provide visibility
        for army in armies.iter().filter(|a| a.faction == faction) {
            let range = Self::army_sight_range(army, &map.hexes[&army.position]);
            for hex in hexes_in_range(&army.position, range) {
                if Self::has_line_of_sight(map, &army.position, &hex) {
                    visible.insert(hex);
                }
            }
        }

        // Settlements provide visibility
        for settlement in settlements.iter().filter(|s| s.faction == faction) {
            let range = settlement.sight_range();
            for hex in hexes_in_range(&settlement.position, range) {
                if Self::has_line_of_sight(map, &settlement.position, &hex) {
                    visible.insert(hex);
                }
            }
        }

        visible
    }

    fn army_sight_range(army: &Army, tile: &HexTile) -> i32 {
        let base_range = 2;
        let elevation_bonus = tile.elevation / 100;  // +1 range per 100m elevation
        let terrain_mod = tile.terrain.visibility_modifier();

        ((base_range + elevation_bonus) as f32 * terrain_mod) as i32
    }

    fn has_line_of_sight(map: &CampaignMap, from: &HexCoord, to: &HexCoord) -> bool {
        let from_tile = &map.hexes[from];
        let to_tile = &map.hexes[to];

        // Check intermediate hexes for blocking terrain
        for hex in hexes_along_line(from, to) {
            let tile = &map.hexes[&hex];

            // Higher terrain blocks LoS to lower terrain
            if tile.elevation > from_tile.elevation && tile.elevation > to_tile.elevation {
                return false;
            }

            // Dense terrain blocks LoS
            if matches!(tile.terrain, CampaignTerrain::Forest | CampaignTerrain::Mountains) {
                if &hex != from && &hex != to {
                    return false;
                }
            }
        }

        true
    }
}
```

### Scouting

```rust
pub struct Scout {
    pub id: EntityId,
    pub position: HexCoord,
    pub faction: FactionId,
    pub skill: f32,         // 0.0-1.0
    pub stealth: f32,       // 0.0-1.0
    pub orders: ScoutOrder,
}

pub enum ScoutOrder {
    Explore(HexCoord),
    Shadow(ArmyId),
    Patrol(Vec<HexCoord>),
    Infiltrate(HexCoord),  // Settlement/fortress
}

impl Scout {
    /// Gather intelligence about a hex
    pub fn gather_intel(&self, tile: &HexTile) -> IntelReport {
        let detail_level = self.skill * tile.terrain.visibility_modifier();

        IntelReport {
            position: tile.position,
            terrain: tile.terrain,
            features: if detail_level > 0.3 {
                Some(tile.features.clone())
            } else {
                None
            },
            army_presence: if detail_level > 0.5 {
                tile.army_presence()
            } else {
                None
            },
            army_composition: if detail_level > 0.8 {
                tile.army_details()
            } else {
                None
            },
            timestamp: current_tick(),
        }
    }

    /// Check if scout is detected
    pub fn detection_check(&self, tile: &HexTile, enemy_presence: bool) -> bool {
        if !enemy_presence {
            return false;
        }

        let detection_chance = 0.3 - (self.stealth * 0.25);
        let terrain_bonus = match tile.terrain {
            CampaignTerrain::Forest => -0.2,
            CampaignTerrain::Mountains => -0.15,
            CampaignTerrain::Plains => 0.1,
            CampaignTerrain::Desert => 0.15,
            _ => 0.0,
        };

        rand::random::<f32>() < (detection_chance + terrain_bonus).max(0.05)
    }
}

pub struct IntelReport {
    pub position: HexCoord,
    pub terrain: CampaignTerrain,
    pub features: Option<Vec<HexFeature>>,
    pub army_presence: Option<bool>,
    pub army_composition: Option<ArmyComposition>,
    pub timestamp: Tick,
}
```

---

## Diplomacy System (Outline)

```rust
pub struct Faction {
    pub id: FactionId,
    pub name: String,
    pub capital: HexCoord,
    pub relations: HashMap<FactionId, Relation>,
    pub treaties: Vec<Treaty>,
}

pub struct Relation {
    pub standing: f32,       // -1.0 to 1.0 (hostile to allied)
    pub trust: f32,          // 0.0 to 1.0
    pub fear: f32,           // 0.0 to 1.0
}

pub enum Treaty {
    NonAggression { expires: Tick },
    TradeAgreement { terms: TradeTerm },
    MilitaryAlliance { defensive: bool, offensive: bool },
    Vassalage { lord: FactionId, vassal: FactionId },
    Peace { terms: Vec<PeaceTerm> },
}

pub enum DiplomaticAction {
    ProposeAlliance,
    DeclarWar,
    OfferPeace(Vec<PeaceTerm>),
    DemandTribute,
    ProposeMarriage,
    ExchangeHostages,
    BreakTreaty(Treaty),
}
```

---

## Campaign Events

```rust
pub enum CampaignEvent {
    // Movement
    ArmyMoved { army: ArmyId, from: HexCoord, to: HexCoord },
    ArmyIntercepted { attacker: ArmyId, defender: ArmyId },

    // Combat
    BattleInitiated { armies: Vec<ArmyId>, location: HexCoord },
    BattleResolved { winner: Option<FactionId>, casualties: BattleCasualties },
    SiegeStarted { army: ArmyId, target: HexCoord },
    SiegeResolved { success: bool },

    // Intelligence
    ScoutReport { faction: FactionId, report: IntelReport },
    ScoutCaptured { scout: EntityId, by: FactionId },
    ArmyDetected { army: ArmyId, by: FactionId },

    // Supply
    SupplyRouteEstablished { route: SupplyRoute },
    SupplyRouteRaided { route: SupplyRoute, loss: f32 },
    ArmyStarving { army: ArmyId },

    // Diplomatic
    TreatyProposed { from: FactionId, to: FactionId, treaty: Treaty },
    TreatyAccepted { treaty: Treaty },
    TreatyBroken { treaty: Treaty, by: FactionId },
    WarDeclared { attacker: FactionId, defender: FactionId },

    // Territorial
    HexCaptured { hex: HexCoord, by: FactionId },
    SettlementCaptured { settlement: HexCoord, by: FactionId },
}
```

---

## Campaign Tick

```rust
pub fn campaign_tick(state: &mut CampaignState, dt_days: f32) {
    // 1. Process army movement
    for army in &mut state.armies {
        let result = army.execute_movement(&state.map, dt_days);
        if let MovementResult::Intercepted(enemy_id) = result {
            state.events.push(CampaignEvent::ArmyIntercepted {
                attacker: enemy_id,
                defender: army.id,
            });
        }
    }

    // 2. Update supply
    for army in &mut state.armies {
        let supply = SupplySystem::calculate_supply(army, &state.supply_routes, &state.map);
        SupplySystem::update_army_supply(army, supply, dt_days);
    }

    // 3. Update visibility
    for faction in &state.factions {
        let visibility = IntelSystem::calculate_visibility(
            &state.map,
            faction.id,
            &state.armies,
            &state.settlements,
        );
        state.map.fog_of_war.insert(faction.id, visibility);
    }

    // 4. Process scout actions
    for scout in &mut state.scouts {
        // Scout logic
    }

    // 5. Check for battle initiation
    for hex in state.contested_hexes() {
        let armies_present = state.armies_at(&hex);
        if has_opposing_factions(&armies_present) {
            state.events.push(CampaignEvent::BattleInitiated {
                armies: armies_present.iter().map(|a| a.id).collect(),
                location: hex,
            });
        }
    }

    // 6. Weather effects
    state.weather.update(dt_days);

    // 7. Advance campaign time
    state.current_day += dt_days;
}
```

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [01-GAME-DESIGN-DOCUMENT](01-GAME-DESIGN-DOCUMENT.md) | Campaign layer overview |
| [05-BATTLE-SYSTEM-SPEC](05-BATTLE-SYSTEM-SPEC.md) | Battle initiation |
| [12-BATTLE-PLANNING-TERRAIN-SPEC](12-BATTLE-PLANNING-TERRAIN-SPEC.md) | Terrain transition |
| [16-RESOURCE-ECONOMY-SPEC](16-RESOURCE-ECONOMY-SPEC.md) | Supply production |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
