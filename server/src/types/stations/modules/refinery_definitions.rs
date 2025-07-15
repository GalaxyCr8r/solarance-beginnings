use super::*;
use crate::types::items::GetItemDefinitionRowOptionById;

pub fn create_basic_iron(
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
        StationModuleBlueprintId::new(definitions::MODULE_REFINERY_MINOR)
    )?;

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        "refinery", // TODO: Do we even need this field?
        true,
        None,
        ctx.timestamp
    )?;

    // // Ice Submodule
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

    // Iron Submodule
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

    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_IRON_ORE),
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};input", module.id, iron_ref.id).as_str(),
        0 // Initial cached price, will be updated immediately
    )?;
    // Calculate and set initial cached current price
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_IRON_ORE)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_IRON_INGOT),
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};output", module.id, iron_ref.id).as_str(),
        0 // Initial cached price, will be updated immediately
    )?;
    // Calculate and set initial cached current price
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_IRON_INGOT)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    // // Silicon Submodule
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
