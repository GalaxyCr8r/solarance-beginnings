use super::*;
use crate::types::items::{definitions::*, GetItemDefinitionRowOptionById, ItemDefinitionId};

pub fn create_basic_solar_array(
    ctx: &ReducerContext,
    station: &Station,
    under_construction: bool,
    array_size: SolarArraySize,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if under_construction {
        return Err("Not yet implemented".to_string());
    }

    let (blueprint_id, base_production) = match array_size {
        SolarArraySize::Small => (MODULE_SOLAR_ARRAY_SMALL, 100),
        SolarArraySize::Medium => (MODULE_SOLAR_ARRAY_MEDIUM, 250),
        SolarArraySize::Large => (MODULE_SOLAR_ARRAY_LARGE, 500),
        SolarArraySize::Industrial => (MODULE_SOLAR_ARRAY_INDUSTRIAL, 1000),
    };

    let blueprint =
        dsl.get_station_module_blueprint_by_id(StationModuleBlueprintId::new(blueprint_id))?;
    let identifier = format!("{:?} Solar Array", array_size);

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        identifier.as_str(),
        true,
        None,
        ctx.timestamp,
    )?;

    // Create solar array submodule
    let solar_array = dsl.create_solar_array_module(
        module.get_id(),
        ItemDefinitionId::new(ITEM_ENERGY_CELL), // Always produces energy cells
        base_production,
        1.0, // Default efficiency modifier
    )?;

    // Create output inventory for energy cells
    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(ITEM_ENERGY_CELL),
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};output", module.id, solar_array.id).as_str(),
        0,
    )?;

    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_ENERGY_CELL)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub enum SolarArraySize {
    Small,      // 100 energy cells/hour
    Medium,     // 250 energy cells/hour
    Large,      // 500 energy cells/hour
    Industrial, // 1000 energy cells/hour
}

// Add missing module constants
pub const MODULE_SOLAR_ARRAY_SMALL: u32 = 7_000;
pub const MODULE_SOLAR_ARRAY_MEDIUM: u32 = 7_001;
pub const MODULE_SOLAR_ARRAY_LARGE: u32 = 7_002;
pub const MODULE_SOLAR_ARRAY_INDUSTRIAL: u32 = 7_003;
