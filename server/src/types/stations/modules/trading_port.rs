use super::*;
use crate::types::items::{
    definitions::{
        ITEM_ENERGY_CELL,
        ITEM_FOOD_AVERAGE,
        ITEM_FOOD_RATIONS,
        ITEM_ICE_ORE,
        ITEM_IRON_ORE,
        ITEM_SILICON_ORE,
    },
    ItemDefinitionId,
    GetItemDefinitionRowOptionById,
};

#[dsl(plural_name = trading_port_modules)]
#[table(name = trading_port_module, public)]
pub struct TradingPort {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,
    // Configuration for item capacity is better in StationModuleBlueprint (max_internal_storage_slots/volume)

    // This table will mainly link to its active trade listings.
}

/// Represents items the Trading Port module is actively buying or selling.
/// This is distinct from general market orders placed by players at a station.
#[dsl(plural_name = trading_port_listings)]
#[table(name = trading_port_listing, public)]
pub struct TradingPortListing {
    #[primary_key]
    #[use_wrapper(path = StationModuleInventoryItemId)]
    /// FK to StationModuleInventoryItem
    id: u64,

    /// None if the port is not buying, Some percentage of how much margin the port want below base price.
    pub buying_margin: Option<f32>,
    /// None if the port is not selling, Some percentage of how much margin the port want above base price.
    pub selling_margin: Option<f32>,
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

    let blueprint = dsl.get_station_module_blueprint_by_id(
        StationModuleBlueprintId::new(definitions::MODULE_TRADING_BAZAAR)
    )?;

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        "bazaar", // TODO: Do we even need this field?
        true,
        None,
        ctx.timestamp
    )?;

    let _trading = dsl.create_trading_port_module(module.get_id())?;

    // Create trading port listings for ITEM_ICE_ORE, ITEM_IRON_ORE, ITEM_SILICON_ORE
    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_ICE_ORE),
        10,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};trading", module.id, ITEM_ICE_ORE).as_str(),
        0 // Initial cached price, will be updated immediately
    )?;
    // Calculate and set initial cached current price
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_ICE_ORE)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item.clone())?;
    }
    dsl.create_trading_port_listing(item.get_id(), Some(0.8), None)?;

    item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_IRON_ORE),
        20,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};trading", module.id, ITEM_IRON_ORE).as_str(),
        0 // Initial cached price, will be updated immediately
    )?;
    // Calculate and set initial cached current price
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_IRON_ORE)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item.clone())?;
    }
    dsl.create_trading_port_listing(item.get_id(), Some(0.8), None)?;

    item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_SILICON_ORE),
        40,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};trading", module.id, ITEM_SILICON_ORE).as_str(),
        0 // Initial cached price, will be updated immediately
    )?;
    // Calculate and set initial cached current price
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_SILICON_ORE)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item.clone())?;
    }
    dsl.create_trading_port_listing(item.get_id(), Some(0.8), None)?;

    item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_ENERGY_CELL),
        1000,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};trading", module.id, ITEM_ENERGY_CELL).as_str(),
        0 // Initial cached price, will be updated immediately
    )?;
    // Calculate and set initial cached current price
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_ENERGY_CELL)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item.clone())?;
    }
    dsl.create_trading_port_listing(item.get_id(), Some(0.5), None)?;

    item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_FOOD_RATIONS),
        100,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};trading", module.id, ITEM_FOOD_RATIONS).as_str(),
        0 // Initial cached price, will be updated immediately
    )?;
    // Calculate and set initial cached current price
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_FOOD_RATIONS)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item.clone())?;
    }
    dsl.create_trading_port_listing(item.get_id(), Some(0.5), None)?;

    item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_FOOD_AVERAGE),
        100,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};trading", module.id, ITEM_FOOD_AVERAGE).as_str(),
        0 // Initial cached price, will be updated immediately
    )?;
    // Calculate and set initial cached current price
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_FOOD_AVERAGE)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item.clone())?;
    }
    dsl.create_trading_port_listing(item.get_id(), Some(0.5), None)?;

    Ok(())
}