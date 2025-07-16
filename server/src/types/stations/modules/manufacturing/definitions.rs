use super::*;
use crate::types::items::{definitions::*, GetItemDefinitionRowOptionById, ItemDefinitionId};

pub fn create_basic_manufacturing_module(
    ctx: &ReducerContext,
    station: &Station,
    under_construction: bool,
    manufacturing_type: ManufacturingType,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if under_construction {
        return Err("Not yet implemented".to_string());
    }

    let blueprint_id = match manufacturing_type {
        ManufacturingType::BasicFactory => MODULE_MANUFACTURING_FACTORY,
        ManufacturingType::AdvancedFactory => MODULE_MANUFACTURING_FACTORY_ADVANCED,
        ManufacturingType::ComponentAssembler => MODULE_MANUFACTURING_ASSEMBLER,
        ManufacturingType::ShipyardFabrication => MODULE_MANUFACTURING_SHIPYARD,
    };

    let blueprint =
        dsl.get_station_module_blueprint_by_id(StationModuleBlueprintId::new(blueprint_id))?;

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        "manufacturing",
        true,
        None,
        ctx.timestamp,
    )?;

    // Create manufacturing submodule
    let manufacturing = dsl.create_manufacturing_module(
        module.get_id(),
        None,  // No recipe selected initially
        false, // Not producing initially
        0,     // No items queued
        0.0,   // No progress
        1.0,   // Default speed modifier
    )?;

    // Create basic inventory slots for common materials
    create_manufacturing_inventory_slot(
        ctx,
        &dsl,
        &module,
        &blueprint,
        ITEM_IRON_INGOT,
        "iron_ingot",
    )?;
    create_manufacturing_inventory_slot(
        ctx,
        &dsl,
        &module,
        &blueprint,
        ITEM_SILICON_RAW,
        "silicon_raw",
    )?;
    create_manufacturing_inventory_slot(
        ctx,
        &dsl,
        &module,
        &blueprint,
        ITEM_CARBON_RAW,
        "carbon_raw",
    )?;

    // Create slots for manufactured components
    create_manufacturing_inventory_slot(
        ctx,
        &dsl,
        &module,
        &blueprint,
        ITEM_METAL_PLATES,
        "metal_plates",
    )?;
    create_manufacturing_inventory_slot(
        ctx,
        &dsl,
        &module,
        &blueprint,
        ITEM_METAL_LINKAGES,
        "metal_linkages",
    )?;
    create_manufacturing_inventory_slot(
        ctx,
        &dsl,
        &module,
        &blueprint,
        ITEM_MODULE_COMPONENTS,
        "module_components",
    )?;

    // Advanced manufacturing gets additional slots
    if matches!(
        manufacturing_type,
        ManufacturingType::AdvancedFactory | ManufacturingType::ComponentAssembler
    ) {
        create_manufacturing_inventory_slot(
            ctx,
            &dsl,
            &module,
            &blueprint,
            ITEM_COMPUTER_WAFERS,
            "computer_wafers",
        )?;
        create_manufacturing_inventory_slot(
            ctx,
            &dsl,
            &module,
            &blueprint,
            ITEM_COMPUTER_CHIPS,
            "computer_chips",
        )?;
        create_manufacturing_inventory_slot(
            ctx,
            &dsl,
            &module,
            &blueprint,
            ITEM_COMPUTER_BOARD,
            "computer_board",
        )?;
        create_manufacturing_inventory_slot(
            ctx,
            &dsl,
            &module,
            &blueprint,
            ITEM_MODULE_COMPONENTS_ADVANCED,
            "advanced_components",
        )?;
    }

    // Shipyard gets ship-specific components
    if matches!(manufacturing_type, ManufacturingType::ShipyardFabrication) {
        create_manufacturing_inventory_slot(
            ctx,
            &dsl,
            &module,
            &blueprint,
            ITEM_METAL_HULL_STRUCTURE,
            "hull_structure",
        )?;
        create_manufacturing_inventory_slot(
            ctx,
            &dsl,
            &module,
            &blueprint,
            ITEM_MODULE_COMPONENTS_SHIP,
            "ship_components",
        )?;
        create_manufacturing_inventory_slot(
            ctx,
            &dsl,
            &module,
            &blueprint,
            ITEM_MODULE_COMPONENTS_ENGINE,
            "engine_components",
        )?;
    }

    Ok(())
}

fn create_manufacturing_inventory_slot(
    ctx: &ReducerContext,
    dsl: &DSL,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
    item_id: u32,
    slot_name: &str,
) -> Result<(), String> {
    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(item_id),
        0,
        blueprint.max_internal_storage_volume_per_slot_m3.unwrap(),
        format!("{};{};{}", module.id, item_id, slot_name).as_str(),
        0,
    )?;

    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(item_id)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub enum ManufacturingType {
    BasicFactory,        // Basic metal processing
    AdvancedFactory,     // Advanced components
    ComponentAssembler,  // Module components
    ShipyardFabrication, // Ship parts
}

// Add missing module constants
pub const MODULE_MANUFACTURING_FACTORY: u32 = 6_000;
pub const MODULE_MANUFACTURING_FACTORY_ADVANCED: u32 = 6_001;
pub const MODULE_MANUFACTURING_ASSEMBLER: u32 = 6_002;
pub const MODULE_MANUFACTURING_SHIPYARD: u32 = 6_003;
