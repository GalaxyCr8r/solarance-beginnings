use spacetimedb::{ table, ReducerContext, SpacetimeType, Timestamp };
use spacetimedsl::*;

use crate::{ types::economy::ResourceAmount, * };

pub mod definitions; // Definitions for initial ingested data.
pub mod impls; // Impls for this file's structs
pub mod modules; // Station modules
// pub mod reducers; // SpacetimeDB Reducers for this file's structs.
// pub mod rls; // Row-level-security rules for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

//////////////////////////////////
// Enums

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum StationSize {
    Capital, // Singular faction hub
    Large,
    Medium,
    Small,
    Outpost,
    Satellite,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq)]
pub enum StationModuleCategory {
    LogisticsAndStorage,
    ResourceProductionAndRefining,
    ManufacturingAndAssembly,
    ResearchAndDevelopment,
    CivilianAndSupportServices,
    DiplomacyAndFaction,
    DefenseAndMilitary,
}

/// Enum for specific module types, more granular than category.
/// This helps define what a blueprint *is*.
#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq)]
pub enum StationModuleSpecificType {
    // Logistics & Storage
    TradingPort,
    Depot,
    CapitalDock,
    Wharf,
    // Resource Production & Refining
    FarmStandard, // Produces standard quality biomatter/food
    FarmLuxury, // Produces luxury quality biomatter/food
    RefineryBasicOre, // e.g., Iron -> Iron Ingots
    RefineryAdvancedOre, // e.g., Titanite -> Titanium Ingots
    SolarArray,
    SynthesizerJumpFuel,
    // Manufacturing & Assembly
    FactoryBasicComponents,
    FactoryAdvancedComponents,
    AssemblerShipModules,
    AssemblerStationModules,
    FabricatorElectronics,
    // Research & Development
    Laboratory,
    Observatory,
    // Civilian & Support Services
    ResidentialBasic,
    ResidentialSpacious,
    ResidentialLuxury,
    Hospital,
    // Diplomacy & Faction
    Embassy,
    // Defense & Military
    AntiCapitalTurretKinetic,
    AntiCapitalTurretEnergy, // Maybe Torpedo as well?
    FighterBay, // For modules launching fighters/interceptors (could be part of AntiCapitalTurret or Garrison?)
    GarrisonRegionalDefense,
}

/////////////////////////////////////
// Tables

#[dsl(plural_name = station_module_blueprints)]
#[table(name = station_module_blueprint, public)]
pub struct StationModuleBlueprint {
    #[primary_key]
    #[wrap]
    pub id: u32,

    #[unique]
    pub name: String,

    pub description: String,

    pub category: StationModuleCategory,
    pub specific_type: StationModuleSpecificType,

    pub build_cost_resources: Vec<ResourceAmount>,
    pub build_time_seconds: u32,

    pub power_consumption_mw_operational: f32, // Power needed when active
    pub power_consumption_mw_idle: f32, // Power needed when idle
    pub cpu_load_flops: f32,

    pub required_station_tech_level: u8,

    pub max_internal_storage_slots: u16, // Number of distinct item types it can hold
    pub max_internal_storage_volume_per_slot_m3: Option<u32>, // Volume per slot

    pub provides_station_morale_boost: Option<i16>, // Base morale if applicable
    pub icon_asset_id: Option<String>,

    pub construction_hp: u32, // HP during construction phase
    pub operational_hp: u32, // Max HP when fully built
}

#[dsl(plural_name = station_modules)]
#[table(name = station_module, public)]
pub struct StationModule {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)]
    #[wrapped(path = StationId)]
    /// FK to SpaceStation
    pub station_id: u64,

    /// FK to StationModuleBlueprint
    #[index(btree)]
    #[wrapped(path = StationModuleBlueprintId)]
    pub blueprint: u32,

    pub station_slot_identifier: String, // e.g., "HabitatRing-A-Slot3", "Core-Power-Slot1"
    pub is_operational: bool,

    pub built_at_timestamp: Option<Timestamp>,
    pub last_status_update_timestamp: Timestamp,
}

#[dsl(plural_name = stations_under_construction)]
#[table(name = station_under_construction, public)]
pub struct StationUnderConstruction {
    #[primary_key]
    #[wrapped(path = StationId)]
    /// FK to SpaceStation
    pub station_id: u64,

    pub is_operational: bool,
    pub construction_progress_percentage: f32,
}

#[dsl(plural_name = station_modules_under_construction)]
#[table(name = station_module_under_construction, public)]
pub struct StationModuleUnderConstruction {
    #[primary_key]
    #[wrapped(path = StationId)]
    /// FK to SpaceStation
    pub station_id: u64,

    pub is_operational: bool,
    pub construction_progress_percentage: f32,
}

/// Stores items used for a module's operation or as temporary input/output buffers.
#[dsl(plural_name = station_module_inventory_items)]
#[table(name = station_module_inventory_item, public)]
pub struct StationModuleInventoryItem {
    #[primary_key]
    #[auto_inc]
    pub inventory_item_id: u64,

    #[index(btree)]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub module_id: u64,

    #[index(btree)]
    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub resource_item_id: u32,

    pub quantity: u32,
    pub max_quantity: u32,

    /// Describes the purpose, e.g., "InputBuffer", "OutputBuffer", "OperationalFuel", "Ammunition"
    pub storage_purpose_tag: String,
}

#[dsl(plural_name = stations)]
#[table(name = station, public)]
pub struct Station {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    #[index(btree)]
    pub size: StationSize,

    #[index(btree)]
    #[wrapped(path = sectors::SectorId)]
    /// FK to Sector.id
    pub sector_id: u64,

    #[unique]
    #[wrapped(path = stellarobjects::StellarObjectId)]
    /// FK to StellarObject
    pub sobj_id: u64,

    #[index(btree)]
    #[wrapped(path = factions::FactionId)]
    /// FK to FactionDefinition
    pub owner_faction_id: u32,

    pub name: String,

    // services_offered: Vec<StationServiceType>, // Could be an enum or FKs to service definitions

    pub gfx_key: Option<String>,
}

#[dsl(plural_name = station_statuses)]
#[table(name = station_status, public)]
pub struct StationStatus {
    #[primary_key]
    #[wrapped(path = StationId)]
    /// FK to Station
    pub station_id: u64,

    pub health: f32,
    pub shields: f32,
    pub energy: f32,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    definitions::init(ctx);

    Ok(())
}
