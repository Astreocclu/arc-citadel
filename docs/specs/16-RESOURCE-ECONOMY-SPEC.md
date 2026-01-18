# 16-RESOURCE-ECONOMY-SPEC
> Physical resource flow: materials have mass, transport has cost, storage has limits

## Overview

Arc Citadel's economy emerges from **physical resource constraints**, not abstract numbers. Resources have mass, volume, and decay rates. Transport requires labor and time. Storage has physical limits. This specification defines resource systems that produce emergent economic behavior through property composition.

---

## Core Philosophy

**Resources are physical objects with physical properties.**

```rust
// ✅ CORRECT: Physical resource modeling
let transport_time = distance / (carrier_speed * terrain_factor);
let transport_cost = carrier_wages * transport_time + wear_on_cart;
let storage_limit = warehouse_volume / resource_volume_per_unit;

// ❌ FORBIDDEN: Abstract economic modifiers
let price = base_price * supply_modifier * demand_modifier; // NEVER DO THIS
```

Economic effects emerge from physical constraints:
- Distant resources cost more (transport time and labor)
- Bulk goods are harder to move (mass affects transport)
- Perishables decay (time-based loss)
- Storage is limited (physical space)

---

## Resource Properties

### Base Resource Definition

```rust
/// Physical properties of a resource type
#[derive(Clone, Debug)]
pub struct ResourceProperties {
    pub id: ResourceId,
    pub name: String,

    // Physical properties
    pub mass_per_unit: f32,        // kg per unit
    pub volume_per_unit: f32,      // m³ per unit
    pub stackable: bool,           // can units be stacked?
    pub max_stack: u32,            // if stackable, how high?

    // Decay properties
    pub decay_rate: f32,           // 0.0 = stable, 1.0 = rots in a day
    pub decay_conditions: DecayConditions,

    // Storage requirements
    pub storage_type: StorageType,
    pub temperature_sensitivity: Option<TemperatureRange>,
    pub moisture_sensitivity: Option<MoistureRange>,

    // Origin and processing
    pub category: ResourceCategory,
    pub raw_material: Option<ResourceId>,  // what this is made from
    pub processing_complexity: f32,         // skill/time to process
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResourceCategory {
    // Raw materials
    RawOre,           // iron ore, copper ore, etc.
    RawStone,         // granite, marble, limestone
    RawWood,          // logs, branches
    RawFiber,         // cotton, flax, wool
    RawHide,          // animal skins
    RawFood,          // grain, vegetables, meat

    // Processed materials
    Metal,            // iron bars, steel ingots
    ProcessedStone,   // cut stone blocks
    Lumber,           // planks, beams
    Textile,          // cloth, rope
    Leather,          // tanned hides
    ProcessedFood,    // flour, preserved meat

    // Finished goods
    Tool,             // hammers, saws
    Weapon,           // swords, bows
    Armor,            // chainmail, plate
    Clothing,         // shirts, boots
    Furniture,        // tables, chairs

    // Special
    Currency,         // coins, gems
    Luxury,           // jewelry, art
    Contraband,       // illegal goods
}

#[derive(Clone, Copy, Debug)]
pub enum StorageType {
    OpenAir,          // can be left outside
    Covered,          // needs roof
    Enclosed,         // needs walls and roof
    Climate,          // needs temperature control
    Secure,           // needs locks/guards
    Hazardous,        // needs special handling
}

#[derive(Clone, Copy, Debug)]
pub struct DecayConditions {
    pub base_rate: f32,               // decay per day at optimal conditions
    pub temperature_factor: f32,      // how much temp affects decay
    pub moisture_factor: f32,         // how much moisture affects decay
    pub light_factor: f32,            // how much light affects decay
}
```

### Resource Examples

```rust
impl ResourceProperties {
    pub fn iron_ore() -> Self {
        Self {
            id: ResourceId::IronOre,
            name: "Iron Ore".into(),
            mass_per_unit: 50.0,      // 50 kg per unit (heavy!)
            volume_per_unit: 0.02,    // 20 liters per unit
            stackable: true,
            max_stack: 100,
            decay_rate: 0.0,          // doesn't decay
            decay_conditions: DecayConditions::none(),
            storage_type: StorageType::OpenAir,
            temperature_sensitivity: None,
            moisture_sensitivity: None,
            category: ResourceCategory::RawOre,
            raw_material: None,
            processing_complexity: 0.0,
        }
    }

    pub fn iron_ingot() -> Self {
        Self {
            id: ResourceId::IronIngot,
            name: "Iron Ingot".into(),
            mass_per_unit: 10.0,      // 10 kg per ingot
            volume_per_unit: 0.0013,  // ~1.3 liters (dense!)
            stackable: true,
            max_stack: 50,
            decay_rate: 0.001,        // very slow rust
            decay_conditions: DecayConditions {
                base_rate: 0.001,
                temperature_factor: 0.0,
                moisture_factor: 2.0,  // rusts faster when wet
                light_factor: 0.0,
            },
            storage_type: StorageType::Covered,
            temperature_sensitivity: None,
            moisture_sensitivity: Some(MoistureRange { max: 0.5 }),
            category: ResourceCategory::Metal,
            raw_material: Some(ResourceId::IronOre),
            processing_complexity: 0.4,  // requires smelting skill
        }
    }

    pub fn raw_meat() -> Self {
        Self {
            id: ResourceId::RawMeat,
            name: "Raw Meat".into(),
            mass_per_unit: 2.0,       // 2 kg per unit
            volume_per_unit: 0.002,   // 2 liters
            stackable: true,
            max_stack: 20,
            decay_rate: 0.3,          // decays quickly!
            decay_conditions: DecayConditions {
                base_rate: 0.3,
                temperature_factor: 3.0,  // heat accelerates decay a lot
                moisture_factor: 1.0,
                light_factor: 0.5,
            },
            storage_type: StorageType::Climate,
            temperature_sensitivity: Some(TemperatureRange {
                min: -5.0,
                max: 5.0,
            }),
            moisture_sensitivity: None,
            category: ResourceCategory::RawFood,
            raw_material: None,
            processing_complexity: 0.0,
        }
    }

    pub fn preserved_meat() -> Self {
        Self {
            id: ResourceId::PreservedMeat,
            name: "Preserved Meat".into(),
            mass_per_unit: 1.5,       // lighter after drying/salting
            volume_per_unit: 0.0015,
            stackable: true,
            max_stack: 50,
            decay_rate: 0.01,         // much slower decay
            decay_conditions: DecayConditions {
                base_rate: 0.01,
                temperature_factor: 0.5,
                moisture_factor: 2.0,  // still affected by moisture
                light_factor: 0.0,
            },
            storage_type: StorageType::Covered,
            temperature_sensitivity: None,
            moisture_sensitivity: Some(MoistureRange { max: 0.3 }),
            category: ResourceCategory::ProcessedFood,
            raw_material: Some(ResourceId::RawMeat),
            processing_complexity: 0.2,
        }
    }
}
```

---

## Storage System

### Physical Storage

```rust
/// A physical storage location
#[derive(Debug)]
pub struct StorageContainer {
    pub id: StorageId,
    pub storage_type: StorageType,
    pub position: Vec3,

    // Physical limits
    pub max_volume: f32,           // m³
    pub max_mass: f32,             // kg (structural limit)

    // Current state
    pub contents: Vec<ResourceStack>,
    pub current_volume: f32,
    pub current_mass: f32,

    // Environment
    pub temperature: f32,          // Celsius
    pub moisture: f32,             // 0.0 - 1.0
    pub light_exposure: f32,       // 0.0 - 1.0
    pub security_level: f32,       // 0.0 - 1.0
}

#[derive(Debug, Clone)]
pub struct ResourceStack {
    pub resource: ResourceId,
    pub quantity: u32,
    pub quality: f32,              // 0.0 - 1.0, degrades with decay
    pub age_days: f32,             // how old this batch is
}

impl StorageContainer {
    /// Check if resource can be stored here
    pub fn can_store(&self, resource: &ResourceProperties, quantity: u32) -> StorageResult {
        // Check storage type compatibility
        if !self.is_compatible_storage_type(resource.storage_type) {
            return StorageResult::IncompatibleType;
        }

        // Check temperature requirements
        if let Some(temp_range) = &resource.temperature_sensitivity {
            if self.temperature < temp_range.min || self.temperature > temp_range.max {
                return StorageResult::TemperatureOutOfRange;
            }
        }

        // Check moisture requirements
        if let Some(moist_range) = &resource.moisture_sensitivity {
            if self.moisture > moist_range.max {
                return StorageResult::TooMoist;
            }
        }

        // Check volume
        let required_volume = resource.volume_per_unit * quantity as f32;
        if self.current_volume + required_volume > self.max_volume {
            return StorageResult::InsufficientVolume {
                available: self.max_volume - self.current_volume,
                required: required_volume,
            };
        }

        // Check mass
        let required_mass = resource.mass_per_unit * quantity as f32;
        if self.current_mass + required_mass > self.max_mass {
            return StorageResult::InsufficientCapacity {
                available: self.max_mass - self.current_mass,
                required: required_mass,
            };
        }

        StorageResult::CanStore
    }

    fn is_compatible_storage_type(&self, required: StorageType) -> bool {
        // Higher-tier storage can handle lower-tier requirements
        match (self.storage_type, required) {
            (_, StorageType::OpenAir) => true,
            (StorageType::Covered, StorageType::Covered) => true,
            (StorageType::Enclosed, StorageType::Covered) => true,
            (StorageType::Climate, StorageType::Covered) => true,
            (StorageType::Enclosed, StorageType::Enclosed) => true,
            (StorageType::Climate, StorageType::Enclosed) => true,
            (StorageType::Climate, StorageType::Climate) => true,
            (StorageType::Secure, _) => true,  // secure storage handles anything
            _ => false,
        }
    }

    /// Process decay for all stored resources
    pub fn process_decay(&mut self, dt_days: f32, resources: &ResourceRegistry) {
        for stack in &mut self.contents {
            let props = resources.get(stack.resource);

            // Calculate decay rate based on conditions
            let decay = props.decay_conditions;
            let temp_effect = if self.temperature > 20.0 {
                (self.temperature - 20.0) / 20.0 * decay.temperature_factor
            } else {
                0.0
            };
            let moisture_effect = self.moisture * decay.moisture_factor;
            let light_effect = self.light_exposure * decay.light_factor;

            let total_decay_rate = decay.base_rate
                * (1.0 + temp_effect + moisture_effect + light_effect);

            // Apply decay to quality
            stack.quality -= total_decay_rate * dt_days;
            stack.quality = stack.quality.max(0.0);
            stack.age_days += dt_days;

            // If quality reaches 0, resource is spoiled
            // (handled separately in cleanup)
        }

        // Remove fully spoiled resources
        self.contents.retain(|stack| stack.quality > 0.0);
        self.recalculate_totals();
    }

    fn recalculate_totals(&mut self) {
        // Recalculate volume and mass after changes
        // Implementation details...
    }
}

pub enum StorageResult {
    CanStore,
    IncompatibleType,
    TemperatureOutOfRange,
    TooMoist,
    InsufficientVolume { available: f32, required: f32 },
    InsufficientCapacity { available: f32, required: f32 },
}
```

---

## Transport System

### Physical Transport

```rust
/// A transport job moving resources between locations
#[derive(Debug)]
pub struct TransportJob {
    pub id: TransportJobId,
    pub resource: ResourceId,
    pub quantity: u32,

    pub origin: StorageId,
    pub destination: StorageId,

    pub carrier: Option<EntityId>,
    pub cart: Option<CartId>,

    pub state: TransportState,
    pub started_at: SimTime,
    pub estimated_completion: SimTime,
}

#[derive(Debug)]
pub enum TransportState {
    Queued,
    Loading,
    InTransit { progress: f32 },
    Unloading,
    Complete,
    Failed { reason: TransportFailure },
}

/// Physical transport capacity
#[derive(Debug)]
pub struct CarryingCapacity {
    pub max_mass: f32,             // kg
    pub max_volume: f32,           // m³
    pub base_speed: f32,           // m/s unloaded
    pub encumbrance_factor: f32,   // speed reduction per kg
}

impl CarryingCapacity {
    /// Human carrying by hand
    pub fn human_carry() -> Self {
        Self {
            max_mass: 30.0,           // 30 kg comfortable carry
            max_volume: 0.05,         // 50 liters
            base_speed: 1.4,          // walking speed
            encumbrance_factor: 0.015, // loses 1.5% speed per kg
        }
    }

    /// Human with pack animal
    pub fn pack_mule() -> Self {
        Self {
            max_mass: 150.0,          // mule can carry 150 kg
            max_volume: 0.3,          // ~300 liters with panniers
            base_speed: 1.2,          // slower than walking
            encumbrance_factor: 0.003, // more efficient per kg
        }
    }

    /// Cart pulled by draft animal
    pub fn ox_cart() -> Self {
        Self {
            max_mass: 1000.0,         // 1 ton capacity
            max_volume: 2.0,          // 2 cubic meters
            base_speed: 0.8,          // slow but steady
            encumbrance_factor: 0.0001, // very efficient for bulk
        }
    }

    /// Calculate actual travel speed
    pub fn travel_speed(&self, carried_mass: f32) -> f32 {
        let speed_loss = carried_mass * self.encumbrance_factor;
        (self.base_speed - speed_loss).max(self.base_speed * 0.3)
    }
}

/// Calculate transport time and cost
pub fn calculate_transport(
    resource: &ResourceProperties,
    quantity: u32,
    carrier: &CarryingCapacity,
    path: &Path,
    carrier_wage_per_hour: f32,
) -> TransportEstimate {
    let total_mass = resource.mass_per_unit * quantity as f32;
    let total_volume = resource.volume_per_unit * quantity as f32;

    // Check if carrier can handle this load
    if total_mass > carrier.max_mass || total_volume > carrier.max_volume {
        // Need multiple trips
        let trips_by_mass = (total_mass / carrier.max_mass).ceil() as u32;
        let trips_by_volume = (total_volume / carrier.max_volume).ceil() as u32;
        let trips = trips_by_mass.max(trips_by_volume);

        // Each trip carries portion of load
        let mass_per_trip = total_mass / trips as f32;
        let speed = carrier.travel_speed(mass_per_trip);
        let time_per_trip = path.calculate_travel_time(speed);
        let total_time = time_per_trip * trips as f32 * 2.0; // round trips

        return TransportEstimate {
            trips,
            total_time_hours: total_time / 3600.0,
            labor_cost: (total_time / 3600.0) * carrier_wage_per_hour,
            decay_during_transport: calculate_decay_during_transport(
                resource, total_time, trips
            ),
        };
    }

    // Single trip
    let speed = carrier.travel_speed(total_mass);
    let time_seconds = path.calculate_travel_time(speed);

    TransportEstimate {
        trips: 1,
        total_time_hours: time_seconds / 3600.0,
        labor_cost: (time_seconds / 3600.0) * carrier_wage_per_hour,
        decay_during_transport: calculate_decay_during_transport(
            resource, time_seconds, 1
        ),
    }
}

fn calculate_decay_during_transport(
    resource: &ResourceProperties,
    time_seconds: f32,
    trips: u32,
) -> f32 {
    // Resources decay during transport (exposed to elements)
    let days = time_seconds / 86400.0;
    let exposure_multiplier = 2.0; // worse conditions during transport

    resource.decay_rate * days * exposure_multiplier * trips as f32
}

#[derive(Debug)]
pub struct TransportEstimate {
    pub trips: u32,
    pub total_time_hours: f32,
    pub labor_cost: f32,
    pub decay_during_transport: f32,
}
```

---

## Production System

### Physical Production

```rust
/// A production recipe converting inputs to outputs
#[derive(Debug)]
pub struct Recipe {
    pub id: RecipeId,
    pub name: String,

    pub inputs: Vec<RecipeInput>,
    pub outputs: Vec<RecipeOutput>,

    pub work_time_hours: f32,      // base time at skill level 5
    pub skill_required: SkillId,
    pub min_skill_level: u8,

    pub tools_required: Vec<ToolRequirement>,
    pub facility_required: Option<FacilityType>,

    // Byproducts and waste
    pub waste: Vec<RecipeOutput>,  // slag, sawdust, etc.
    pub fuel_consumed: Option<FuelRequirement>,
}

#[derive(Debug)]
pub struct RecipeInput {
    pub resource: ResourceId,
    pub quantity: u32,
    pub min_quality: f32,          // minimum acceptable quality
    pub consumed: bool,            // false for tools/catalysts
}

#[derive(Debug)]
pub struct RecipeOutput {
    pub resource: ResourceId,
    pub quantity: u32,
    pub quality_from_inputs: bool, // quality depends on input quality
    pub quality_from_skill: bool,  // quality depends on worker skill
}

#[derive(Debug)]
pub struct FuelRequirement {
    pub fuel_type: FuelType,
    pub amount_per_hour: f32,      // units consumed per hour of work
}

#[derive(Clone, Copy, Debug)]
pub enum FuelType {
    Wood,
    Charcoal,
    Coal,
    Oil,
}

/// Production facility with physical constraints
#[derive(Debug)]
pub struct ProductionFacility {
    pub id: FacilityId,
    pub facility_type: FacilityType,
    pub position: Vec3,

    // Physical properties
    pub work_stations: u8,         // how many can work simultaneously
    pub storage_volume: f32,       // internal storage space

    // State
    pub current_jobs: Vec<ProductionJob>,
    pub input_storage: StorageContainer,
    pub output_storage: StorageContainer,
    pub fuel_storage: FuelStorage,

    // Efficiency
    pub maintenance_state: f32,    // 0.0 = broken, 1.0 = perfect
    pub tool_quality: f32,         // average quality of installed tools
}

impl ProductionFacility {
    /// Start a production job
    pub fn start_production(
        &mut self,
        recipe: &Recipe,
        worker: EntityId,
        worker_skill: u8,
    ) -> ProductionResult {
        // Check facility type matches
        if let Some(required) = recipe.facility_required {
            if self.facility_type != required {
                return ProductionResult::WrongFacility;
            }
        }

        // Check inputs available
        for input in &recipe.inputs {
            if !self.has_input(input.resource, input.quantity, input.min_quality) {
                return ProductionResult::MissingInput(input.resource);
            }
        }

        // Check tools available
        for tool in &recipe.tools_required {
            if !self.has_tool(tool) {
                return ProductionResult::MissingTool(tool.tool_type);
            }
        }

        // Check fuel
        if let Some(fuel_req) = &recipe.fuel_consumed {
            let total_fuel = fuel_req.amount_per_hour * recipe.work_time_hours;
            if !self.has_fuel(fuel_req.fuel_type, total_fuel) {
                return ProductionResult::InsufficientFuel;
            }
        }

        // Check work station available
        if self.current_jobs.len() >= self.work_stations as usize {
            return ProductionResult::NoWorkStation;
        }

        // Calculate actual work time based on skill and conditions
        let skill_modifier = skill_time_modifier(worker_skill, recipe.min_skill_level);
        let maintenance_modifier = 1.0 + (1.0 - self.maintenance_state) * 0.5;
        let actual_time = recipe.work_time_hours * skill_modifier * maintenance_modifier;

        // Reserve inputs
        for input in &recipe.inputs {
            if input.consumed {
                self.reserve_input(input.resource, input.quantity);
            }
        }

        // Start job
        let job = ProductionJob {
            id: ProductionJobId::new(),
            recipe: recipe.id,
            worker,
            progress: 0.0,
            total_time_hours: actual_time,
            input_quality: self.average_input_quality(&recipe.inputs),
            worker_skill,
        };

        self.current_jobs.push(job);
        ProductionResult::Started
    }

    /// Progress production jobs
    pub fn tick(&mut self, dt_hours: f32, resources: &ResourceRegistry) {
        let mut completed = Vec::new();

        for job in &mut self.current_jobs {
            job.progress += dt_hours;

            // Consume fuel proportionally
            if let Some(fuel_req) = resources.get_recipe(job.recipe).fuel_consumed {
                self.consume_fuel(fuel_req.fuel_type, fuel_req.amount_per_hour * dt_hours);
            }

            if job.progress >= job.total_time_hours {
                completed.push(job.id);
            }
        }

        // Complete finished jobs
        for job_id in completed {
            self.complete_job(job_id, resources);
        }

        // Apply maintenance decay
        self.maintenance_state -= 0.001 * dt_hours; // slow decay
        self.maintenance_state = self.maintenance_state.max(0.0);
    }

    fn complete_job(&mut self, job_id: ProductionJobId, resources: &ResourceRegistry) {
        let job = self.current_jobs.iter()
            .find(|j| j.id == job_id)
            .cloned();

        if let Some(job) = job {
            let recipe = resources.get_recipe(job.recipe);

            // Consume reserved inputs
            for input in &recipe.inputs {
                if input.consumed {
                    self.consume_input(input.resource, input.quantity);
                }
            }

            // Calculate output quality
            let quality = calculate_output_quality(
                &recipe.outputs[0],
                job.input_quality,
                job.worker_skill,
            );

            // Produce outputs
            for output in &recipe.outputs {
                self.output_storage.add(ResourceStack {
                    resource: output.resource,
                    quantity: output.quantity,
                    quality,
                    age_days: 0.0,
                });
            }

            // Produce waste
            for waste in &recipe.waste {
                self.output_storage.add(ResourceStack {
                    resource: waste.resource,
                    quantity: waste.quantity,
                    quality: 0.5, // waste quality is always mediocre
                    age_days: 0.0,
                });
            }

            self.current_jobs.retain(|j| j.id != job_id);
        }
    }

    // Helper methods...
    fn has_input(&self, resource: ResourceId, quantity: u32, min_quality: f32) -> bool {
        // Implementation...
        true
    }

    fn has_tool(&self, requirement: &ToolRequirement) -> bool {
        // Implementation...
        true
    }

    fn has_fuel(&self, fuel_type: FuelType, amount: f32) -> bool {
        // Implementation...
        true
    }

    fn reserve_input(&mut self, resource: ResourceId, quantity: u32) {
        // Implementation...
    }

    fn consume_input(&mut self, resource: ResourceId, quantity: u32) {
        // Implementation...
    }

    fn consume_fuel(&mut self, fuel_type: FuelType, amount: f32) {
        // Implementation...
    }

    fn average_input_quality(&self, inputs: &[RecipeInput]) -> f32 {
        // Implementation...
        0.8
    }
}

fn skill_time_modifier(skill: u8, min_skill: u8) -> f32 {
    // Higher skill = faster work
    // At minimum skill, 50% slower
    // At max skill (10), 30% faster
    let skill_above_min = skill.saturating_sub(min_skill) as f32;
    let max_above_min = 10u8.saturating_sub(min_skill) as f32;
    let ratio = skill_above_min / max_above_min.max(1.0);

    1.5 - (ratio * 0.8) // 1.5 at min skill, 0.7 at max skill
}

fn calculate_output_quality(
    output: &RecipeOutput,
    input_quality: f32,
    worker_skill: u8,
) -> f32 {
    let mut quality = 0.5; // base quality

    if output.quality_from_inputs {
        quality = (quality + input_quality) / 2.0;
    }

    if output.quality_from_skill {
        let skill_quality = worker_skill as f32 / 10.0;
        quality = (quality + skill_quality) / 2.0;
    }

    quality.clamp(0.0, 1.0)
}

#[derive(Clone, Debug)]
pub struct ProductionJob {
    pub id: ProductionJobId,
    pub recipe: RecipeId,
    pub worker: EntityId,
    pub progress: f32,
    pub total_time_hours: f32,
    pub input_quality: f32,
    pub worker_skill: u8,
}

pub enum ProductionResult {
    Started,
    WrongFacility,
    MissingInput(ResourceId),
    MissingTool(ToolType),
    InsufficientFuel,
    NoWorkStation,
}
```

---

## Trade System

### Physical Trade

```rust
/// Trade emerges from physical cost differences, not abstract supply/demand
#[derive(Debug)]
pub struct TradeRoute {
    pub id: TradeRouteId,
    pub origin: SettlementId,
    pub destination: SettlementId,
    pub path: Path,

    pub frequency: TradePeriod,
    pub carrier_type: CarrierType,
    pub capacity: CarryingCapacity,
}

/// Calculate if trade is profitable based on physical costs
pub fn evaluate_trade_opportunity(
    resource: &ResourceProperties,
    quantity: u32,
    route: &TradeRoute,
    origin_price: f32,      // price at origin
    dest_price: f32,        // price at destination
    carrier_wage: f32,      // wage per hour
) -> TradeEvaluation {
    // Calculate transport costs
    let transport = calculate_transport(
        resource,
        quantity,
        &route.capacity,
        &route.path,
        carrier_wage,
    );

    // Calculate value loss from decay
    let decay_loss = transport.decay_during_transport * dest_price * quantity as f32;

    // Total costs
    let purchase_cost = origin_price * quantity as f32;
    let total_cost = purchase_cost + transport.labor_cost + decay_loss;

    // Revenue
    let revenue = dest_price * quantity as f32 * (1.0 - transport.decay_during_transport);

    // Profit
    let profit = revenue - total_cost;
    let margin = profit / total_cost;

    TradeEvaluation {
        resource: resource.id,
        quantity,
        purchase_cost,
        transport_cost: transport.labor_cost,
        decay_cost: decay_loss,
        total_cost,
        revenue,
        profit,
        margin,
        transport_time: transport.total_time_hours,
        trips_required: transport.trips,
        is_profitable: profit > 0.0,
    }
}

#[derive(Debug)]
pub struct TradeEvaluation {
    pub resource: ResourceId,
    pub quantity: u32,
    pub purchase_cost: f32,
    pub transport_cost: f32,
    pub decay_cost: f32,
    pub total_cost: f32,
    pub revenue: f32,
    pub profit: f32,
    pub margin: f32,           // profit / cost
    pub transport_time: f32,   // hours
    pub trips_required: u32,
    pub is_profitable: bool,
}

/// Market prices emerge from physical production costs
pub fn calculate_base_price(
    resource: &ResourceProperties,
    production_recipe: Option<&Recipe>,
    labor_wage: f32,           // local wage per hour
    input_prices: &HashMap<ResourceId, f32>,
) -> f32 {
    match production_recipe {
        Some(recipe) => {
            // Price = input costs + labor costs + margin
            let mut input_cost = 0.0;
            for input in &recipe.inputs {
                if input.consumed {
                    if let Some(price) = input_prices.get(&input.resource) {
                        input_cost += price * input.quantity as f32;
                    }
                }
            }

            let labor_cost = recipe.work_time_hours * labor_wage;

            // Margin based on skill requirement (scarce skills = higher margin)
            let skill_margin = 1.0 + (recipe.min_skill_level as f32 * 0.1);

            (input_cost + labor_cost) * skill_margin
        }
        None => {
            // Raw resource - price based on rarity and extraction difficulty
            // This is set by world generation
            100.0 // placeholder
        }
    }
}
```

---

## Settlement Economy

### Resource Flow

```rust
/// Settlement economy tracks physical resource flows
#[derive(Debug)]
pub struct SettlementEconomy {
    pub settlement_id: SettlementId,

    // Storage
    pub warehouses: Vec<StorageContainer>,
    pub total_storage_volume: f32,
    pub total_storage_mass: f32,

    // Production
    pub facilities: Vec<ProductionFacility>,
    pub production_capacity: HashMap<RecipeId, f32>,

    // Labor
    pub population: u32,
    pub workers: HashMap<SkillId, u32>,
    pub unemployment: u32,

    // Trade
    pub trade_routes: Vec<TradeRoute>,
    pub pending_imports: Vec<TransportJob>,
    pub pending_exports: Vec<TransportJob>,

    // Consumption
    pub daily_consumption: HashMap<ResourceId, f32>,
    pub stockpile_days: HashMap<ResourceId, f32>,
}

impl SettlementEconomy {
    /// Calculate days of supply remaining for each resource
    pub fn calculate_stockpile_days(&mut self) {
        self.stockpile_days.clear();

        for (resource, daily) in &self.daily_consumption {
            let stockpile = self.total_resource(*resource);
            let days = stockpile / daily;
            self.stockpile_days.insert(*resource, days);
        }
    }

    /// Total quantity of a resource across all storage
    pub fn total_resource(&self, resource: ResourceId) -> f32 {
        self.warehouses.iter()
            .flat_map(|w| w.contents.iter())
            .filter(|stack| stack.resource == resource)
            .map(|stack| stack.quantity as f32)
            .sum()
    }

    /// Calculate storage utilization
    pub fn storage_utilization(&self) -> StorageUtilization {
        let used_volume: f32 = self.warehouses.iter()
            .map(|w| w.current_volume)
            .sum();
        let total_volume: f32 = self.warehouses.iter()
            .map(|w| w.max_volume)
            .sum();

        let used_mass: f32 = self.warehouses.iter()
            .map(|w| w.current_mass)
            .sum();
        let total_mass: f32 = self.warehouses.iter()
            .map(|w| w.max_mass)
            .sum();

        StorageUtilization {
            volume_used: used_volume,
            volume_total: total_volume,
            volume_percent: used_volume / total_volume,
            mass_used: used_mass,
            mass_total: total_mass,
            mass_percent: used_mass / total_mass,
        }
    }

    /// Process daily economy tick
    pub fn daily_tick(&mut self, resources: &ResourceRegistry) {
        // Process decay in all storage
        for warehouse in &mut self.warehouses {
            warehouse.process_decay(1.0, resources);
        }

        // Process consumption
        for (resource, daily) in &self.daily_consumption {
            self.consume_resource(*resource, *daily);
        }

        // Update stockpile calculations
        self.calculate_stockpile_days();

        // Generate production jobs based on needs
        self.plan_production(resources);

        // Generate trade jobs based on shortages
        self.plan_trade(resources);
    }

    fn consume_resource(&mut self, resource: ResourceId, amount: f32) {
        let mut remaining = amount;

        // Consume from warehouses in order (oldest first to reduce waste)
        for warehouse in &mut self.warehouses {
            for stack in &mut warehouse.contents {
                if stack.resource == resource && remaining > 0.0 {
                    let consume = (stack.quantity as f32).min(remaining);
                    stack.quantity -= consume as u32;
                    remaining -= consume;
                }
            }
            warehouse.contents.retain(|s| s.quantity > 0);
        }

        // If we couldn't consume enough, settlement has shortage
        if remaining > 0.0 {
            // Handle shortage (reduced morale, starvation, etc.)
            // Detailed in Social Pressure spec
        }
    }

    fn plan_production(&mut self, resources: &ResourceRegistry) {
        // Implementation...
    }

    fn plan_trade(&mut self, resources: &ResourceRegistry) {
        // Implementation...
    }
}

#[derive(Debug)]
pub struct StorageUtilization {
    pub volume_used: f32,
    pub volume_total: f32,
    pub volume_percent: f32,
    pub mass_used: f32,
    pub mass_total: f32,
    pub mass_percent: f32,
}
```

---

## Emergent Economic Behavior

### What Emerges from Physical Constraints

| Physical Constraint | Emergent Behavior |
|--------------------|-------------------|
| Transport has mass limits | Bulk goods traded locally, luxuries traded far |
| Perishables decay | Food production near consumers, or preservation tech |
| Storage has volume | Warehouses become strategic assets |
| Production needs fuel | Settlements cluster near fuel sources |
| Skills are scarce | Specialized production centers |
| Quality degrades | Quality becomes tradeable attribute |

### Example: Iron Trade Route

```rust
// Scenario: Iron ore mine 100km from city
let ore = ResourceProperties::iron_ore();     // 50 kg per unit
let ingot = ResourceProperties::iron_ingot(); // 10 kg per unit

// Transport ore with ox cart
let cart = CarryingCapacity::ox_cart();       // 1000 kg capacity
let ore_per_trip = (cart.max_mass / ore.mass_per_unit) as u32; // 20 units

// At 0.8 m/s over 100km = ~35 hours one way, 70 hours round trip
// Say 10 trips to move 200 units = 700 hours of transport

// OR: Smelt at mine, transport ingots
// 200 ore → ~40 ingots (5:1 ratio)
// 40 ingots × 10 kg = 400 kg
// Single trip! 70 hours total

// Emergent result: Smelters locate near mines, not near markets
```

### Example: Food Distribution

```rust
// Raw meat decays 30% per day
// Preserved meat decays 1% per day

// City 3 days from farms:
// Raw meat: 0.7³ = 34% remaining (66% waste!)
// Preserved: 0.97³ = 91% remaining (9% waste)

// Emergent result:
// - Food preservation industry emerges
// - Fresh food commands premium near source
// - Remote cities invest in cold storage or preservation
```

---

## Quality System

### Physical Quality

```rust
/// Quality affects resource properties
#[derive(Clone, Copy, Debug)]
pub struct QualityModifiers {
    pub effectiveness: f32,    // how well it performs its function
    pub durability: f32,       // how long it lasts
    pub aesthetics: f32,       // how good it looks (luxury value)
}

impl QualityModifiers {
    pub fn from_quality(quality: f32) -> Self {
        Self {
            effectiveness: 0.5 + quality * 0.5,   // 50-100%
            durability: 0.3 + quality * 0.7,      // 30-100%
            aesthetics: quality,                   // 0-100%
        }
    }
}

/// Apply quality to weapon physics
pub fn quality_adjusted_weapon(base: &WeaponPhysics, quality: f32) -> WeaponPhysics {
    let mods = QualityModifiers::from_quality(quality);

    WeaponPhysics {
        // Mass unchanged - quality doesn't change weight
        mass: base.mass,
        length: base.length,
        balance_point: base.balance_point,

        // Edge geometry improved by quality (better forging)
        edge_geometry: base.edge_geometry.map(|e| EdgeGeometry {
            edge_angle: e.edge_angle * (0.8 + mods.effectiveness * 0.2),
            edge_sharpness: e.edge_sharpness * mods.effectiveness,
        }),

        // Material effectively "upgraded" by better working
        material: MaterialId::QualityAdjusted {
            base: base.material,
            hardness_bonus: mods.durability * 0.2,
            toughness_bonus: mods.durability * 0.1,
        },

        category: base.category,
        grip: base.grip,
    }
}
```

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [01-GAME-DESIGN-DOCUMENT](01-GAME-DESIGN-DOCUMENT.md) | Resource flow as core system |
| [06-BALANCE-COMPOSITION-SPEC](06-BALANCE-COMPOSITION-SPEC.md) | Property composition philosophy |
| [13-MODULE-SCHEMA-SPEC](13-MODULE-SCHEMA-SPEC.md) | Resource module definitions |
| [15-WORLD-GENERATION-SPEC](15-WORLD-GENERATION-SPEC.md) | Resource distribution |
| [18-SOCIAL-PRESSURE-MORALE-SPEC](18-SOCIAL-PRESSURE-MORALE-SPEC.md) | Shortage effects on morale |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
