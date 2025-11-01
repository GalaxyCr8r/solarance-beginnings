use log::info;

use super::*;

/// Calculate the manufacturing production for a manufacturing module
pub fn calculate_manufacturing_production(
    ctx: &ReducerContext,
    manufacturing: &Manufacturing,
    time_elapsed_seconds: f32,
) -> Result<ManufacturingProductionResult, String> {
    let dsl = dsl(ctx);

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
    let (actual_items_completed, inputs_consumed, was_limited) = check_and_consume_inputs(
        ctx,
        &dsl,
        &station_module,
        &recipe,
        items_that_can_be_completed,
    )?;

    Ok(ManufacturingProductionResult {
        items_completed: actual_items_completed,
        progress_made: actual_items_completed as f32 + remaining_progress,
        inputs_consumed,
        was_limited_by_inputs: was_limited,
    })
}

/// Check input availability and calculate what can actually be produced
fn check_and_consume_inputs(
    _ctx: &ReducerContext,
    dsl: &DSL,
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
            .find(|item| {
                item.module_id == station_module.id
                    && item.resource_item_id == input_resource.resource_item_id
            })
        {
            let available_items = inventory.quantity / input_resource.quantity;
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
pub fn apply_manufacturing_production(
    ctx: &ReducerContext,
    manufacturing: &Manufacturing,
    production_result: &ManufacturingProductionResult,
) -> Result<(), String> {
    let dsl = dsl(ctx);

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
            .find(|item| {
                item.module_id == station_module.id && item.resource_item_id == *resource_id
            })
        {
            if inventory.quantity >= *quantity_consumed {
                inventory.set_quantity(inventory.quantity - quantity_consumed);
                dsl.update_station_module_inventory_item_by_id(inventory)?;
            }
        }
    }

    // Add produced items to output inventory
    if let Some(mut output_inventory) = dsl
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item| {
            item.module_id == station_module.id
                && item.resource_item_id == recipe.output_resource_id
        })
    {
        let items_to_add = production_result.items_completed * recipe.output_quantity;
        output_inventory.set_quantity(output_inventory.quantity + items_to_add);
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
pub fn calculate_manufacturing_efficiency(
    ctx: &ReducerContext,
    manufacturing: &Manufacturing,
) -> Result<f32, String> {
    let dsl = dsl(ctx);

    let station_module = dsl.get_station_module_by_id(manufacturing.get_id())?;
    let station = dsl.get_station_by_id(StationId::new(station_module.station_id))?;

    let mut efficiency = manufacturing.production_speed_modifier;

    // Station health affects efficiency
    if let Ok(station_status) = dsl.get_station_status_by_id(station.get_id()) {
        efficiency *= (station_status.health / 100.0).max(0.1);
    }

    // Module operational status
    if !station_module.is_operational {
        efficiency = 0.0;
    }

    Ok(efficiency.max(0.0).min(3.0)) // Manufacturing can be more efficient than other modules
}

#[derive(Clone, Debug)]
pub struct ManufacturingProductionResult {
    pub items_completed: u32,
    pub progress_made: f32,
    pub inputs_consumed: Vec<(u32, u32)>, // (resource_id, quantity)
    pub was_limited_by_inputs: bool,
}
