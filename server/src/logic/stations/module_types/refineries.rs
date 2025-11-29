use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::definitions::station_module_types::*;
use crate::tables::items::*;
use crate::tables::stations::*;

#[dsl(plural_name = refinery_modules)]
#[table(name = refinery_module, public)]
pub struct Refinery {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,

    #[use_wrapper(path = crate::tables::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub input_ore_resource_id: u32, // FK to ResourceDefinition

    #[use_wrapper(path = crate::tables::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub output_ingot_resource_id: u32, // FK to ResourceDefinition

    #[use_wrapper(path = crate::tables::items::ItemDefinitionId)]
    /// FK to ItemDefinition
    pub waste_resource_id: Option<u32>, // FK to ResourceDefinition

    /// How many units of ore to make 1 unit of ingot.
    pub ore_to_ingot_ratio: u32,
    /// How many units of waste are produced to make 1 unit of ingot.
    pub waste_per_ingot_ratio: u32,

    pub base_ingots_produced_per_hour: u32,
    pub current_efficiency_modifier: f32, // Default 1.0
}

#[derive(Clone, Debug)]
pub struct RefineryProductionResult {
    pub ingots_produced: u32,
    pub ore_consumed: u32,
    pub waste_produced: u32,
    pub was_limited_by_ore: bool,
}

//////////////////////////////////////////////
/// Create Module
///

pub fn create_basic_refinery_module(
    dsl: &DSL,
    station: &Station,
    under_construction: bool,
    input_resource: ItemDefinitionId,
    output_resource: ItemDefinitionId,
    waste_resource: Option<ItemDefinitionId>,
) -> Result<(), String> {
    if under_construction {
        return Err("Not yet implemented".to_string());
    }

    let blueprint = dsl
        .get_station_module_blueprint_by_id(StationModuleBlueprintId::new(MODULE_REFINERY_MINOR))?;

    let output_item_def = dsl.get_item_definition_by_id(&output_resource)?;
    let identifier = format!("{} Refinery", output_item_def.get_name());

    let module = dsl.create_station_module(
        station.get_id(),
        blueprint.get_id(),
        identifier.as_str(), // TODO: Do we even need this field?
        true,
        None,
        dsl.ctx().timestamp,
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
        blueprint
            .get_max_internal_storage_volume_per_slot_m3()
            .unwrap(),
        format!("{};{};input", module.get_id(), iron_ref.get_id()).as_str(),
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
        blueprint
            .get_max_internal_storage_volume_per_slot_m3()
            .unwrap(),
        format!("{};{};output", module.get_id(), iron_ref.get_id()).as_str(),
        0, // Initial cached price, will be updated immediately
    )?;
    // Calculate and set initial cached current price
    let initial_price = item.calculate_current_price(&output_item_def);
    item.set_cached_price(initial_price);
    dsl.update_station_module_inventory_item_by_id(item)?;

    if let Some(waste) = waste_resource {
        let mut item = dsl.create_station_module_inventory_item(
            module.get_id(),
            &waste,
            0,
            blueprint
                .get_max_internal_storage_volume_per_slot_m3()
                .unwrap(),
            format!("{};{};waste", module.get_id(), iron_ref.get_id()).as_str(),
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

////////////////////////////////////////////////////
/// Utilities

/// Calculate the production output for a refinery module based on available input resources
/// and current efficiency modifiers. Only produces whole units of output.
pub fn calculate_refinery_production(
    dsl: &DSL,
    refinery: &Refinery,
    _time_elapsed_hours: f32,
) -> Result<RefineryProductionResult, String> {
    // Get the station module to access inventory
    let station_module = dsl.get_station_module_by_id(refinery.get_id())?;

    // Find input ore inventory item
    let input_inventory = dsl
        .get_station_module_inventory_items_by_module_id(refinery.get_id())
        .find(|item| item.get_resource_item_id() == refinery.get_input_ore_resource_id())
        .ok_or("Input ore inventory not found")?;

    // info!(
    //     "Input: id{} Amount: {}/{}",
    //     input_inventory.get_id(), input_inventory.get_quantity(), input_inventory.max_quantity
    // );

    // Calculate maximum possible whole ingots based on available ore
    let max_whole_ingots_from_ore = (*input_inventory.get_quantity() as f32
        / refinery.ore_to_ingot_ratio as f32)
        .floor() as u32;

    // info!(
    //     "max_whole_ingots_from_ore: {} / {} = {}",
    //     input_inventory.get_quantity(), refinery.ore_to_ingot_ratio, max_whole_ingots_from_ore
    // );

    // Calculate production capacity based on time and efficiency (as whole units)
    // let production_capacity_float = refinery.base_ingots_produced_per_hour
    //     * refinery.current_efficiency_modifier
    //     * time_elapsed_hours;
    // let production_capacity_whole = production_capacity_float.floor() as u32;

    // info!(
    //     "production_capacity_float: {} / {} = {}",
    //     refinery.base_ingots_produced_per_hour,
    //     refinery.current_efficiency_modifier,
    //     time_elapsed_hours
    // );

    // Actual production is limited by available ore or production capacity (whole units only)
    let actual_ingots_produced_whole = max_whole_ingots_from_ore; // max_whole_ingots_from_ore.min(production_capacity_whole);

    // info!(
    //     "actual_ingots_produced_whole: min({}, {})",
    //     max_whole_ingots_from_ore, production_capacity_whole
    // );

    // Only proceed if we can produce at least 1 whole ingot
    if actual_ingots_produced_whole < 1 {
        info!(
            "Refinery module #{} cannot produce whole ingots: max from ore={}, capacity={}",
            station_module.get_id(),
            max_whole_ingots_from_ore,
            -1 //production_capacity_whole
        );
        return Ok(RefineryProductionResult {
            ingots_produced: 0,
            ore_consumed: 0,
            waste_produced: 0,
            was_limited_by_ore: max_whole_ingots_from_ore < 1,
        });
    }
    // info!(
    //     "Refinery module #{}: Calculating Input: '{}' amount: {}/{}, Output '{}'",
    //     station_module.get_id(),
    //     dsl.get_item_definition_by_id(refinery.get_input_ore_resource_id())?
    //         .name,
    //     input_inventory.get_quantity(),
    //     input_inventory.max_quantity,
    //     dsl.get_item_definition_by_id(refinery.get_output_ingot_resource_id())?
    //         .name
    // );

    // Convert to u32 for final calculations
    let ingots_produced = actual_ingots_produced_whole as u32;

    // Calculate exact resource consumption for the whole ingots we're producing
    let ore_consumed = ingots_produced * refinery.ore_to_ingot_ratio;
    let waste_produced = ingots_produced * refinery.waste_per_ingot_ratio;

    spacetimedb::log::info!(
        "Refinery module {} producing {} whole ingots from {} ore ({} waste produced)",
        station_module.get_id(),
        ingots_produced,
        ore_consumed,
        waste_produced
    );

    Ok(RefineryProductionResult {
        ingots_produced,
        ore_consumed,
        waste_produced,
        was_limited_by_ore: max_whole_ingots_from_ore < 1,
    })
}

/// Apply the calculated production results to the refinery's inventory
pub fn apply_refinery_production(
    dsl: &DSL,
    refinery: &Refinery,
    production_result: &RefineryProductionResult,
) -> Result<(), String> {
    if production_result.ingots_produced == 0 {
        spacetimedb::log::info!("No ingots produced, skipping production application");
        return Ok(()); // No production to apply
    }

    let station_module = dsl.get_station_module_by_id(refinery.get_id())?;

    spacetimedb::log::info!(
        "Applying refinery production: {} ingots, {} ore consumed for module {}",
        production_result.ingots_produced,
        production_result.ore_consumed,
        station_module.get_id()
    );

    // Get all inventory items for this module
    let module_inventory_items: Vec<_> = dsl
        .get_station_module_inventory_items_by_module_id(refinery.get_id())
        .collect();

    spacetimedb::log::info!(
        "Found {} inventory items for module {}",
        module_inventory_items.len(),
        station_module.get_id()
    );

    // Update input ore inventory (consume ore)
    let mut input_found = false;
    for mut input_inventory in module_inventory_items.iter().cloned() {
        if input_inventory.get_resource_item_id() == refinery.get_input_ore_resource_id() {
            input_found = true;
            let ore_to_consume = production_result.ore_consumed as u32;
            spacetimedb::log::info!(
                "Found input inventory with {} ore, consuming {}",
                input_inventory.get_quantity(),
                ore_to_consume
            );

            if *input_inventory.get_quantity() >= ore_to_consume {
                input_inventory.set_quantity(input_inventory.get_quantity() - ore_to_consume);
                dsl.update_station_module_inventory_item_by_id(input_inventory)?;
                spacetimedb::log::info!("Successfully consumed {} ore", ore_to_consume);
            } else {
                spacetimedb::log::info!(
                    "Not enough ore to consume: {} available, {} needed",
                    input_inventory.get_quantity(),
                    ore_to_consume
                );
            }
            break;
        }
    }

    if !input_found {
        spacetimedb::log::info!(
            "WARNING: Input ore inventory not found for resource_id {}",
            refinery.input_ore_resource_id
        );
    }

    // Update output ingot inventory (add ingots)
    let mut output_found = false;
    for mut output_inventory in module_inventory_items.iter().cloned() {
        if output_inventory.get_resource_item_id() == refinery.get_output_ingot_resource_id() {
            output_found = true;
            let ingots_to_add = production_result.ingots_produced as u32;
            spacetimedb::log::info!(
                "Found output inventory with {} ingots, adding {}",
                output_inventory.get_quantity(),
                ingots_to_add
            );

            output_inventory.set_quantity(output_inventory.get_quantity() + ingots_to_add);
            dsl.update_station_module_inventory_item_by_id(output_inventory)?;
            spacetimedb::log::info!("Successfully added {} ingots", ingots_to_add);
            break;
        }
    }

    if !output_found {
        spacetimedb::log::info!(
            "WARNING: Output ingot inventory not found for resource_id {}",
            refinery.output_ingot_resource_id
        );
    }

    // Update waste inventory if waste is produced
    if let Some(waste_resource_id) = refinery.waste_resource_id {
        if production_result.waste_produced > 0 {
            let mut waste_found = false;
            for mut waste_inventory in module_inventory_items.iter().cloned() {
                if waste_inventory.get_resource_item_id().value() == waste_resource_id {
                    waste_found = true;
                    let waste_to_add = production_result.waste_produced as u32;
                    waste_inventory.set_quantity(waste_inventory.get_quantity() + waste_to_add);
                    dsl.update_station_module_inventory_item_by_id(waste_inventory)?;
                    spacetimedb::log::info!("Added {} waste", waste_to_add);
                    break;
                }
            }

            if !waste_found {
                spacetimedb::log::info!(
                    "WARNING: Waste inventory not found for resource_id {}",
                    waste_resource_id
                );
            }
        }
    }

    Ok(())
}

/// Calculate efficiency modifiers based on station conditions, upgrades, etc.
pub fn calculate_refinery_efficiency(dsl: &DSL, refinery: &Refinery) -> Result<f32, String> {
    // Get the station module to check conditions
    let station_module = dsl.get_station_module_by_id(refinery.get_id())?;
    let station = dsl.get_station_by_id(station_module.get_station_id())?;

    let mut efficiency = 1.0;

    // Base efficiency from the refinery's current modifier
    efficiency *= refinery.current_efficiency_modifier;

    // Station health affects efficiency (get from StationStatus)
    if let Ok(station_status) = dsl.get_station_status_by_id(station.get_id()) {
        efficiency *= (station_status.get_health() / 100.0).max(0.1); // Minimum 10% efficiency
    }

    // Module operational status
    if !*station_module.get_is_operational() {
        efficiency = 0.0;
    }

    // Clamp efficiency between 0.0 and 2.0 (max 200% efficiency)
    Ok(efficiency.max(0.0).min(2.0))
}
