use super::*;
use crate::{
    factions::FactionId,
    types::items::{
        definitions::{
            ITEM_ICE_ORE,
            ITEM_IRON_INGOT,
            ITEM_IRON_ORE,
            ITEM_SILICON_ORE,
            ITEM_SILICON_RAW,
            ITEM_WATER,
        },
        ItemDefinitionId,
    },
};

#[dsl(plural_name = trading_port_modules)]
#[table(name = trading_port_module, public)]
pub struct TradingPort {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,
    // Configuration for item capacity is better in StationModuleBlueprint (max_internal_storage_slots/volume)

    // This table will mainly link to its active trade listings.
}

/// Represents items the Trading Port module is actively buying or selling.
/// This is distinct from general market orders placed by players at a station.
#[dsl(plural_name = trading_port_listings)]
#[table(name = trading_port_listing, public)]
pub struct TradingPortListing {
    #[primary_key]
    // #[auto_inc]
    // pub listing_id: u64,

    // #[index(btree)]
    // #[wrapped(path = StationModuleId)]
    // /// FK to StationModuleInstance (must be a TradingPort)
    // pub trading_port_module_id: u64,

    // #[index(btree)]
    #[wrapped(path = StationModuleInventoryItemId)]
    /// FK to StationModuleInventoryItem
    pub station_inventory_item_id: u64,

    // #[index(btree)]
    // #[wrapped(path = crate::types::items::ItemDefinitionId)]
    // /// FK to ItemDefinition
    // pub item_id: u32,

    /// None if the port is not buying, Some percentage of how much margin the port want below base price.
    pub buying_margin: Option<f32>,
    /// None if the port is not selling, Some percentage of how much margin the port want above base price.
    pub selling_margin: Option<f32>,

    // /// Optional current price, calculated by a reducer based on stock, demand, faction standing etc.
    // pub current_calculated_price_per_unit: Option<u64>,
    // pub last_price_update_timestamp: Timestamp,
}

pub fn create_basic_bazaar(
    ctx: &ReducerContext,
    station: &Station,
    under_construction: bool
) -> Result<(), String> {
    let dsl = dsl(ctx);
    //
    if under_construction {
        return Err("Not yet implemented".to_string());
    }

    let blueprint = dsl
        .get_station_module_blueprint_by_id(
            StationModuleBlueprintId::new(definitions::MODULE_TRADING_BAZAAR)
        )
        .ok_or("Couldn't find blueprint. Did a module blueprint ID change?")?;

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        "bazaar", // TODO: Do we even need this field?
        true,
        None,
        ctx.timestamp
    )?;

    // Create trading port listings for ITEM_ICE_ORE, ITEM_IRON_ORE, ITEM_SILICON_ORE
    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_ICE_ORE),
        10,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};trading", module.id, ITEM_ICE_ORE).as_str()
    )?;
    dsl.create_trading_port_listing(item.get_id(), Some(0.8), None)?;

    item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_IRON_ORE),
        20,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};trading", module.id, ITEM_IRON_ORE).as_str()
    )?;
    dsl.create_trading_port_listing(item.get_id(), Some(0.8), None)?;

    item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_SILICON_ORE),
        40,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};trading", module.id, ITEM_SILICON_ORE).as_str()
    )?;
    dsl.create_trading_port_listing(item.get_id(), Some(0.8), None)?;

    Ok(())
}

///////////////////////////////////////////////////////////////////////

#[dsl(plural_name = storage_depot_modules)]
#[table(name = storage_depot_module, public)]
pub struct StorageDepot {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,
    // Configuration for item capacity is better in StationModuleBlueprint (max_internal_storage_slots/volume)

    // This table denotes that this station is a storage depot
}

//////////////////////////////////////////////////////////////////////

#[dsl(plural_name = embassy_presences)]
#[table(name = embassy_presence, public)]
pub struct EmbassyPresence {
    #[primary_key] // Composite key: "embassy_module_id:representing_faction_id" - Must be MANUALLY enforced
    #[wrap]
    pub presence_id_composite: String,
    // Alternatively:
    // #[primary_key_part1]
    /// FK to StationModuleInstance (must be an Embassy)
    // pub embassy_module_instance_id: u64,
    // #[primary_key_part2]
    /// FK to Faction (the faction that has established presence here)
    // pub representing_faction_id: u32,

    #[index(btree)]
    #[wrapped(path = StationModuleId)]
    pub embassy_module_id: u64,

    #[index(btree)]
    #[wrapped(path = FactionId)]
    pub representing_faction_id: u32,

    pub established_at_timestamp: Timestamp,
    pub diplomatic_status_notes: Option<String>, // e.g., "Ambassadorial level", "Trade mission"
}

#[dsl(plural_name = embassy_modules)]
#[table(name = embassy_module, public)]
pub struct Embassy {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,
    // Configuration for item capacity is better in StationModuleBlueprint (max_internal_storage_slots/volume)

    // This table denotes that this station is a storage depot
}

//////////////////////////////////////////////////////////////////////

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum FarmOutputQuality {
    Lower,
    Average,
    Upper,
    Luxury,
}

#[dsl(plural_name = farm_modules)]
#[table(name = farm_module, public)]
pub struct Farm {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,

    /// Defined by the StationModuleBlueprint.specific_type (e.g., LuxuryFoodFarm produces Luxury Food)
    /// FK to ResourceDefinition (e.g., "Luxury Food", "Standard Wood")
    pub output_resource_id: u32,

    pub output_quality: FarmOutputQuality,

    /// Resource ID for the primary input (e.g., "Raw Biomatter Type A")
    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub primary_input_resource_id: u32,
    pub primary_input_conversion_rate: f32, // units of primary input per unit of output

    /// Resource ID for a secondary input (e.g., "Purified Water")
    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub secondary_input_resource_id: Option<u32>,
    pub secondary_input_conversion_rate: Option<f32>,

    pub base_production_units_per_hour: f32,
    pub current_efficiency_modifier: f32, // Based on sector, upgrades, staffing. Default 1.0
}

//////////////////////////////////////////////////////////////////////

#[dsl(plural_name = observatory_modules)]
#[table(name = observatory_module, public)]
pub struct Observatory {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,

    pub base_research_points_per_hour: u32,
    /// Efficiency based on sector type (nebula, anomaly) and module upgrades.
    pub current_efficiency_modifier: f32, // Default 1.0

    /// Input resource ID (e.g., "Advanced Sensor Crystal")
    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub primary_input_resource_id: u32,
    pub primary_input_consumption_rate: Option<f32>, // units per hour of operation

    /// Output resource ID (e.g., "Raw Astronomical Data")
    pub output_data_fragment_resource_id: u32,
}

//////////////////////////////////////////////////////////////////////

#[dsl(plural_name = refinery_modules)]
#[table(name = refinery_module, public)]
pub struct Refinery {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,

    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub input_ore_resource_id: u32, // FK to ResourceDefinition

    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub output_ingot_resource_id: u32, // FK to ResourceDefinition

    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub waste_resource_id: Option<u32>, // FK to ResourceDefinition

    /// How many units of ore to make 1 unit of ingot.
    pub ore_to_ingot_ratio: f32,
    /// How many units of waste are produced to make 1 unit of ingot.
    pub waste_per_ingot_ratio: f32,

    pub base_ingots_produced_per_hour: f32,
    pub current_efficiency_modifier: f32, // Default 1.0
}

pub fn create_basic_refinery(
    ctx: &ReducerContext,
    station: &Station,
    under_construction: bool
) -> Result<(), String> {
    let dsl = dsl(ctx);
    //
    if under_construction {
        return Err("Not yet implemented".to_string());
    }

    let blueprint = dsl
        .get_station_module_blueprint_by_id(
            StationModuleBlueprintId::new(definitions::MODULE_REFINERY_MINOR)
        )
        .ok_or("Couldn't find blueprint. Did a module blueprint ID change?")?;

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        "refinery", // TODO: Do we even need this field?
        true,
        None,
        ctx.timestamp
    )?;

    // /// Ice Submodule
    // let ice_ref = dsl.create_refinery_module(
    //     module.get_id(),
    //     ItemDefinitionId::new(ITEM_ICE_ORE),
    //     ItemDefinitionId::new(ITEM_WATER),
    //     None,
    //     10.0,
    //     0.0,
    //     30.0,
    //     1.0
    // )?;

    // dsl.create_station_module_inventory_item(
    //     module.get_id(),
    //     ItemDefinitionId::new(ITEM_ICE_ORE),
    //     0,
    //     blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
    //     format!("{};{};input", module.id, ice_ref.id).as_str()
    // )?;

    // dsl.create_station_module_inventory_item(
    //     module.get_id(),
    //     ItemDefinitionId::new(ITEM_WATER),
    //     0,
    //     blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
    //     format!("{};{};output", module.id, ice_ref.id).as_str()
    // )?;

    /// Iron Submodule
    let iron_ref = dsl.create_refinery_module(
        module.get_id(),
        ItemDefinitionId::new(ITEM_IRON_ORE),
        ItemDefinitionId::new(ITEM_IRON_INGOT),
        None,
        10.0,
        0.0,
        30.0,
        1.0
    )?;

    dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_IRON_ORE),
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};input", module.id, iron_ref.id).as_str()
    )?;

    dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_IRON_INGOT),
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};output", module.id, iron_ref.id).as_str()
    )?;

    // /// Silicon Submodule
    // let silicon_ref = dsl.create_refinery_module(
    //     module.get_id(),
    //     ItemDefinitionId::new(ITEM_SILICON_ORE),
    //     ItemDefinitionId::new(ITEM_SILICON_RAW),
    //     None,
    //     10.0,
    //     0.0,
    //     30.0,
    //     1.0
    // )?;

    // dsl.create_station_module_inventory_item(
    //     module.get_id(),
    //     ItemDefinitionId::new(ITEM_SILICON_ORE),
    //     0,
    //     blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
    //     format!("{};{};input", module.id, silicon_ref.id).as_str()
    // )?;

    // dsl.create_station_module_inventory_item(
    //     module.get_id(),
    //     ItemDefinitionId::new(ITEM_SILICON_RAW),
    //     0,
    //     blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
    //     format!("{};{};output", module.id, silicon_ref.id).as_str()
    // )?;

    Ok(())
}

//////////////////////////////////////////////////////////////////////

#[dsl(plural_name = solar_array_modules)]
#[table(name = solar_array_module, public)]
pub struct SolarArray {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,

    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub output_energy_cell_resource_id: u32, // FK to ResourceDefinition

    pub base_energy_cells_produced_per_hour: u32,
    /// Efficiency based on sector's sunlight_percentage and module health/upgrades.
    pub current_efficiency_modifier: f32, // Default 1.0
}

//////////////////////////////////////////////////////////////////////

#[dsl(plural_name = synthesizer_modules)]
#[table(name = synthesizer_module, public)]
pub struct Synthesizer {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,

    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub input_exotic_matter_resource_id: u32,

    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub input_gas_resource_id: u32,

    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub output_jump_fuel_resource_id: u32,

    pub exotic_matter_per_fuel_unit: f32,
    pub gas_per_fuel_unit: f32,
    pub base_fuel_units_produced_per_hour: f32,
}

//////////////////////////////////////////////////////////////////////

/// Defines a recipe that a manufacturing module can use.
#[dsl(plural_name = production_recipe_definitions)]
#[table(name = production_recipe_definition, public)]
pub struct ProductionRecipeDefinition {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u32,
    #[unique]
    pub name: String, // e.g., "Basic Hull Plating", "Mk1 Laser Cannon Assembly"

    pub input_resources: Vec<ResourceAmount>,

    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub output_resource_id: u32, // FK to ResourceDefinition

    pub output_quantity: u32,
    pub base_production_time_seconds: u32,
    /// Which type of module can use this recipe (e.g., Factory, Assembler)
    pub required_module_specific_type: StationModuleSpecificType,
    pub required_tech_id_to_unlock: Option<u32>, // FK to TechnologyTreeNode
}

/// Data for a generic manufacturing module instance (Factory, Assembler, Fabricator).
#[dsl(plural_name = manufacturing_modules)]
#[table(name = manufacturing_module, public)]
pub struct Manufacturing {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,

    #[wrapped(path = ProductionRecipeDefinitionId)]
    /// The recipe this specific module instance is currently configured to produce. FK to ProductionRecipeDefinition
    pub current_recipe_id: Option<u32>,

    pub is_producing: bool,
    pub production_queue_count: u32, // Number of items queued for production
    pub current_production_progress_seconds: f32,
    /// Modifier based on upgrades, staffing, etc. Affects production_time_seconds.
    pub production_speed_modifier: f32, // Default 1.0
}

//////////////////////////////////////////////////////////////////////

#[dsl(plural_name = laboratory_modules)]
#[table(name = laboratory_module, public)]
pub struct Laboratory {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,

    pub base_research_points_per_hour: u32,

    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// Input resource ID (e.g., "Analyzed Data Cache") FK to ItemDefinition
    pub primary_input_resource_id: u32,

    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    /// Input resource ID (e.g., "Rare Crystal Sample") FK to ItemDefinition
    pub secondary_input_resource_id: Option<u32>,

    pub primary_input_consumption_rate: f32, // units per hour
    pub secondary_input_consumption_rate: Option<f32>, // units per hour
    pub current_efficiency_modifier: f32, // Based on upgrades, staffing
}

//////////////////////////////////////////////////////////////////////
///
#[dsl(plural_name = capital_dock_modules)]
#[table(name = capital_dock_module, public)]
pub struct CapitalDock {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,

    pub max_capital_ship_capacity: u8, // e.g., 10
}

/// Tracks capital ships currently docked at a specific Capital Dock module.
#[dsl(plural_name = docked_capital_ship_at_modules)]
#[table(name = docked_capital_ship_at_module, public)]
pub struct DockedCapitalShipAt {
    #[primary_key]
    #[wrapped(path = ships::ShipGlobalId)]
    pub ship_id: u64, // FK to Ship (must be a capital ship)

    #[index(btree)]
    #[wrapped(path = StationModuleId)]
    pub capital_dock_module_id: u64, // FK to StationModuleInstance (a CapitalDock)

    pub docked_at_timestamp: Timestamp,
}

//////////////////////////////////////////////////////////////////////

#[dsl(plural_name = anti_capital_turret_modules)]
#[table(name = anti_capital_turret_module, public)]
pub struct AntiCapitalTurret {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,

    /// FK to a ShipModuleBlueprint that defines the weapon's stats (damage, range, fire rate)
    #[wrapped(path = crate::types::items::ItemDefinitionId)]
    pub weapon_core_blueprint_id: u32,

    #[wrapped(path = ships::ShipGlobalId)]
    pub current_target_ship_id: Option<u64>, // FK to ShipInstance

    pub can_launch_fighters: bool,
    pub fighter_capacity: Option<u8>,
    // Fighters stored here would be ShipInstances linked to this module, perhaps in a `DockedShipAtModule` table.
    // Ammo and fuel are in StationModuleInventoryItem.
}

//////////////////////////////////////////////////////////////////////

#[dsl(plural_name = residential_modules)]
#[table(name = residential_module, public)]
pub struct Residential {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,

    /// Base max occupancy from blueprint, actual can be affected by upgrades/morale.
    pub base_max_occupancy: u32,
    pub current_occupancy: u32, // Generic population/crew
    /// Morale specifically within this residential module. Affects overall station morale.
    pub current_internal_morale_percentage: f32, // 0.0 to 100.0
    pub crew_replenishment_pool: u32, // Available crew for players to hire
    pub crew_generation_rate_per_hour: f32,

    // --- Fields specific to Spacious Residential ---
    /// For Spacious/Luxury, the level of park/amenity upgrades.
    pub amenity_upgrade_level: Option<u8>,
    /// Additional morale boost from amenities.
    pub amenity_morale_boost: Option<i16>,

    // --- Fields specific to Luxury Residential ---
    pub max_luxury_npc_slots: Option<u8>,
    pub current_luxury_npc_count: Option<u8>,

    /// FKs to ResourceDefinition (luxury food, drinks etc.)
    pub luxury_upkeep_requirements: Option<Vec<ResourceAmount>>,
}

// /// Notable NPCs that prefer to live in Luxury Residential modules.
// #[table(name = notable_npc_preference, public)]
// #[derive(Clone, Debug)]
// pub struct NotableNpcPreference {
//     #[primary_key]
//     pub npc_definition_id: u32, // FK to NpcDefinition (for NPCs that are "notable")
//     /// The luxury item this NPC particularly desires.
//     pub preferred_luxury_item_resource_id: u32, // FK to ResourceDefinition
//     /// Multiplier for how much above base value they might pay.
//     pub preferred_item_buy_price_modifier: f32, // e.g., 1.2 for 20% over market
//     /// What kind of quests or interactions this NPC might offer.
//     pub interaction_profile_tag: String, // e.g., "RareMaterialsTrader", "InformationBroker"
// }

// /// Tracks which notable NPCs are currently residing in a specific luxury module.
// #[table(name = luxury_npc_resident, public)]
// #[derive(Clone, Debug)]
// pub struct LuxuryNpcResident {
//     #[primary_key]
//     pub npc_instance_id: u64, // FK to NpcInstance
//     #[index(btree)]
//     pub luxury_module_instance_id: u64, // FK to a StationModuleInstance (LuxuryResidential)
//     pub residency_began_timestamp: Timestamp,
//     pub current_satisfaction_rating: f32, // 0.0 to 1.0
// }

//////////////////////////////////////////////////////////////////////

#[dsl(plural_name = hospital_modules)]
#[table(name = hospital_module, public)]
pub struct Hospital {
    #[primary_key]
    #[wrapped(path = StationModuleId)]
    /// FK to StationModule
    pub id: u64,

    pub medical_bay_capacity: u16, // Max players/NPCs that can be treated simultaneously
    pub healing_effectiveness_modifier: f32, // Base 1.0, affected by upgrades/staff
    /// If morale boost is sector-wide and distinct from station morale.
    pub sector_morale_boost_value: Option<i16>,
}

//////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////
