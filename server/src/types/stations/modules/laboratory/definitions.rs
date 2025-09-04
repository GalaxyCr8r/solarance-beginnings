use super::*;
use crate::types::items::{definitions::*, GetItemDefinitionRowOptionById, ItemDefinitionId};

pub fn create_basic_laboratory(
    ctx: &ReducerContext,
    station: &Station,
    under_construction: bool,
    lab_type: LaboratoryType,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if under_construction {
        return Err("Not yet implemented".to_string());
    }

    let (blueprint_id, primary_input, secondary_input, research_points_per_hour) = match lab_type {
        LaboratoryType::Basic => (MODULE_LABORATORY_BASIC, ITEM_VIVEIUM_CRYSTAL, None, 50),
        LaboratoryType::Advanced => (
            MODULE_LABORATORY_ADVANCED,
            ITEM_VIVEIUM_CRYSTAL,
            Some(ITEM_RESEARCH_DEVICE),
            100,
        ),
        LaboratoryType::Exotic => (
            MODULE_LABORATORY_EXOTIC,
            ITEM_RESEARCH_FRAGMENT_EXOTIC,
            Some(ITEM_RESEARCH_DEVICE_RARE),
            200,
        ),
    };

    let blueprint =
        dsl.get_station_module_blueprint_by_id(StationModuleBlueprintId::new(blueprint_id))?;
    let identifier = format!("{:?} Laboratory", lab_type);

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        identifier.as_str(),
        true,
        None,
        ctx.timestamp,
    )?;

    // Create laboratory submodule
    let lab = dsl.create_laboratory_module(
        module.get_id(),
        research_points_per_hour,
        ItemDefinitionId::new(primary_input),
        secondary_input.map(|id| ItemDefinitionId::new(id)),
        1.0,                          // Primary consumption rate (1 unit per hour)
        secondary_input.map(|_| 0.5), // Secondary consumption rate (0.5 units per hour if present)
        1.0,                          // Default efficiency
    )?;

    // Create inventory slots for inputs
    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(primary_input),
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};input_primary", module.id, lab.id).as_str(),
        0,
    )?;
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(primary_input)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    // Create secondary input inventory if needed
    if let Some(secondary_id) = secondary_input {
        let mut item = dsl.create_station_module_inventory_item(
            module.get_id(),
            ItemDefinitionId::new(secondary_id),
            0,
            blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
            format!("{};{};input_secondary", module.id, lab.id).as_str(),
            0,
        )?;
        if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(secondary_id)) {
            let initial_price = item.calculate_current_price(&item_def);
            item.set_cached_price(initial_price);
            dsl.update_station_module_inventory_item_by_id(item)?;
        }
    }

    // Create output inventory for research fragments
    let output_fragment = match lab_type {
        LaboratoryType::Basic => ITEM_RESEARCH_FRAGMENT,
        LaboratoryType::Advanced => ITEM_RESEARCH_FRAGMENT_RARE,
        LaboratoryType::Exotic => ITEM_RESEARCH_FRAGMENT_EXOTIC,
    };

    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(output_fragment),
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};output", module.id, lab.id).as_str(),
        0,
    )?;
    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(output_fragment)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub enum LaboratoryType {
    Basic,
    Advanced,
    Exotic,
}

// Add missing module constants
pub const MODULE_LABORATORY_BASIC: u32 = 5_000;
pub const MODULE_LABORATORY_ADVANCED: u32 = 5_001;
pub const MODULE_LABORATORY_EXOTIC: u32 = 5_002;
