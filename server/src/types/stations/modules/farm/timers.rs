use super::*;

/// Calculate the production output for a farm module based on available input resources
pub fn calculate_farm_production(
    ctx: &ReducerContext,
    farm: &Farm,
    time_elapsed_hours: f32,
) -> Result<FarmProductionResult, String> {
    let dsl = dsl(ctx);

    let station_module = dsl.get_station_module_by_id(farm.get_id())?;

    // Find primary input (compost) inventory
    let primary_inventory = dsl
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item| {
            item.module_id == station_module.id
                && item.resource_item_id == farm.primary_input_resource_id
        })
        .ok_or("Primary input inventory not found")?;

    // Find secondary input (water) inventory if required
    let secondary_inventory = if let Some(secondary_id) = farm.secondary_input_resource_id {
        dsl.get_all_station_module_inventory_items()
            .into_iter()
            .find(|item| {
                item.module_id == station_module.id && item.resource_item_id == secondary_id
            })
    } else {
        None
    };

    // Calculate maximum production based on available inputs
    let max_from_primary = primary_inventory.quantity as f32 / farm.primary_input_conversion_rate;

    let max_from_secondary = if let (Some(inventory), Some(rate)) = (
        secondary_inventory.as_ref(),
        farm.secondary_input_conversion_rate,
    ) {
        inventory.quantity as f32 / rate
    } else {
        f32::MAX // No secondary input required
    };

    let max_from_inputs = max_from_primary.min(max_from_secondary);

    // Calculate production capacity based on time and efficiency
    let production_capacity =
        farm.base_production_units_per_hour * farm.current_efficiency_modifier * time_elapsed_hours;

    // Calculate maximum whole units that can be produced from inputs
    let max_whole_from_inputs = max_from_inputs.floor();

    // Calculate production capacity as whole units
    let production_capacity_whole = production_capacity.floor();

    // Actual production is limited by inputs or capacity (whole units only)
    let actual_food_produced_whole = max_whole_from_inputs.min(production_capacity_whole);

    // Only proceed if we can produce at least 1 whole unit of food
    if actual_food_produced_whole < 1.0 {
        spacetimedb::log::info!(
            "Farm module {} cannot produce whole food units: max from inputs={:.2}, capacity={:.2}",
            station_module.id,
            max_whole_from_inputs,
            production_capacity_whole
        );
        return Ok(FarmProductionResult {
            food_produced: 0.0,
            primary_consumed: 0.0,
            secondary_consumed: 0.0,
            was_limited_by_inputs: max_whole_from_inputs < production_capacity_whole,
        });
    }

    // Calculate exact resource consumption for the whole units we're producing
    let primary_consumed = actual_food_produced_whole * farm.primary_input_conversion_rate;
    let secondary_consumed = if let Some(rate) = farm.secondary_input_conversion_rate {
        actual_food_produced_whole * rate
    } else {
        0.0
    };

    spacetimedb::log::info!(
        "Farm module {} producing {} whole food units",
        station_module.id,
        actual_food_produced_whole
    );

    Ok(FarmProductionResult {
        food_produced: actual_food_produced_whole,
        primary_consumed,
        secondary_consumed,
        was_limited_by_inputs: max_whole_from_inputs < production_capacity_whole,
    })
}

/// Apply the calculated production results to the farm's inventory
pub fn apply_farm_production(
    ctx: &ReducerContext,
    farm: &Farm,
    production_result: &FarmProductionResult,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if production_result.food_produced <= 0.0 {
        return Ok(());
    }

    let station_module = dsl.get_station_module_by_id(farm.get_id())?;

    // Consume primary input
    if let Some(mut primary_inventory) = dsl
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item| {
            item.module_id == station_module.id
                && item.resource_item_id == farm.primary_input_resource_id
        })
    {
        let to_consume = production_result.primary_consumed as u32;
        if primary_inventory.quantity >= to_consume {
            primary_inventory.set_quantity(primary_inventory.quantity - to_consume);
            dsl.update_station_module_inventory_item_by_id(primary_inventory)?;
        }
    }

    // Consume secondary input if applicable
    if let Some(secondary_id) = farm.secondary_input_resource_id {
        if production_result.secondary_consumed > 0.0 {
            if let Some(mut secondary_inventory) = dsl
                .get_all_station_module_inventory_items()
                .into_iter()
                .find(|item| {
                    item.module_id == station_module.id && item.resource_item_id == secondary_id
                })
            {
                let to_consume = production_result.secondary_consumed as u32;
                if secondary_inventory.quantity >= to_consume {
                    secondary_inventory.set_quantity(secondary_inventory.quantity - to_consume);
                    dsl.update_station_module_inventory_item_by_id(secondary_inventory)?;
                }
            }
        }
    }

    // Add produced food to output inventory
    if let Some(mut output_inventory) = dsl
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item| {
            item.module_id == station_module.id && item.resource_item_id == farm.output_resource_id
        })
    {
        let to_add = production_result.food_produced as u32;
        output_inventory.set_quantity(output_inventory.quantity + to_add);
        dsl.update_station_module_inventory_item_by_id(output_inventory)?;
    }

    Ok(())
}

/// Calculate efficiency modifiers for farm production
pub fn calculate_farm_efficiency(ctx: &ReducerContext, farm: &Farm) -> Result<f32, String> {
    let dsl = dsl(ctx);

    let station_module = dsl.get_station_module_by_id(farm.get_id())?;
    let station = dsl.get_station_by_id(StationId::new(station_module.station_id))?;

    let mut efficiency = farm.current_efficiency_modifier;

    // Station health affects efficiency
    if let Ok(station_status) = dsl.get_station_status_by_id(station.get_id()) {
        efficiency *= (station_status.health / 100.0).max(0.1);
    }

    // Module operational status
    if !station_module.is_operational {
        efficiency = 0.0;
    }

    Ok(efficiency.max(0.0).min(2.0))
}

#[derive(Clone, Debug)]
pub struct FarmProductionResult {
    pub food_produced: f32,
    pub primary_consumed: f32,
    pub secondary_consumed: f32,
    pub was_limited_by_inputs: bool,
}
