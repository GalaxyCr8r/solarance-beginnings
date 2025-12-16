use spacetimedb::{table, SpacetimeType, Timestamp};
use spacetimedsl::*;

use crate::tables::economy::ResourceAmount;
use crate::tables::items::*;
use crate::*;

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
    StorageDepot,
    CapitalDock,
    Wharf,
    // Resource Production & Refining
    FarmStandard,        // Produces standard quality biomatter/food
    FarmLuxury,          // Produces luxury quality biomatter/food
    RefineryBasicOre,    // e.g., Iron -> Iron Ingots
    RefineryAdvancedOre, // e.g., Titanite -> Titanium Ingots
    RefineryExoticOre,
    SolarArray,
    SynthesizerJumpFuel,
    // Manufacturing & Assembly
    FactoryBasicComponents,
    FactoryAdvancedComponents,
    AssemblerShipModules,
    AssemblerStationModules,
    FabricatorElectronics,
    ComponentAssembler,
    ShipyardFabrication,
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

#[dsl(plural_name = station_module_blueprints, method(update = true))]
#[table(name = station_module_blueprint, public)]
pub struct StationModuleBlueprint {
    #[primary_key]
    #[create_wrapper]
    #[referenced_by(path = crate::tables::stations, table = station_module)]
    id: u32,

    #[unique]
    pub name: String,

    pub description: String,

    pub category: StationModuleCategory,
    pub specific_type: StationModuleSpecificType,

    pub build_cost_resources: Vec<ResourceAmount>,
    pub build_time_seconds: u32,

    pub power_consumption_mw_operational: f32, // Power needed when active
    pub power_consumption_mw_idle: f32,        // Power needed when idle
    pub cpu_load_flops: f32,

    pub required_station_tech_level: u8,

    pub max_internal_storage_slots: u16, // Number of distinct item types it can hold
    pub max_internal_storage_volume_per_slot_m3: Option<u32>, // Volume per slot

    pub provides_station_morale_boost: Option<i16>, // Base morale if applicable
    pub icon_asset_id: Option<String>,

    pub construction_hp: u32, // HP during construction phase
    pub operational_hp: u32,  // Max HP when fully built
}

#[dsl(plural_name = station_modules, method(update = true))]
#[table(name = station_module, public)]
pub struct StationModule {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::tables::stations, table = station_module_inventory_item)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(StationId)]
    #[foreign_key(path = crate::tables::stations, table = station, column = id, on_delete = Delete)]
    /// FK to SpaceStation
    pub station_id: u64,

    /// FK to StationModuleBlueprint
    #[index(btree)]
    #[use_wrapper(StationModuleBlueprintId)]
    #[foreign_key(path = crate::tables::stations, table = station_module_blueprint, column = id, on_delete = Error)]
    pub blueprint: u32,

    pub station_slot_identifier: String, // e.g., "HabitatRing-A-Slot3", "Core-Power-Slot1"
    pub is_operational: bool,

    pub built_at_timestamp: Option<Timestamp>,
    pub last_status_update_timestamp: Timestamp,
}

#[dsl(plural_name = stations_under_construction, method(update = true))]
#[table(name = station_under_construction, public)]
pub struct StationUnderConstruction {
    #[primary_key]
    #[use_wrapper(StationId)]
    #[foreign_key(path = crate::tables::stations, table = station, column = id, on_delete = Delete)]
    /// FK to SpaceStation
    id: u64,

    pub is_operational: bool,
    pub construction_progress_percentage: f32,
}

#[dsl(plural_name = station_modules_under_construction, method(update = true))]
#[table(name = station_module_under_construction, public)]
pub struct StationModuleUnderConstruction {
    #[primary_key]
    #[use_wrapper(StationId)]
    #[foreign_key(path = crate::tables::stations, table = station, column = id, on_delete = Delete)]
    /// FK to SpaceStation
    id: u64,

    pub is_operational: bool,
    pub construction_progress_percentage: f32,
}

/// Stores items used for a module's operation or as temporary input/output buffers.
#[dsl(plural_name = station_module_inventory_items, method(update = true))]
#[table(name = station_module_inventory_item, public)]
pub struct StationModuleInventoryItem {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(StationModuleId)]
    #[foreign_key(path = crate::tables::stations, table = station_module, column = id, on_delete = Delete)]
    /// FK to StationModule
    pub module_id: u64,

    #[index(btree)]
    #[use_wrapper(crate::tables::items::ItemDefinitionId)]
    #[foreign_key(path = crate::tables::items, table = item_definition, column = id, on_delete = Error)]
    /// FK to ItemDefinition
    pub resource_item_id: u32,

    pub quantity: u32,
    pub max_quantity: u32,

    /// Describes the purpose, e.g., "InputBuffer", "OutputBuffer", "OperationalFuel", "Ammunition"
    pub storage_purpose_tag: String,

    /// Cached current price the station buys and/or sells at.
    pub cached_price: u32,
}

#[dsl(plural_name = stations, method(update = true))]
#[table(name = station, public)]
pub struct Station {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::tables::stations, table = station_module)]
    #[referenced_by(path = crate::tables::stations, table = station_under_construction)]
    #[referenced_by(path = crate::tables::stations, table = station_module_under_construction)]
    #[referenced_by(path = crate::tables::stations, table = station_status)]
    #[referenced_by(path = crate::tables::ships, table = ship)]
    id: u64,

    #[index(btree)]
    pub size: StationSize,

    #[index(btree)]
    #[use_wrapper(sectors::SectorId)]
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Error)]
    /// FK to Sector.id
    pub sector_id: u64,

    #[unique]
    #[use_wrapper(stellarobjects::StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    pub sobj_id: u64,

    #[index(btree)]
    #[use_wrapper(factions::FactionId)]
    #[foreign_key(path = crate::tables::factions, table = faction, column = id, on_delete = Error)]
    /// FK to FactionDefinition
    pub owner_faction_id: u32,

    pub name: String,

    // services_offered: Vec<StationServiceType>, // Could be an enum or FKs to service definitions
    pub gfx_key: Option<String>,
}

#[dsl(plural_name = station_statuses, method(update = true))]
#[table(name = station_status, public)]
pub struct StationStatus {
    #[primary_key]
    #[use_wrapper(StationId)]
    #[foreign_key(path = crate::tables::stations, table = station, column = id, on_delete = Delete)]
    /// FK to Station
    id: u64,

    pub health: f32,
    pub shields: f32,
    pub energy: f32,
}

////////////////////////////////////////////////////
/// Impls
///

impl StationSize {
    /// How many modules can this size support?
    pub fn max_module_amount(&self) -> u8 {
        match self {
            StationSize::Capital => 13,
            StationSize::Large => 9,
            StationSize::Medium => 7,
            StationSize::Small => 5,
            StationSize::Outpost => 3,
            StationSize::Satellite => 1,
        }
    }

    pub fn calculate_base_cost(&self) -> u32 {
        (self.max_module_amount().pow(2) as u32) * 100_000 + 300_000
    }

    /// Retooling a space station to a larger size should be possible, but discouraged.
    pub fn calculate_upgrade_cost(&self, new_size: StationSize) -> u32 {
        new_size.calculate_base_cost() - self.calculate_base_cost()
            + ((new_size.max_module_amount() - self.max_module_amount()) as u32)
    }

    pub fn calculate_base_health(&self) -> u32 {
        (self.max_module_amount().pow(2) as u32) * 25_000 + 100_000
    }

    pub fn calculate_base_shields(&self) -> u32 {
        (self.max_module_amount().pow(2) as u32) * 50_000 + 200_000
    }
}

impl StationModuleInventoryItem {
    /// Calculates the current price of an item based on its quantity and item definition.
    pub fn calculate_current_price(&self, item_def: &ItemDefinition) -> u32 {
        // Convert to floats
        let max_quantity = self.max_quantity as f32;
        let current_quantity = self.quantity as f32;
        let value = *item_def.get_base_value() as f32;

        //info!("Calculating price for {}", item_def.name);
        // Calc the percent and resultant multiplier
        let percent_full = current_quantity / max_quantity;
        let multiplier = percent_full * -2.0 + 1.0; // 1.0 .. -1.0
                                                    // info!("    Curr/Max : {}/{}", current_quantity, max_quantity);
                                                    // info!("    Multipler : {}", multiplier);

        // Find the value of the given margin
        let curr_margin_perc = (*item_def.get_margin_percentage() as f32) * 0.01;
        let margin_value = value * curr_margin_perc;
        // info!("    Curr Margin Perc : {}", curr_margin_perc);
        // info!("    Base Margin Value : {}c", margin_value);
        // info!("    Adjusted Value : {}c", margin_value * multiplier);
        // info!(
        //     "    New Value : {}c",
        //     (value + margin_value * multiplier) as u32
        // );

        // If current_quantity == max_quantity then the current price should be base_value + (base_value * default_margin * -1.0)
        // If current_quantity == 0 then the current price should be base_value + (base_value * default_margin * 1.0)
        (value + margin_value * multiplier) as u32
    }
}
