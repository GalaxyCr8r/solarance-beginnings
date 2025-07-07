use super::*;

pub fn display_ship_on_tree(ctx: &DbConnection, state: &mut State, ui: &mut Ui, ship: &DockedShip) {
    let ship_type = ctx.db().ship_type_definition().id().find(&ship.shiptype_id);

    let mut select_enabled = true;
    if state.selected_ship.clone().is_some_and(|selected| selected.id == ship.id) {
        select_enabled = false;
    }

    ui.horizontal(|ui| {
        // You can make ships collapsible too, or just list them
        ui.label(
            format!(
                "    - Ship: {} (ID: {})",
                if ship_type.is_some() {
                    ship_type.unwrap().name
                } else {
                    "Unknown Ship Type".to_string()
                },
                ship.id
            )
        );

        // Buttons on the right
        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            // Add buttons in reverse order of appearance (rightmost first)
            if ui.button("Undock").clicked() {
                println!("Undock clicked for ship ID: {}", ship.id);
                state.selected_ship = None;
                let _ = ctx.reducers().undock_ship(ShipGlobalId { value: ship.id });
                // TODO Add a system message to alert the player if it failed.
            }
            if select_enabled && ui.button("Select").clicked() {
                println!("Select clicked for ship ID: {}", ship.id);
                // Handle selection, e.g., update some state
                // *selected_ship_id = Some(ship.id);
                state.selected_ship = Some(ship.clone());
            } else if !select_enabled {
                ui.add_enabled(select_enabled, egui::Button::new("Select"));
            }
        });
    });
}

pub fn collect_docked_ships_per_sector(ctx: &DbConnection) -> HashMap<u64, Vec<DockedShip>> {
    let mut docked_ships_map: HashMap<u64, Vec<DockedShip>> = HashMap::new();

    for docked_ship in ctx
        .db()
        .docked_ship() // Assuming generated table handle
        .iter()
        .filter(|ship| ship.player_id == ctx.identity()) {
        // sector_id is u64, which is a Copy, so no clone needed for the key.
        // Clone the ship itself to store in the Vec.
        docked_ships_map.entry(docked_ship.sector_id).or_default().push(docked_ship.clone());
    }
    docked_ships_map
}

pub fn prepare_docked_ships_for_system_tree(
    ctx: &DbConnection
) -> HashMap<u32, (StarSystem, Vec<(Sector, Vec<DockedShip>)>)> {
    let docked_ships_per_sector = collect_docked_ships_per_sector(ctx);
    let mut systems_data: HashMap<
        u32,
        (StarSystem, Vec<(Sector, Vec<DockedShip>)>)
    > = HashMap::new();

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
                system_entry.1.push((sector.clone(), ships_in_this_sector.clone()));
            } else {
                info!(
                    "Warning: StarSystem with ID {} not found for sector {}",
                    sector.system_id,
                    sector.name
                );
            }
        } else {
            info!("Warning: Sector with ID {} not found, but ships are docked there.", sector_id);
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
