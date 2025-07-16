use super::*;

/// Calculate the research production for a laboratory module
pub fn calculate_laboratory_production(
    ctx: &ReducerContext,
    laboratory: &Laboratory,
    time_elapsed_hours: f32,
) -> Result<LaboratoryProductionResult, String> {
    let dsl = dsl(ctx);

    let station_module = dsl.get_station_module_by_id(laboratory.get_id())?;

    // Find primary input inventory
    let primary_inventory = dsl
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item| {
            item.module_id == station_module.id
                && item.resource_item_id == laboratory.primary_input_resource_id
        })
        .ok_or("Primary input inventory not found")?;

    // Find secondary input inventory if required
    let secondary_inventory = if let Some(secondary_id) = laboratory.secondary_input_resource_id {
        dsl.get_all_station_module_inventory_items()
            .into_iter()
            .find(|item| {
                item.module_id == station_module.id && item.resource_item_id == secondary_id
            })
    } else {
        None
    };

    // Calculate maximum research based on available inputs
    let primary_consumption_needed = laboratory.primary_input_consumption_rate * time_elapsed_hours;
    let max_from_primary = if primary_consumption_needed > 0.0 {
        (primary_inventory.quantity as f32) / primary_consumption_needed
    } else {
        1.0
    };

    let max_from_secondary = if let (Some(inventory), Some(rate)) = (
        secondary_inventory.as_ref(),
        laboratory.secondary_input_consumption_rate,
    ) {
        let secondary_consumption_needed = rate * time_elapsed_hours;
        if secondary_consumption_needed > 0.0 {
            (inventory.quantity as f32) / secondary_consumption_needed
        } else {
            1.0
        }
    } else {
        1.0 // No secondary input required
    };

    // Research production is limited by available inputs (fraction of full production)
    let production_fraction = max_from_primary.min(max_from_secondary).min(1.0);

    // Calculate actual research production
    let research_points_produced = (laboratory.base_research_points_per_hour as f32)
        * laboratory.current_efficiency_modifier
        * time_elapsed_hours
        * production_fraction;

    // Calculate resource consumption
    let primary_consumed = primary_consumption_needed * production_fraction;
    let secondary_consumed = if let Some(rate) = laboratory.secondary_input_consumption_rate {
        rate * time_elapsed_hours * production_fraction
    } else {
        0.0
    };

    // Convert research points to research fragments (10 points = 1 fragment)
    let fragments_produced = (research_points_produced / 10.0).floor();

    Ok(LaboratoryProductionResult {
        research_points_produced,
        fragments_produced,
        primary_consumed,
        secondary_consumed,
        was_limited_by_inputs: production_fraction < 1.0,
    })
}

/// Apply the calculated production results to the laboratory's inventory
pub fn apply_laboratory_production(
    ctx: &ReducerContext,
    laboratory: &Laboratory,
    production_result: &LaboratoryProductionResult,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if production_result.fragments_produced <= 0.0 {
        return Ok(());
    }

    let station_module = dsl.get_station_module_by_id(laboratory.get_id())?;

    // Consume primary input
    if production_result.primary_consumed > 0.0 {
        if let Some(mut primary_inventory) = dsl
            .get_all_station_module_inventory_items()
            .into_iter()
            .find(|item| {
                item.module_id == station_module.id
                    && item.resource_item_id == laboratory.primary_input_resource_id
            })
        {
            let to_consume = production_result.primary_consumed as u32;
            if primary_inventory.quantity >= to_consume {
                primary_inventory.set_quantity(primary_inventory.quantity - to_consume);
                dsl.update_station_module_inventory_item_by_id(primary_inventory)?;
            }
        }
    }

    // Consume secondary input if applicable
    if let Some(secondary_id) = laboratory.secondary_input_resource_id {
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

    // Determine output fragment type based on inputs
    let output_fragment_id = determine_output_fragment_type(laboratory);

    // Add produced research fragments to output inventory
    if let Some(mut output_inventory) = dsl
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item| {
            item.module_id == station_module.id && item.resource_item_id == output_fragment_id
        })
    {
        let to_add = production_result.fragments_produced as u32;
        output_inventory.set_quantity(output_inventory.quantity + to_add);
        dsl.update_station_module_inventory_item_by_id(output_inventory)?;
    }

    Ok(())
}

/// Determine the type of research fragment produced based on laboratory inputs
fn determine_output_fragment_type(laboratory: &Laboratory) -> u32 {
    use crate::types::items::definitions::*;

    // Basic logic: if using exotic materials, produce exotic fragments
    if laboratory.primary_input_resource_id == ITEM_RESEARCH_FRAGMENT_EXOTIC {
        ITEM_RESEARCH_FRAGMENT_EXOTIC
    } else if laboratory.secondary_input_resource_id == Some(ITEM_RESEARCH_DEVICE_RARE) {
        ITEM_RESEARCH_FRAGMENT_RARE
    } else {
        ITEM_RESEARCH_FRAGMENT
    }
}

/// Calculate efficiency modifiers for laboratory production
pub fn calculate_laboratory_efficiency(
    ctx: &ReducerContext,
    laboratory: &Laboratory,
) -> Result<f32, String> {
    let dsl = dsl(ctx);

    let station_module = dsl.get_station_module_by_id(laboratory.get_id())?;
    let station = dsl.get_station_by_id(StationId::new(station_module.station_id))?;

    let mut efficiency = laboratory.current_efficiency_modifier;

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
pub struct LaboratoryProductionResult {
    pub research_points_produced: f32,
    pub fragments_produced: f32,
    pub primary_consumed: f32,
    pub secondary_consumed: f32,
    pub was_limited_by_inputs: bool,
}
