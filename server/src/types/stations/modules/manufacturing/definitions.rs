use super::*;
use crate::types::items::{definitions::*, GetItemDefinitionRowOptionById, ItemDefinitionId};

#[derive(Clone, Debug)]
pub enum ManufacturingType {
    BasicFactory,        // Basic metal processing
    AdvancedFactory,     // Advanced components
    ComponentAssembler,  // Module components
    ShipyardFabrication, // Ship parts
}

// Manufacturing module constants
pub const MODULE_MANUFACTURING_FACTORY: u32 = 6_000;
pub const MODULE_MANUFACTURING_FACTORY_ADVANCED: u32 = 6_001;
pub const MODULE_MANUFACTURING_ASSEMBLER: u32 = 6_002;
pub const MODULE_MANUFACTURING_SHIPYARD: u32 = 6_003;

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
    let _manufacturing = dsl.create_manufacturing_module(
        module.get_id(),
        None, // No recipe selected initially
        true, // Attempt producing initially
        0,    // No items queued
        0.0,  // No progress
        1.0,  // Default speed modifier
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

/// Generic function to create a manufacturing module with a specific recipe
pub fn create_manufacturing_module_and_recipe(
    ctx: &ReducerContext,
    station: &Station,
    module_name: &str,
    recipe_name: &str,
    input_resources: Vec<ResourceAmount>,
    output_resource_id: u32,
    output_quantity: u32,
    production_time_seconds: u32,
    under_construction: bool,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if under_construction {
        return Err("Not yet implemented".to_string());
    }

    let blueprint = dsl.get_station_module_blueprint_by_id(StationModuleBlueprintId::new(
        MODULE_MANUFACTURING_FACTORY,
    ))?;

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        module_name,
        true,
        None,
        ctx.timestamp,
    )?;

    // Create the recipe
    let recipe = dsl.create_production_recipe_definition(
        recipe_name,
        input_resources.clone(),
        ItemDefinitionId::new(output_resource_id),
        output_quantity,
        production_time_seconds,
        StationModuleSpecificType::FactoryBasicComponents,
        None, // No tech requirement
    )?;

    // Create manufacturing submodule with the recipe
    let _manufacturing = dsl.create_manufacturing_module(
        module.get_id(),
        Some(recipe.get_id()), // Set the recipe
        true,                  // Start producing
        0,                     // No items queued initially
        0.0,                   // No progress
        1.0,                   // Default speed modifier
    )?;

    // Create inventory slots for all input resources
    for input_resource in &input_resources {
        create_manufacturing_inventory_slot(
            ctx,
            &dsl,
            &module,
            &blueprint,
            input_resource.resource_item_id,
            &format!("input_{}", input_resource.resource_item_id),
        )?;
    }

    // Create inventory slot for output resource
    create_manufacturing_inventory_slot(
        ctx,
        &dsl,
        &module,
        &blueprint,
        output_resource_id,
        &format!("output_{}", output_resource_id),
    )?;

    Ok(())
}

/// Create a basic manufacturing module that turns iron ingots and energy cells into metal plates
pub fn create_metal_plate_module(
    ctx: &ReducerContext,
    station: &Station,
    under_construction: bool,
) -> Result<(), String> {
    let input_resources = vec![
        ResourceAmount::new(ITEM_IRON_INGOT, 2),  // 2 iron ingots
        ResourceAmount::new(ITEM_ENERGY_CELL, 5), // 5 energy cells
    ];

    create_manufacturing_module_and_recipe(
        ctx,
        station,
        "metal_plate_factory",
        "Basic Metal Plate Production",
        input_resources,
        ITEM_METAL_PLATES,
        3,  // Produces 3 metal plates
        60, // Takes 60 seconds
        under_construction,
    )
}

fn create_manufacturing_inventory_slot(
    _ctx: &ReducerContext,
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
