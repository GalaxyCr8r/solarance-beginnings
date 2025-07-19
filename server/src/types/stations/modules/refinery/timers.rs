use log::info;

use crate::types::items::GetItemDefinitionRowOptionById;

use super::*;

/// Calculate the production output for a refinery module based on available input resources
/// and current efficiency modifiers. Only produces whole units of output.
pub fn calculate_refinery_production(
    ctx: &ReducerContext,
    refinery: &Refinery,
    time_elapsed_hours: f32,
) -> Result<RefineryProductionResult, String> {
    let dsl = dsl(ctx);

    // Get the station module to access inventory
    let station_module = dsl.get_station_module_by_id(refinery.get_id())?;

    // Find input ore inventory item
    let input_inventory = dsl
        .get_station_module_inventory_items_by_module_id(refinery.get_id())
        .find(|item| item.resource_item_id == refinery.input_ore_resource_id)
        .ok_or("Input ore inventory not found")?;

    // Calculate maximum possible whole ingots based on available ore
    let max_whole_ingots_from_ore =
        (input_inventory.quantity as f32 / refinery.ore_to_ingot_ratio).floor();

    // Calculate production capacity based on time and efficiency (as whole units)
    let production_capacity_float = refinery.base_ingots_produced_per_hour
        * refinery.current_efficiency_modifier
        * time_elapsed_hours;
    let production_capacity_whole = production_capacity_float.floor();

    // Actual production is limited by available ore or production capacity (whole units only)
    let actual_ingots_produced_whole = max_whole_ingots_from_ore.min(production_capacity_whole);

    // Only proceed if we can produce at least 1 whole ingot
    if actual_ingots_produced_whole < 1.0 {
        info!(
            "Refinery module #{} cannot produce whole ingots: max from ore={:.2}, capacity={:.2}",
            station_module.id, max_whole_ingots_from_ore, production_capacity_whole
        );
        return Ok(RefineryProductionResult {
            ingots_produced: 0,
            ore_consumed: 0,
            waste_produced: 0,
            was_limited_by_ore: max_whole_ingots_from_ore < production_capacity_whole,
        });
    }
    info!(
        "Refinery module #{}: Calculating Input: '{}' amount: {}/{}, Output '{}'",
        station_module.id,
        dsl.get_item_definition_by_id(refinery.get_input_ore_resource_id())?
            .name,
        input_inventory.quantity,
        input_inventory.max_quantity,
        dsl.get_item_definition_by_id(refinery.get_output_ingot_resource_id())?
            .name
    );

    // Convert to u32 for final calculations
    let ingots_produced = actual_ingots_produced_whole as u32;

    // Calculate exact resource consumption for the whole ingots we're producing
    let ore_consumed = ((ingots_produced as f32) * refinery.ore_to_ingot_ratio).ceil() as u32;
    let waste_produced = ((ingots_produced as f32) * refinery.waste_per_ingot_ratio).ceil() as u32;

    spacetimedb::log::info!(
        "Refinery module {} producing {} whole ingots from {} ore ({} waste produced)",
        station_module.id,
        ingots_produced,
        ore_consumed,
        waste_produced
    );

    Ok(RefineryProductionResult {
        ingots_produced,
        ore_consumed,
        waste_produced,
        was_limited_by_ore: max_whole_ingots_from_ore < production_capacity_whole,
    })
}

/// Apply the calculated production results to the refinery's inventory
pub fn apply_refinery_production(
    ctx: &ReducerContext,
    refinery: &Refinery,
    production_result: &RefineryProductionResult,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if production_result.ingots_produced == 0 {
        spacetimedb::log::info!("No ingots produced, skipping production application");
        return Ok(()); // No production to apply
    }

    let station_module = dsl.get_station_module_by_id(refinery.get_id())?;

    spacetimedb::log::info!(
        "Applying refinery production: {} ingots, {} ore consumed for module {}",
        production_result.ingots_produced,
        production_result.ore_consumed,
        station_module.id
    );

    // Get all inventory items for this module
    let module_inventory_items: Vec<_> = dsl
        .get_station_module_inventory_items_by_module_id(refinery.get_id())
        .collect();

    spacetimedb::log::info!(
        "Found {} inventory items for module {}",
        module_inventory_items.len(),
        station_module.id
    );

    // Update input ore inventory (consume ore)
    let mut input_found = false;
    for mut input_inventory in module_inventory_items.iter().cloned() {
        if input_inventory.resource_item_id == refinery.input_ore_resource_id {
            input_found = true;
            let ore_to_consume = production_result.ore_consumed as u32;
            spacetimedb::log::info!(
                "Found input inventory with {} ore, consuming {}",
                input_inventory.quantity,
                ore_to_consume
            );

            if input_inventory.quantity >= ore_to_consume {
                input_inventory.set_quantity(input_inventory.quantity - ore_to_consume);
                dsl.update_station_module_inventory_item_by_id(input_inventory)?;
                spacetimedb::log::info!("Successfully consumed {} ore", ore_to_consume);
            } else {
                spacetimedb::log::info!(
                    "Not enough ore to consume: {} available, {} needed",
                    input_inventory.quantity,
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
        if output_inventory.resource_item_id == refinery.output_ingot_resource_id {
            output_found = true;
            let ingots_to_add = production_result.ingots_produced as u32;
            spacetimedb::log::info!(
                "Found output inventory with {} ingots, adding {}",
                output_inventory.quantity,
                ingots_to_add
            );

            output_inventory.set_quantity(output_inventory.quantity + ingots_to_add);
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
                if waste_inventory.resource_item_id == waste_resource_id {
                    waste_found = true;
                    let waste_to_add = production_result.waste_produced as u32;
                    waste_inventory.set_quantity(waste_inventory.quantity + waste_to_add);
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
pub fn calculate_refinery_efficiency(
    ctx: &ReducerContext,
    refinery: &Refinery,
) -> Result<f32, String> {
    let dsl = dsl(ctx);

    // Get the station module to check conditions
    let station_module = dsl.get_station_module_by_id(refinery.get_id())?;
    let station = dsl.get_station_by_id(StationId::new(station_module.station_id))?;

    let mut efficiency = 1.0;

    // Base efficiency from the refinery's current modifier
    efficiency *= refinery.current_efficiency_modifier;

    // Station health affects efficiency (get from StationStatus)
    if let Ok(station_status) = dsl.get_station_status_by_id(station.get_id()) {
        efficiency *= (station_status.health / 100.0).max(0.1); // Minimum 10% efficiency
    }

    // Module operational status
    if !station_module.is_operational {
        efficiency = 0.0;
    }

    // Clamp efficiency between 0.0 and 2.0 (max 200% efficiency)
    Ok(efficiency.max(0.0).min(2.0))
}

#[derive(Clone, Debug)]
pub struct RefineryProductionResult {
    pub ingots_produced: u32,
    pub ore_consumed: u32,
    pub waste_produced: u32,
    pub was_limited_by_ore: bool,
}
