use super::*;

/// Check if a module can buy items from the player
/// This includes trading ports, refineries (for input resources), and manufacturing modules (for recipe inputs)
pub fn module_can_buy_from_player(
    ctx: &DbConnection,
    module: &StationModule,
    item_id: u32,
) -> bool {
    // Check if it's a trading port
    if ctx
        .db()
        .trading_port_module()
        .id()
        .find(&module.id)
        .is_some()
    {
        return true;
    }

    // Check if it's a refinery that accepts this item as input
    if let Some(refinery) = ctx.db().refinery_module().id().find(&module.id) {
        if refinery.input_ore_resource_id == item_id {
            return true;
        }
    }

    // Check if it's a manufacturing module with a recipe that uses this item as input
    if let Some(manufacturing) = ctx.db().manufacturing_module().id().find(&module.id) {
        if let Some(recipe_id) = manufacturing.current_recipe_id {
            if let Some(recipe) = ctx
                .db()
                .production_recipe_definition()
                .id()
                .find(&recipe_id)
            {
                // Check if this item is in the recipe's input resources
                for input_resource in &recipe.input_resources {
                    if input_resource.resource_item_id == item_id {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Check if a module can sell items to the player
/// This includes trading ports, refineries (for output/waste resources), and manufacturing modules (for recipe outputs)
pub fn module_can_sell_to_player(ctx: &DbConnection, module: &StationModule, item_id: u32) -> bool {
    // Check if it's a trading port
    if ctx
        .db()
        .trading_port_module()
        .id()
        .find(&module.id)
        .is_some()
    {
        return true;
    }

    // Check if it's a refinery that produces this item as output or waste
    if let Some(refinery) = ctx.db().refinery_module().id().find(&module.id) {
        if refinery.output_ingot_resource_id == item_id {
            return true;
        }
        if let Some(waste_id) = refinery.waste_resource_id {
            if waste_id == item_id {
                return true;
            }
        }
    }

    // Check if it's a manufacturing module with a recipe that produces this item
    if let Some(manufacturing) = ctx.db().manufacturing_module().id().find(&module.id) {
        if let Some(recipe_id) = manufacturing.current_recipe_id {
            if let Some(recipe) = ctx
                .db()
                .production_recipe_definition()
                .id()
                .find(&recipe_id)
            {
                if recipe.output_resource_id == item_id {
                    return true;
                }
            }
        }
    }

    false
}

pub fn collect_docked_ships_per_sector(ctx: &DbConnection) -> HashMap<u64, Vec<DockedShip>> {
    let mut docked_ships_map: HashMap<u64, Vec<DockedShip>> = HashMap::new();

    for docked_ship in ctx
        .db()
        .docked_ship() // Assuming generated table handle
        .iter()
        .filter(|ship| ship.player_id == ctx.identity())
    {
        // sector_id is u64, which is a Copy, so no clone needed for the key.
        // Clone the ship itself to store in the Vec.
        docked_ships_map
            .entry(docked_ship.sector_id)
            .or_default()
            .push(docked_ship.clone());
    }
    docked_ships_map
}

pub fn prepare_docked_ships_for_system_tree(
    ctx: &DbConnection,
) -> HashMap<u32, (StarSystem, Vec<(Sector, Vec<DockedShip>)>)> {
    let docked_ships_per_sector = collect_docked_ships_per_sector(ctx);
    let mut systems_data: HashMap<u32, (StarSystem, Vec<(Sector, Vec<DockedShip>)>)> =
        HashMap::new();

    for (sector_id, ships_in_this_sector) in docked_ships_per_sector.iter() {
        // Find the sector object for the current sector_id
        if let Some(sector) = ctx.db().sector().id().find(sector_id) {
            // Assuming PK on Sector is 'id'
            // Find the star system for this sector
            if let Some(star_system) = ctx.db().star_system().id().find(&sector.system_id) {
                // Assuming PK on StarSystem is 'id'
                // Get or insert the entry for this star system
                let system_entry = systems_data
                    .entry(star_system.id) // Use system_id as the key
                    .or_insert_with(|| (star_system.clone(), Vec::new()));

                // Add the current sector and its ships to this system's list
                // We clone ships_in_this_sector because we are borrowing it from docked_ships_per_sector
                system_entry
                    .1
                    .push((sector.clone(), ships_in_this_sector.clone()));
            } else {
                info!(
                    "Warning: StarSystem with ID {} not found for sector {}",
                    sector.system_id, sector.name
                );
            }
        } else {
            info!(
                "Warning: Sector with ID {} not found, but ships are docked there.",
                sector_id
            );
        }
    }

    // Sort sectors within each system, e.g., by name or ID
    for (_system_id, (_system, sectors_with_ships)) in systems_data.iter_mut() {
        sectors_with_ships.sort_by_key(|(sector, _ships)| sector.id.clone());
        // Or by name: sectors_with_ships.sort_by(|(s1, _), (s2, _)| s1.name.cmp(&s2.name));

        // Optional: Sort ships within each sector
        for (_sector, ships) in sectors_with_ships.iter_mut() {
            ships.sort_by_key(|ship| ship.id.clone());
            // Or by name: ships.sort_by(|s1, s2| s1.name.cmp(&s2.name));
        }
    }

    // // If we want the outer map to be sorted for consistent tree display:
    // let mut sorted_systems_vec: Vec<_> = systems_data.into_iter().collect();
    // sorted_systems_vec.sort_by_key(|(system_id, (system_obj, _))| system_obj.name.clone());

    // sorted_systems_vec // We'll have to change the return value to be a vec, we'll do that elsewhere.

    systems_data
}
