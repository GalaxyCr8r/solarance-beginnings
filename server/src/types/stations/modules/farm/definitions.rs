use super::*;
use crate::types::items::{definitions::*, GetItemDefinitionRowOptionById, ItemDefinitionId};

pub fn create_basic_food_farm(
    ctx: &ReducerContext,
    station: &Station,
    under_construction: bool,
    output_quality: FarmOutputQuality,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if under_construction {
        return Err("Not yet implemented".to_string());
    }

    // Determine output resource based on quality
    let (output_resource, blueprint_id) = match output_quality {
        FarmOutputQuality::Lower => (ITEM_FOOD_RATIONS, MODULE_FARM_BASIC),
        FarmOutputQuality::Average => (ITEM_FOOD_AVERAGE, MODULE_FARM_STANDARD),
        FarmOutputQuality::Upper => (ITEM_FOOD_LUXURY, MODULE_FARM_ADVANCED),
        FarmOutputQuality::Luxury => (ITEM_FOOD_LUXURY, MODULE_FARM_LUXURY),
    };

    let blueprint =
        dsl.get_station_module_blueprint_by_id(StationModuleBlueprintId::new(blueprint_id))?;

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        "farm",
        true,
        None,
        ctx.timestamp,
    )?;

    // Create farm submodule
    let farm = dsl.create_farm_module(
        module.get_id(),
        output_resource,
        output_quality.clone(),
        ItemDefinitionId::new(ITEM_BIOMATTER_COMPOST), // Primary input
        2.0,                                           // 2 units of compost per unit of food
        Some(ItemDefinitionId::new(ITEM_WATER)),       // Secondary input
        Some(0.5),                                     // 0.5 units of water per unit of food
        10.0,                                          // Base production: 10 units per hour
        1.0,                                           // Default efficiency
    )?;

    // Create inventory slots for inputs and outputs
    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_BIOMATTER_COMPOST),
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};input_primary", module.id, farm.id).as_str(),
        0,
    )?;
    if let Ok(item_def) =
        dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_BIOMATTER_COMPOST))
    {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_WATER),
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};input_secondary", module.id, farm.id).as_str(),
        0,
    )?;
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_WATER)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(output_resource),
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};output", module.id, farm.id).as_str(),
        0,
    )?;
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(output_resource)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    Ok(())
}

// Add missing module constants to stations/definitions.rs
pub const MODULE_FARM_BASIC: u32 = 4_000;
pub const MODULE_FARM_STANDARD: u32 = 4_001;
pub const MODULE_FARM_ADVANCED: u32 = 4_002;
pub const MODULE_FARM_LUXURY: u32 = 4_003;
