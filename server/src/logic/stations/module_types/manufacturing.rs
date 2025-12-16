use log::info;
use spacetimedb::{table, Identity, SpacetimeType, Timestamp};
use spacetimedsl::*;

use crate::definitions::item_types::*;
use crate::tables::economy::ResourceAmount;
use crate::tables::items::*;
use crate::tables::stations::*;

/// Defines a recipe that a manufacturing module can use.
#[dsl(plural_name = production_recipe_definitions, method(update = true))]
#[table(name = production_recipe_definition, public)]
pub struct ProductionRecipeDefinition {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u32,

    #[unique]
    pub name: String, // e.g., "Basic Hull Plating", "Mk1 Laser Cannon Assembly"

    pub input_resources: Vec<ResourceAmount>,

    #[use_wrapper(crate::tables::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub output_resource_id: u32, // FK to ResourceDefinition

    pub output_quantity: u32,
    pub base_production_time_seconds: u32,
    /// Which type of module can use this recipe (e.g., Factory, Assembler)
    pub required_module_specific_type: StationModuleSpecificType,
    pub required_tech_id_to_unlock: Option<u32>, // FK to TechnologyTreeNode
}

/// Data for a generic manufacturing module instance (Factory, Assembler, Fabricator).
#[dsl(plural_name = manufacturing_modules, method(update = true))]
#[table(name = manufacturing_module, public)]
pub struct Manufacturing {
    #[primary_key]
    #[use_wrapper(StationModuleId)]
    /// FK to StationModule
    id: u64,

    #[use_wrapper(ProductionRecipeDefinitionId)]
    /// The recipe this specific module instance is currently configured to produce. FK to ProductionRecipeDefinition
    pub current_recipe_id: Option<u32>,

    pub is_producing: bool,
    pub production_queue_count: u32, // Number of items queued for production
    pub current_production_progress_seconds: f32,
    /// Modifier based on upgrades, staffing, etc. Affects production_time_seconds.
    pub production_speed_modifier: f32, // Default 1.0
}

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

#[derive(Clone, Debug)]
pub struct ManufacturingProductionResult {
    pub items_completed: u32,
    pub progress_made: f32,
    pub inputs_consumed: Vec<(u32, u32)>, // (resource_id, quantity)
    pub was_limited_by_inputs: bool,
}

/////////////////////////////////////////////////////////////////
/// Create Module

pub fn create_basic_manufacturing_module<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station: &Station,
    under_construction: bool,
    manufacturing_type: ManufacturingType,
) -> Result<(), String> {
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
        "Manufacturing",
        true,
        None,
        dsl.ctx().timestamp(),
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
    create_manufacturing_inventory_slot(&dsl, &module, &blueprint, ITEM_IRON_INGOT, "iron_ingot")?;
    create_manufacturing_inventory_slot(
        &dsl,
        &module,
        &blueprint,
        ITEM_SILICON_RAW,
        "silicon_raw",
    )?;
    create_manufacturing_inventory_slot(&dsl, &module, &blueprint, ITEM_CARBON_RAW, "carbon_raw")?;

    // Create slots for manufactured components
    create_manufacturing_inventory_slot(
        &dsl,
        &module,
        &blueprint,
        ITEM_METAL_PLATES,
        "metal_plates",
    )?;
    create_manufacturing_inventory_slot(
        &dsl,
        &module,
        &blueprint,
        ITEM_METAL_LINKAGES,
        "metal_linkages",
    )?;
    create_manufacturing_inventory_slot(
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
            &dsl,
            &module,
            &blueprint,
            ITEM_COMPUTER_WAFERS,
            "computer_wafers",
        )?;
        create_manufacturing_inventory_slot(
            &dsl,
            &module,
            &blueprint,
            ITEM_COMPUTER_CHIPS,
            "computer_chips",
        )?;
        create_manufacturing_inventory_slot(
            &dsl,
            &module,
            &blueprint,
            ITEM_COMPUTER_BOARD,
            "computer_board",
        )?;
        create_manufacturing_inventory_slot(
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
            &dsl,
            &module,
            &blueprint,
            ITEM_METAL_HULL_STRUCTURE,
            "hull_structure",
        )?;
        create_manufacturing_inventory_slot(
            &dsl,
            &module,
            &blueprint,
            ITEM_MODULE_COMPONENTS_SHIP,
            "ship_components",
        )?;
        create_manufacturing_inventory_slot(
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
pub fn create_manufacturing_module_and_recipe<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station: &Station,
    module_name: &str,
    recipe_name: &str,
    input_resources: Vec<ResourceAmount>,
    output_resource_id: u32,
    output_quantity: u32,
    production_time_seconds: u32,
    under_construction: bool,
) -> Result<(), String> {
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
        dsl.ctx().timestamp(),
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
    dsl.create_manufacturing_module(
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
            &dsl,
            &module,
            &blueprint,
            input_resource.resource_item_id,
            &format!("input_{}", input_resource.resource_item_id),
        )?;
    }

    // Create inventory slot for output resource
    create_manufacturing_inventory_slot(
        &dsl,
        &module,
        &blueprint,
        output_resource_id,
        &format!("output_{}", output_resource_id),
    )?;

    Ok(())
}

/// Create a basic manufacturing module that turns iron ingots and energy cells into metal plates
pub fn create_metal_plate_module<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station: &Station,
    under_construction: bool,
) -> Result<(), String> {
    let input_resources = vec![
        ResourceAmount::new(ITEM_IRON_INGOT, 2),  // 2 iron ingots
        ResourceAmount::new(ITEM_ENERGY_CELL, 5), // 5 energy cells
    ];

    create_manufacturing_module_and_recipe(
        dsl,
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

fn create_manufacturing_inventory_slot<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    module: &StationModule,
    blueprint: &StationModuleBlueprint,
    item_id: u32,
    slot_name: &str,
) -> Result<(), String> {
    let mut item = dsl.create_station_module_inventory_item(
        module.get_id(),
        ItemDefinitionId::new(item_id),
        0,
        blueprint
            .get_max_internal_storage_volume_per_slot_m3()
            .unwrap(),
        format!("{};{};{}", module.get_id(), item_id, slot_name).as_str(),
        0,
    )?;

    if let Ok(item_def) = dsl.get_item_definition_by_id(ItemDefinitionId::new(item_id)) {
        let initial_price = item.calculate_current_price(&item_def);
        item.set_cached_price(initial_price);
        dsl.update_station_module_inventory_item_by_id(item)?;
    }

    Ok(())
}

//////////////////////////////////////////////////////////////////////////
/// Utility

/// Calculate the manufacturing production for a manufacturing module
pub fn calculate_manufacturing_production<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    manufacturing: &Manufacturing,
    time_elapsed_seconds: f32,
) -> Result<ManufacturingProductionResult, String> {
    // Check if manufacturing is active and has a recipe
    if !manufacturing.is_producing || manufacturing.current_recipe_id.is_none() {
        info!("Skipping production calculation");
        return Ok(ManufacturingProductionResult {
            items_completed: 0,
            progress_made: 0.0,
            inputs_consumed: Vec::new(),
            was_limited_by_inputs: false,
        });
    }

    let recipe_id = manufacturing.current_recipe_id.unwrap();
    let recipe =
        dsl.get_production_recipe_definition_by_id(ProductionRecipeDefinitionId::new(recipe_id))?;

    let station_module = dsl.get_station_module_by_id(manufacturing.get_id())?;

    // Calculate production time with speed modifier
    let effective_production_time =
        recipe.base_production_time_seconds as f32 / manufacturing.production_speed_modifier;
    info!("effective_production_time: {}", effective_production_time);

    // Calculate progress made this tick
    let progress_increment = time_elapsed_seconds / effective_production_time;
    let new_progress = manufacturing.current_production_progress_seconds + progress_increment;
    info!(
        "progress_increment: {}, new_progress: {}",
        progress_increment, new_progress
    );

    // Check if we can complete items
    let items_that_can_be_completed = new_progress.floor() as u32;
    let remaining_progress = new_progress.fract();
    info!(
        "items_that_can_be_completed: {}, remaining_progress: {}",
        items_that_can_be_completed, remaining_progress
    );

    // Check if we have enough inputs for the items we want to complete
    let (actual_items_completed, inputs_consumed, was_limited) =
        check_and_consume_inputs(&dsl, &station_module, &recipe, items_that_can_be_completed)?;

    Ok(ManufacturingProductionResult {
        items_completed: actual_items_completed,
        progress_made: actual_items_completed as f32 + remaining_progress,
        inputs_consumed,
        was_limited_by_inputs: was_limited,
    })
}

/// Check input availability and calculate what can actually be produced
fn check_and_consume_inputs<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station_module: &StationModule,
    recipe: &ProductionRecipeDefinition,
    desired_items: u32,
) -> Result<(u32, Vec<(u32, u32)>, bool), String> {
    if desired_items == 0 {
        return Ok((0, Vec::new(), false));
    }

    // Check availability of all required inputs
    let mut max_producible = desired_items;
    let mut was_limited = false;

    for input_resource in &recipe.input_resources {
        if let Some(inventory) = dsl
            .get_all_station_module_inventory_items()
            .into_iter()
            .find(|item: &StationModuleInventoryItem| {
                item.module_id == station_module.get_id().value()
                    && item.resource_item_id == input_resource.resource_item_id
            })
        {
            let available_items = inventory.get_quantity() / input_resource.quantity;
            if available_items < max_producible {
                max_producible = available_items;
                was_limited = true;
            }
        } else {
            // Required input not found, can't produce anything
            return Ok((0, Vec::new(), true));
        }
    }

    // Calculate actual consumption
    let mut inputs_consumed = Vec::new();
    for input_resource in &recipe.input_resources {
        let consumed = input_resource.quantity * max_producible;
        inputs_consumed.push((input_resource.resource_item_id, consumed));
    }

    Ok((max_producible, inputs_consumed, was_limited))
}

/// Apply the calculated production results to the manufacturing module's inventory
pub fn apply_manufacturing_production<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    manufacturing: &Manufacturing,
    production_result: &ManufacturingProductionResult,
) -> Result<(), String> {
    let mut updated_manufacturing = manufacturing.clone();

    if production_result.items_completed == 0 {
        updated_manufacturing.set_current_production_progress_seconds(
            manufacturing.get_current_production_progress_seconds()
                + production_result.progress_made,
        );
        dsl.update_manufacturing_module_by_id(updated_manufacturing)?;

        return Ok(());
    }

    let recipe_id = manufacturing.current_recipe_id.unwrap();
    let recipe =
        dsl.get_production_recipe_definition_by_id(ProductionRecipeDefinitionId::new(recipe_id))?;

    let station_module = dsl.get_station_module_by_id(manufacturing.get_id())?;

    // Consume input materials
    for (resource_id, quantity_consumed) in &production_result.inputs_consumed {
        if let Some(mut inventory) = dsl
            .get_all_station_module_inventory_items()
            .into_iter()
            .find(|item: &StationModuleInventoryItem| {
                item.module_id == station_module.get_id().value()
                    && item.resource_item_id == *resource_id
            })
        {
            if inventory.quantity >= *quantity_consumed {
                inventory.quantity = inventory.quantity - quantity_consumed;
                dsl.update_station_module_inventory_item_by_id(inventory)?;
            }
        }
    }

    // Add produced items to output inventory
    if let Some(mut output_inventory) = dsl
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item: &StationModuleInventoryItem| {
            item.module_id == station_module.get_id().value()
                && item.resource_item_id == recipe.output_resource_id
        })
    {
        let items_to_add = production_result.items_completed * recipe.output_quantity;
        output_inventory.quantity = output_inventory.quantity + items_to_add;
        dsl.update_station_module_inventory_item_by_id(output_inventory)?;
    }

    // Update manufacturing progress
    updated_manufacturing.set_current_production_progress_seconds(
        production_result.progress_made - (production_result.items_completed as f32),
    );

    // Update queue count
    if updated_manufacturing.production_queue_count > 0 {
        let new_queue_count = updated_manufacturing
            .production_queue_count
            .saturating_sub(production_result.items_completed);
        updated_manufacturing.set_production_queue_count(new_queue_count);

        // Stop producing if queue is empty
        if new_queue_count == 0 {
            updated_manufacturing.set_is_producing(false);
        }
    }

    dsl.update_manufacturing_module_by_id(updated_manufacturing)?;

    Ok(())
}

/// Calculate efficiency modifiers for manufacturing production
pub fn calculate_manufacturing_efficiency<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    manufacturing: &Manufacturing,
) -> Result<f32, String> {
    let station_module = dsl.get_station_module_by_id(manufacturing.get_id())?;
    let station = dsl.get_station_by_id(station_module.get_station_id())?;

    let mut efficiency = manufacturing.production_speed_modifier;

    // Station health affects efficiency
    if let Ok(station_status) = dsl.get_station_status_by_id(station.get_id()) {
        efficiency *= (station_status.get_health() / 100.0).max(0.1);
    }

    // Module operational status
    if !*station_module.get_is_operational() {
        efficiency = 0.0;
    }

    Ok(efficiency.max(0.0).min(3.0)) // Manufacturing can be more efficient than other modules
}
