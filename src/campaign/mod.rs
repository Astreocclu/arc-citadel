pub mod battle;
pub mod location;
pub mod map;
pub mod route;
pub mod scouts;
pub mod supply;
pub mod visibility;
pub mod weather;

pub use location::Location;
pub use map::{CampaignMap, CampaignTerrain, HexCoord, HexTile};
pub use route::{
    Army, ArmyId, ArmyOrder, ArmyStance, CampaignEvent, CampaignState, MovementResult, campaign_tick,
};
pub use supply::{
    ArmySupply, DepotId, SupplyDepot, SupplyEvent, SupplySystem,
    BASE_SUPPLY_DAYS, FORAGE_BASE_RATE, STARVATION_ATTRITION_RATE,
    calculate_forage_yield,
};
pub use weather::{
    RegionalWeather, Season, Weather, WeatherEvent, WeatherState, WeatherZone,
};
pub use visibility::{
    FactionVisibility, HexIntel, HexVisibility, VisibilityEvent, VisibilitySystem,
    BASE_VISIBILITY_RANGE,
    calculate_visibility_range, get_visible_hexes,
};
pub use battle::{
    BattleEvent, BattleOutcome, BattleResult,
    resolve_battle, resolve_combat_round, apply_retreat, check_rout,
    calculate_combat_strength,
    BASE_CASUALTY_RATE, ROUT_THRESHOLD,
};
pub use scouts::{
    IntelType, Scout, ScoutEvent, ScoutId, ScoutIntel, ScoutMission, ScoutSystem,
    SCOUT_DETECTION_RANGE, SCOUT_EVASION_CHANCE, SCOUT_SPEED_MULTIPLIER,
};
