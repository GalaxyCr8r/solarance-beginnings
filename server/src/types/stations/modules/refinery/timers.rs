use super::*;

/// Calculate the production output for a refinery module based on available input resources
/// and current efficiency modifiers.
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
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item| {
            item.module_id == station_module.id && 
            item.resource_item_id == refinery.input_ore_resource_id
        })
        .ok_or("Input ore inventory not found")?;
    
    // Calculate maximum possible production based on available ore
    let max_ingots_from_ore = input_inventory.quantity as f32 / refinery.ore_to_ingot_ratio;
    
    // Calculate production capacity based on time and efficiency
    let production_capacity = refinery.base_ingots_produced_per_hour 
        * refinery.current_efficiency_modifier 
        * time_elapsed_hours;
    
    // Actual production is limited by available ore or production capacity
    let actual_ingots_produced = max_ingots_from_ore.min(production_capacity);
    
    // Calculate resource consumption and waste production
    let ore_consumed = actual_ingots_produced * refinery.ore_to_ingot_ratio;
    let waste_produced = actual_ingots_produced * refinery.waste_per_ingot_ratio;
    
    Ok(RefineryProductionResult {
        ingots_produced: actual_ingots_produced,
        ore_consumed,
        waste_produced,
        was_limited_by_ore: max_ingots_from_ore < production_capacity,
    })
}

/// Apply the calculated production results to the refinery's inventory
pub fn apply_refinery_production(
    ctx: &ReducerContext,
    refinery: &Refinery,
    production_result: &RefineryProductionResult,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    
    if production_result.ingots_produced <= 0.0 {
        return Ok(()); // No production to apply
    }
    
    let station_module = dsl.get_station_module_by_id(refinery.get_id())?;
    
    // Update input ore inventory (consume ore)
    if let Some(mut input_inventory) = dsl
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item| {
            item.module_id == station_module.id && 
            item.resource_item_id == refinery.input_ore_resource_id
        }) {
        
        let ore_to_consume = production_result.ore_consumed as u32;
        if input_inventory.quantity >= ore_to_consume {
            input_inventory.set_quantity(input_inventory.quantity - ore_to_consume);
            dsl.update_station_module_inventory_item_by_id(input_inventory)?;
        }
    }
    
    // Update output ingot inventory (add ingots)
    if let Some(mut output_inventory) = dsl
        .get_all_station_module_inventory_items()
        .into_iter()
        .find(|item| {
            item.module_id == station_module.id && 
            item.resource_item_id == refinery.output_ingot_resource_id
        }) {
        
        let ingots_to_add = production_result.ingots_produced as u32;
        output_inventory.set_quantity(output_inventory.quantity + ingots_to_add);
        dsl.update_station_module_inventory_item_by_id(output_inventory)?;
    }
    
    // Update waste inventory if waste is produced
    if let Some(waste_resource_id) = refinery.waste_resource_id {
        if production_result.waste_produced > 0.0 {
            if let Some(mut waste_inventory) = dsl
                .get_all_station_module_inventory_items()
                .into_iter()
                .find(|item| {
                    item.module_id == station_module.id && 
                    item.resource_item_id == waste_resource_id
                }) {
                
                let waste_to_add = production_result.waste_produced as u32;
                waste_inventory.set_quantity(waste_inventory.quantity + waste_to_add);
                dsl.update_station_module_inventory_item_by_id(waste_inventory)?;
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
    pub ingots_produced: f32,
    pub ore_consumed: f32,
    pub waste_produced: f32,
    pub was_limited_by_ore: bool,
}