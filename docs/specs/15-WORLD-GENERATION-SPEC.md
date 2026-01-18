# 15-WORLD-GENERATION-SPEC
> Procedural world generation through simulation

## Overview

Arc Citadel generates worlds through **simulated history** rather than random placement. Geographic features create constraints, civilizations emerge and interact, and history unfolds over simulated centuries. The resulting world has internally consistent geography, cultures, and political situations.

---

## Generation Philosophy

### Simulation Over Randomization

```
Traditional Approach:
    Random terrain → Place civilizations → Add backstory

Arc Citadel Approach:
    Physical formation → Climate simulation → Life emergence →
    Civilization development → Historical events → Current state
```

Each layer emerges from the previous, creating coherent worlds where geography explains culture and history explains politics.

---

## Geographic Generation

### Tectonic Simulation

```rust
/// Generate continental structure through plate tectonics
pub struct TectonicSimulator {
    plates: Vec<TectonicPlate>,
    world_size: (u32, u32),
    time_scale: f32,  // Million years per step
}

#[derive(Clone, Debug)]
pub struct TectonicPlate {
    pub id: PlateId,
    pub cells: HashSet<(u32, u32)>,
    pub velocity: Vec2,       // Direction and speed
    pub plate_type: PlateType,
    pub density: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum PlateType {
    Continental,  // Lighter, forms landmass
    Oceanic,      // Denser, subducts
}

impl TectonicSimulator {
    /// Simulate plate movement and interactions
    pub fn simulate_step(&mut self) -> TectonicEvents {
        let mut events = TectonicEvents::default();

        // Move plates
        for plate in &mut self.plates {
            plate.move_by_velocity(self.time_scale);
        }

        // Detect collisions
        for i in 0..self.plates.len() {
            for j in (i + 1)..self.plates.len() {
                if let Some(boundary) = self.detect_boundary(&self.plates[i], &self.plates[j]) {
                    let event = self.resolve_boundary(boundary, i, j);
                    events.add(event);
                }
            }
        }

        events
    }

    fn resolve_boundary(&self, boundary: PlateBoundary, i: usize, j: usize) -> GeologicEvent {
        let plate_a = &self.plates[i];
        let plate_b = &self.plates[j];

        match boundary.boundary_type {
            BoundaryType::Convergent => {
                // Plates colliding
                match (plate_a.plate_type, plate_b.plate_type) {
                    (PlateType::Continental, PlateType::Continental) => {
                        // Mountain building
                        GeologicEvent::MountainFormation {
                            location: boundary.center,
                            magnitude: (plate_a.velocity - plate_b.velocity).length(),
                        }
                    },
                    (PlateType::Oceanic, PlateType::Continental)
                    | (PlateType::Continental, PlateType::Oceanic) => {
                        // Subduction - volcanic arc
                        GeologicEvent::VolcanicArc {
                            location: boundary.center,
                            subducting_plate: if plate_a.plate_type == PlateType::Oceanic { i } else { j },
                        }
                    },
                    (PlateType::Oceanic, PlateType::Oceanic) => {
                        // Ocean trench and island arc
                        GeologicEvent::IslandArc {
                            location: boundary.center,
                        }
                    },
                }
            },
            BoundaryType::Divergent => {
                // Plates separating
                GeologicEvent::RiftFormation {
                    location: boundary.center,
                    creates_ocean: plate_a.plate_type == PlateType::Continental,
                }
            },
            BoundaryType::Transform => {
                // Sliding past
                GeologicEvent::FaultLine {
                    location: boundary.center,
                    earthquake_potential: (plate_a.velocity - plate_b.velocity).length(),
                }
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlateBoundary {
    pub center: Vec2,
    pub boundary_type: BoundaryType,
    pub length: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum BoundaryType {
    Convergent,  // Colliding
    Divergent,   // Separating
    Transform,   // Sliding
}

#[derive(Clone, Debug)]
pub enum GeologicEvent {
    MountainFormation { location: Vec2, magnitude: f32 },
    VolcanicArc { location: Vec2, subducting_plate: usize },
    IslandArc { location: Vec2 },
    RiftFormation { location: Vec2, creates_ocean: bool },
    FaultLine { location: Vec2, earthquake_potential: f32 },
}
```

### Erosion and Weathering

```rust
/// Simulate erosion over geological time
pub struct ErosionSimulator {
    heightmap: Grid<f32>,
    rock_hardness: Grid<f32>,
    rainfall: Grid<f32>,
}

impl ErosionSimulator {
    /// Run hydraulic erosion
    pub fn erode(&mut self, iterations: u32) {
        for _ in 0..iterations {
            self.rainfall_step();
            self.flow_step();
            self.sediment_step();
        }
    }

    fn rainfall_step(&mut self) {
        // Add water based on rainfall patterns
        for y in 0..self.heightmap.height {
            for x in 0..self.heightmap.width {
                let rain = self.rainfall.get(x, y);
                self.water.add(x, y, rain);
            }
        }
    }

    fn flow_step(&mut self) {
        // Water flows downhill
        let mut new_water = Grid::new(self.heightmap.width, self.heightmap.height, 0.0);
        let mut erosion = Grid::new(self.heightmap.width, self.heightmap.height, 0.0);

        for y in 1..self.heightmap.height - 1 {
            for x in 1..self.heightmap.width - 1 {
                let current_height = self.heightmap.get(x, y) + self.water.get(x, y);

                // Find lowest neighbor
                let mut lowest = (x, y);
                let mut lowest_height = current_height;

                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = (x as i32 + dx) as u32;
                    let ny = (y as i32 + dy) as u32;
                    let neighbor_height = self.heightmap.get(nx, ny) + self.water.get(nx, ny);

                    if neighbor_height < lowest_height {
                        lowest = (nx, ny);
                        lowest_height = neighbor_height;
                    }
                }

                if lowest != (x, y) {
                    // Calculate flow rate based on slope
                    let slope = current_height - lowest_height;
                    let flow = self.water.get(x, y).min(slope * 0.5);

                    // Transfer water
                    new_water.add(lowest.0, lowest.1, flow);

                    // Erosion proportional to flow and slope
                    let erode_amount = flow * slope * (1.0 / self.rock_hardness.get(x, y));
                    erosion.add(x, y, -erode_amount);
                    erosion.add(lowest.0, lowest.1, erode_amount * 0.8);  // Some sediment lost
                }
            }
        }

        // Apply changes
        self.water = new_water;
        self.heightmap.apply_delta(&erosion);
    }
}
```

### Climate Zones

```rust
/// Determine climate based on geography
pub fn calculate_climate(
    position: (u32, u32),
    latitude: f32,          // -90 to 90
    elevation: f32,         // meters
    distance_to_ocean: f32, // km
    prevailing_wind: Vec2,
    mountain_shadow: bool,  // Behind mountains from wind
) -> Climate {
    // Base temperature from latitude
    let base_temp = 30.0 - latitude.abs() * 0.6;

    // Elevation reduces temperature (~6.5°C per 1000m)
    let elevation_effect = elevation * 0.0065;

    // Ocean moderates temperature
    let continental_effect = (distance_to_ocean / 500.0).min(1.0) * 15.0;

    let avg_temp = base_temp - elevation_effect - continental_effect * (latitude / 90.0).abs();

    // Precipitation
    let base_precip = if distance_to_ocean < 100.0 { 1500.0 } else { 800.0 };
    let rain_shadow = if mountain_shadow { 0.3 } else { 1.0 };
    let latitude_factor = 1.0 - (latitude.abs() - 30.0).abs() / 60.0;  // Peak at tropics

    let annual_precip = base_precip * rain_shadow * latitude_factor;

    // Determine biome
    let biome = determine_biome(avg_temp, annual_precip, latitude);

    Climate {
        average_temperature: avg_temp,
        temperature_range: 20.0 + continental_effect,
        annual_precipitation: annual_precip,
        biome,
        growing_season_days: calculate_growing_season(avg_temp, latitude),
    }
}

fn determine_biome(temp: f32, precip: f32, latitude: f32) -> Biome {
    match (temp, precip) {
        (t, _) if t < -10.0 => Biome::IceCap,
        (t, p) if t < 0.0 && p < 300.0 => Biome::Tundra,
        (t, p) if t < 10.0 && p > 500.0 => Biome::Taiga,
        (t, p) if t > 20.0 && p > 2000.0 => Biome::TropicalRainforest,
        (t, p) if t > 20.0 && p > 1000.0 => Biome::TropicalSeasonal,
        (t, p) if t > 15.0 && p < 250.0 => Biome::Desert,
        (t, p) if t > 10.0 && p < 500.0 => Biome::Steppe,
        (t, p) if p > 1000.0 => Biome::TemperateForest,
        (t, p) if p > 500.0 => Biome::TemperateGrassland,
        _ => Biome::Shrubland,
    }
}

#[derive(Clone, Debug)]
pub struct Climate {
    pub average_temperature: f32,
    pub temperature_range: f32,
    pub annual_precipitation: f32,
    pub biome: Biome,
    pub growing_season_days: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Biome {
    IceCap,
    Tundra,
    Taiga,
    TemperateForest,
    TemperateGrassland,
    Mediterranean,
    Steppe,
    Desert,
    Shrubland,
    TropicalSeasonal,
    TropicalRainforest,
    Savanna,
    Mangrove,
}
```

---

## Resource Distribution

### Geological Resources

```rust
/// Place resources based on geological history
pub struct ResourceDistributor {
    geology: GeologyMap,
    resources: Grid<Vec<ResourceDeposit>>,
}

impl ResourceDistributor {
    /// Generate resource deposits based on geology
    pub fn distribute_resources(&mut self) {
        for y in 0..self.geology.height {
            for x in 0..self.geology.width {
                let geology = self.geology.get(x, y);
                let deposits = self.generate_deposits(&geology);
                self.resources.set(x, y, deposits);
            }
        }
    }

    fn generate_deposits(&self, geology: &GeologyInfo) -> Vec<ResourceDeposit> {
        let mut deposits = Vec::new();

        // Igneous rocks from volcanic activity
        if geology.volcanic_activity > 0.5 {
            deposits.push(ResourceDeposit {
                resource: ResourceType::Obsidian,
                quantity: geology.volcanic_activity * 1000.0,
                quality: 0.8,
                depth: 0.0,  // Surface
            });

            if rand::random::<f32>() < geology.volcanic_activity * 0.3 {
                deposits.push(ResourceDeposit {
                    resource: ResourceType::Gold,
                    quantity: 100.0,
                    quality: 0.9,
                    depth: 50.0,  // Veins
                });
            }
        }

        // Sedimentary deposits
        if geology.sediment_depth > 100.0 {
            deposits.push(ResourceDeposit {
                resource: ResourceType::Clay,
                quantity: geology.sediment_depth * 10.0,
                quality: 0.6,
                depth: 0.0,
            });

            if geology.sediment_age > 100_000_000.0 {
                // Old sediments may have coal
                deposits.push(ResourceDeposit {
                    resource: ResourceType::Coal,
                    quantity: geology.sediment_depth * 5.0,
                    quality: 0.7,
                    depth: 20.0,
                });
            }
        }

        // Mountain building concentrates metals
        if geology.tectonic_activity > 0.7 {
            deposits.push(ResourceDeposit {
                resource: ResourceType::Iron,
                quantity: geology.tectonic_activity * 500.0,
                quality: 0.6,
                depth: 30.0,
            });

            deposits.push(ResourceDeposit {
                resource: ResourceType::Copper,
                quantity: geology.tectonic_activity * 200.0,
                quality: 0.7,
                depth: 25.0,
            });
        }

        // Alluvial deposits in river valleys
        if geology.is_river_valley {
            deposits.push(ResourceDeposit {
                resource: ResourceType::Sand,
                quantity: 10000.0,
                quality: 0.5,
                depth: 0.0,
            });

            // Placer gold in some rivers
            if rand::random::<f32>() < 0.1 {
                deposits.push(ResourceDeposit {
                    resource: ResourceType::Gold,
                    quantity: 10.0,
                    quality: 0.95,  // High purity placer
                    depth: 0.0,
                });
            }
        }

        deposits
    }
}

#[derive(Clone, Debug)]
pub struct ResourceDeposit {
    pub resource: ResourceType,
    pub quantity: f32,     // Units available
    pub quality: f32,      // Purity/grade
    pub depth: f32,        // Meters underground
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceType {
    // Metals
    Iron,
    Copper,
    Tin,
    Gold,
    Silver,

    // Stone
    Granite,
    Marble,
    Limestone,
    Obsidian,

    // Fuel
    Coal,
    Peat,

    // Other
    Clay,
    Sand,
    Salt,
    Gems,
}
```

---

## Civilization Emergence

### Settlement Site Selection

```rust
/// Evaluate a location for settlement potential
pub fn evaluate_settlement_site(
    location: (u32, u32),
    world: &WorldMap,
) -> SettlementScore {
    let mut score = SettlementScore::default();

    // Water access is critical
    let water = world.nearest_water(location);
    score.water = match water.water_type {
        WaterType::River => 1.0,
        WaterType::Lake => 0.8,
        WaterType::Spring => 0.7,
        WaterType::Well => 0.5,
        WaterType::None => 0.0,
    } * (1.0 / (water.distance + 1.0)).min(1.0);

    // Fertile land for agriculture
    let climate = world.climate_at(location);
    score.agriculture = match climate.biome {
        Biome::TemperateGrassland => 0.9,
        Biome::TemperateForest => 0.7,
        Biome::Mediterranean => 0.8,
        Biome::TropicalSeasonal => 0.6,
        Biome::Savanna => 0.5,
        Biome::Steppe => 0.4,
        Biome::Desert | Biome::Tundra | Biome::IceCap => 0.1,
        _ => 0.5,
    } * (climate.growing_season_days as f32 / 200.0).min(1.0);

    // Defensibility
    let terrain = world.terrain_at(location);
    score.defense = match terrain {
        Terrain::Hill => 0.8,
        Terrain::Cliff => 0.9,
        Terrain::Peninsula => 0.85,
        Terrain::Island => 0.7,
        Terrain::Plains => 0.3,
        Terrain::Forest => 0.5,
        _ => 0.4,
    };

    // Resource access
    let resources = world.resources_at(location);
    score.resources = (resources.len() as f32 * 0.1).min(1.0)
        + resources.iter().map(|r| r.quality * 0.05).sum::<f32>();

    // Trade potential (crossroads, ports)
    score.trade = world.trade_potential(location);

    score.total = score.water * 0.3
        + score.agriculture * 0.25
        + score.defense * 0.15
        + score.resources * 0.15
        + score.trade * 0.15;

    score
}

#[derive(Clone, Debug, Default)]
pub struct SettlementScore {
    pub water: f32,
    pub agriculture: f32,
    pub defense: f32,
    pub resources: f32,
    pub trade: f32,
    pub total: f32,
}
```

### Civilization Simulation

```rust
/// Simulate civilization development over time
pub struct CivilizationSimulator {
    civilizations: Vec<Civilization>,
    world: WorldMap,
    year: i32,
}

impl CivilizationSimulator {
    /// Advance history by one era
    pub fn simulate_era(&mut self) -> Vec<HistoricalEvent> {
        let mut events = Vec::new();

        for civ in &mut self.civilizations {
            // Population growth
            let growth = self.calculate_growth(civ);
            civ.population = (civ.population as f32 * growth) as u64;

            // Technology advancement
            if let Some(tech) = self.tech_discovery_check(civ) {
                events.push(HistoricalEvent::TechDiscovery {
                    civilization: civ.id,
                    technology: tech,
                    year: self.year,
                });
                civ.technologies.insert(tech);
            }

            // Expansion pressure
            if civ.population > civ.territory_capacity() {
                if let Some(expansion) = self.attempt_expansion(civ) {
                    events.push(expansion);
                }
            }
        }

        // Inter-civilization interactions
        events.extend(self.resolve_conflicts());
        events.extend(self.resolve_diplomacy());

        self.year += 50;  // Era is 50 years
        events
    }

    fn calculate_growth(&self, civ: &Civilization) -> f32 {
        let base_growth = 1.02;  // 2% per era

        // Modify by technology
        let tech_bonus = if civ.technologies.contains(&Technology::Agriculture) { 1.05 } else { 1.0 }
            * if civ.technologies.contains(&Technology::Medicine) { 1.02 } else { 1.0 };

        // Modify by resources
        let resource_factor = civ.food_surplus() / civ.population as f32;

        // Wars reduce population
        let war_factor = if civ.at_war { 0.95 } else { 1.0 };

        base_growth * tech_bonus * resource_factor.min(1.1) * war_factor
    }

    fn attempt_expansion(&mut self, civ: &mut Civilization) -> Option<HistoricalEvent> {
        // Find best adjacent territory
        let candidates = self.world.adjacent_territories(&civ.territory);

        let best = candidates.into_iter()
            .filter(|t| !self.is_claimed(*t))
            .max_by(|a, b| {
                evaluate_settlement_site(*a, &self.world).total
                    .partial_cmp(&evaluate_settlement_site(*b, &self.world).total)
                    .unwrap()
            })?;

        civ.territory.push(best);

        Some(HistoricalEvent::Expansion {
            civilization: civ.id,
            territory: best,
            year: self.year,
        })
    }

    fn resolve_conflicts(&mut self) -> Vec<HistoricalEvent> {
        let mut events = Vec::new();

        // Check for territorial disputes
        for i in 0..self.civilizations.len() {
            for j in (i + 1)..self.civilizations.len() {
                let civ_a = &self.civilizations[i];
                let civ_b = &self.civilizations[j];

                if self.territories_adjacent(civ_a, civ_b) {
                    let tension = self.calculate_tension(civ_a, civ_b);

                    if tension > 0.7 && rand::random::<f32>() < tension - 0.5 {
                        // War breaks out
                        events.push(HistoricalEvent::War {
                            attacker: civ_a.id,
                            defender: civ_b.id,
                            cause: WarCause::TerritorialDispute,
                            year: self.year,
                        });

                        // Resolve war (simplified)
                        let outcome = self.resolve_war(i, j);
                        events.push(outcome);
                    }
                }
            }
        }

        events
    }
}

#[derive(Clone, Debug)]
pub struct Civilization {
    pub id: CivId,
    pub name: String,
    pub population: u64,
    pub territory: Vec<(u32, u32)>,
    pub capital: (u32, u32),
    pub technologies: HashSet<Technology>,
    pub government: GovernmentType,
    pub culture: CultureId,
    pub at_war: bool,
}

#[derive(Clone, Debug)]
pub enum HistoricalEvent {
    CivFounded { civilization: CivId, location: (u32, u32), year: i32 },
    TechDiscovery { civilization: CivId, technology: Technology, year: i32 },
    Expansion { civilization: CivId, territory: (u32, u32), year: i32 },
    War { attacker: CivId, defender: CivId, cause: WarCause, year: i32 },
    WarEnded { winner: CivId, loser: CivId, terms: PeaceTerms, year: i32 },
    Alliance { civ_a: CivId, civ_b: CivId, year: i32 },
    Revolution { civilization: CivId, new_government: GovernmentType, year: i32 },
    Disaster { civilization: CivId, disaster_type: DisasterType, casualties: u64, year: i32 },
    GoldenAge { civilization: CivId, year: i32 },
    Collapse { civilization: CivId, year: i32 },
}
```

---

## Culture Generation

### Cultural Traits

```rust
/// Generate culture based on environment and history
pub fn generate_culture(
    environment: &Environment,
    history: &[HistoricalEvent],
    founding_species: Species,
) -> Culture {
    let mut culture = Culture::default();

    // Environment shapes values
    match environment.biome {
        Biome::Desert => {
            culture.values.insert(CulturalValue::Hospitality, 0.9);
            culture.values.insert(CulturalValue::Frugality, 0.8);
            culture.values.insert(CulturalValue::ClanLoyalty, 0.7);
        },
        Biome::TemperateForest => {
            culture.values.insert(CulturalValue::NatureRespect, 0.7);
            culture.values.insert(CulturalValue::Independence, 0.6);
        },
        Biome::Steppe => {
            culture.values.insert(CulturalValue::MartialProwess, 0.8);
            culture.values.insert(CulturalValue::Mobility, 0.9);
            culture.values.insert(CulturalValue::HorsemanshipHonor, 0.7);
        },
        _ => {}
    }

    // History shapes culture
    let war_count = history.iter().filter(|e| matches!(e, HistoricalEvent::War { .. })).count();
    if war_count > 5 {
        culture.values.insert(CulturalValue::MartialProwess, 0.8);
        culture.values.insert(CulturalValue::MilitaryService, 0.7);
    }

    let golden_ages = history.iter().filter(|e| matches!(e, HistoricalEvent::GoldenAge { .. })).count();
    if golden_ages > 0 {
        culture.values.insert(CulturalValue::ArtisticExpression, 0.7);
        culture.values.insert(CulturalValue::Learning, 0.6);
    }

    // Species influences
    culture.species_base = founding_species;

    // Generate naming conventions
    culture.naming = generate_naming_convention(environment, &culture.values);

    // Generate religion
    culture.religion = generate_religion(environment, &culture.values);

    culture
}

#[derive(Clone, Debug, Default)]
pub struct Culture {
    pub values: HashMap<CulturalValue, f32>,
    pub species_base: Species,
    pub naming: NamingConvention,
    pub religion: Religion,
    pub customs: Vec<Custom>,
    pub taboos: Vec<Taboo>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CulturalValue {
    Hospitality,
    Frugality,
    ClanLoyalty,
    NatureRespect,
    Independence,
    MartialProwess,
    Mobility,
    HorsemanshipHonor,
    MilitaryService,
    ArtisticExpression,
    Learning,
    TradeSkill,
    Piety,
    AncestorVeneration,
}

#[derive(Clone, Debug)]
pub struct NamingConvention {
    pub given_first: bool,
    pub uses_patronymic: bool,
    pub clan_names: bool,
    pub title_position: TitlePosition,
    pub phoneme_patterns: Vec<PhonemePattern>,
}

#[derive(Clone, Debug)]
pub struct Religion {
    pub polytheism: bool,
    pub deities: Vec<Deity>,
    pub afterlife_belief: AfterlifeBelief,
    pub sacred_places: Vec<SacredPlaceType>,
    pub rituals: Vec<Ritual>,
}
```

---

## Historical Events

### Event Generation

```rust
/// Generate specific historical events
pub struct EventGenerator {
    rng: StdRng,
    world: WorldMap,
}

impl EventGenerator {
    /// Generate events that shaped the world
    pub fn generate_history(&mut self, duration_years: i32) -> Vec<HistoricalEvent> {
        let mut events = Vec::new();
        let mut year = -duration_years;

        while year < 0 {
            // Major events
            if self.rng.gen::<f32>() < 0.02 {
                events.push(self.generate_major_event(year));
            }

            // Minor events
            if self.rng.gen::<f32>() < 0.1 {
                events.push(self.generate_minor_event(year));
            }

            year += 10;  // Decade steps
        }

        events
    }

    fn generate_major_event(&mut self, year: i32) -> HistoricalEvent {
        match self.rng.gen_range(0..5) {
            0 => self.generate_great_war(year),
            1 => self.generate_empire_rise(year),
            2 => self.generate_cataclysm(year),
            3 => self.generate_great_migration(year),
            4 => self.generate_golden_age(year),
            _ => unreachable!(),
        }
    }

    fn generate_great_war(&mut self, year: i32) -> HistoricalEvent {
        // Select civilizations
        let civs: Vec<_> = self.world.civilizations().collect();
        let attacker = civs[self.rng.gen_range(0..civs.len())].id;
        let defender = civs[self.rng.gen_range(0..civs.len())].id;

        if attacker == defender {
            return self.generate_minor_event(year);  // Retry
        }

        HistoricalEvent::War {
            attacker,
            defender,
            cause: WarCause::Conquest,
            year,
        }
    }

    fn generate_cataclysm(&mut self, year: i32) -> HistoricalEvent {
        let disaster = match self.rng.gen_range(0..4) {
            0 => DisasterType::Plague,
            1 => DisasterType::Volcanic,
            2 => DisasterType::Earthquake,
            3 => DisasterType::Famine,
            _ => unreachable!(),
        };

        let affected_civ = self.world.civilizations().next().unwrap().id;

        HistoricalEvent::Disaster {
            civilization: affected_civ,
            disaster_type: disaster,
            casualties: self.rng.gen_range(10000..1000000),
            year,
        }
    }
}
```

---

## World State Export

### Game-Ready World

```rust
/// Export generated world for gameplay
pub struct WorldExporter;

impl WorldExporter {
    /// Convert simulation state to game world
    pub fn export(
        terrain: &Grid<Terrain>,
        climate: &Grid<Climate>,
        resources: &Grid<Vec<ResourceDeposit>>,
        civilizations: &[Civilization],
        history: &[HistoricalEvent],
    ) -> GameWorld {
        GameWorld {
            campaign_map: Self::build_campaign_map(terrain, climate, resources),
            factions: Self::build_factions(civilizations),
            settlements: Self::build_settlements(civilizations),
            lore: Self::compile_lore(history, civilizations),
            current_year: 0,
        }
    }

    fn build_campaign_map(
        terrain: &Grid<Terrain>,
        climate: &Grid<Climate>,
        resources: &Grid<Vec<ResourceDeposit>>,
    ) -> CampaignMap {
        let mut map = CampaignMap::new(terrain.width, terrain.height);

        for y in 0..terrain.height {
            for x in 0..terrain.width {
                let cell = CampaignCell {
                    terrain: convert_terrain(terrain.get(x, y)),
                    climate: climate.get(x, y).clone(),
                    resources: resources.get(x, y).clone(),
                    owner: None,
                    settlement: None,
                };
                map.set(x, y, cell);
            }
        }

        map
    }

    fn build_factions(civilizations: &[Civilization]) -> Vec<Faction> {
        civilizations.iter().map(|civ| {
            Faction {
                id: FactionId(civ.id.0),
                name: civ.name.clone(),
                culture: civ.culture,
                government: civ.government,
                territory: civ.territory.clone(),
                relations: HashMap::new(),
            }
        }).collect()
    }

    fn compile_lore(
        history: &[HistoricalEvent],
        civilizations: &[Civilization],
    ) -> WorldLore {
        WorldLore {
            timeline: history.iter().cloned().collect(),
            notable_figures: Self::generate_notable_figures(history),
            legends: Self::generate_legends(history, civilizations),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameWorld {
    pub campaign_map: CampaignMap,
    pub factions: Vec<Faction>,
    pub settlements: Vec<Settlement>,
    pub lore: WorldLore,
    pub current_year: i32,
}

#[derive(Clone, Debug)]
pub struct WorldLore {
    pub timeline: Vec<HistoricalEvent>,
    pub notable_figures: Vec<HistoricalFigure>,
    pub legends: Vec<Legend>,
}
```

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [07-CAMPAIGN-MAP-SPEC](07-CAMPAIGN-MAP-SPEC.md) | Generated map structure |
| [08-GENETICS-SYSTEM-SPEC](08-GENETICS-SYSTEM-SPEC.md) | Species generation |
| [16-RESOURCE-ECONOMY-SPEC](16-RESOURCE-ECONOMY-SPEC.md) | Resource distribution |
| [17-SOCIAL-MEMORY-SPEC](17-SOCIAL-MEMORY-SPEC.md) | Historical memory system |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
