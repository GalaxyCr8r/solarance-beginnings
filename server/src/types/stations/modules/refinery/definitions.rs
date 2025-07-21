use super::*;
use crate::types::{
    items::{GetItemDefinitionRowOptionById, ItemDefinitionId},
    stations::definitions::MODULE_REFINERY_MINOR,
};

pub fn create_basic_refinery_module(
    ctx: &ReducerContext,
    station: &Station,
    under_construction: bool,
    input_resource: ItemDefinitionId,
    output_resource: ItemDefinitionId,
    waste_resource: Option<ItemDefinitionId>,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    //
    if under_construction {
        return Err("Not yet implemented".to_string());
    }

    let blueprint = dsl
        .get_station_module_blueprint_by_id(StationModuleBlueprintId::new(MODULE_REFINERY_MINOR))?;

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        "refinery", // TODO: Do we even need this field?
        true,
        None,
        ctx.timestamp,
    )?;

    // Submodule
    let iron_ref = dsl.create_refinery_module(
        module.get_id(),
        &input_resource,
        &output_resource,
        None,
        5,
        0,
        30,
        1.0,
    )?;

    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        &input_resource,
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};input", module.id, iron_ref.id).as_str(),
        0, // Initial cached price, will be updated immediately
    )?;
    // Calculate and set initial cached current price
    if let Ok(item_def) = dsl.get_item_definition_by_id(&input_resource) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        &output_resource,
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};output", module.id, iron_ref.id).as_str(),
        0, // Initial cached price, will be updated immediately
    )?;
    // Calculate and set initial cached current price
    if let Ok(item_def) = dsl.get_item_definition_by_id(&output_resource) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    if let Some(waste) = waste_resource {
        let mut item = dsl.create_station_module_inventory_item(
            module.get_id(),
            &waste,
            0,
            blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
            format!("{};{};waste", module.id, iron_ref.id).as_str(),
            0, // Initial cached price, will be updated immediately
        )?;
        // Calculate and set initial cached current price
        if let Ok(item_def) = dsl.get_item_definition_by_id(&waste) {
            let initial_price = item.calculate_current_price(&item_def);
            item.set_cached_price(initial_price);
            dsl.update_station_module_inventory_item_by_id(item)?;
        }
    }

    Ok(())
}
