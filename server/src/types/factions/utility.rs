use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::types::{common::utility::try_server_only, sectors::*, ships::*, stations::*};

use super::{timers::*, *};

/// Gets the faction name for display purposes, with fallback for unknown factions
pub fn get_faction_name(ctx: &ReducerContext, faction_id: &FactionId) -> String {
    let dsl = dsl(ctx);

    if let Ok(faction) = dsl.get_faction_by_id(faction_id) {
        faction.get_name().clone()
    } else {
        format!("Unknown Faction #{}", faction_id.value())
    }
}

/// Checks if a faction exists and is joinable by players
pub fn is_faction_joinable(ctx: &ReducerContext, faction_id: &FactionId) -> bool {
    let dsl = dsl(ctx);

    if let Ok(faction) = dsl.get_faction_by_id(faction_id) {
        *faction.get_joinable()
    } else {
        false
    }
}

/// Gets the faction tier for a given faction
pub fn get_faction_tier(ctx: &ReducerContext, faction_id: &FactionId) -> Option<FactionTier> {
    let dsl = dsl(ctx);

    if let Ok(faction) = dsl.get_faction_by_id(faction_id) {
        Some(faction.get_tier().clone())
    } else {
        None
    }
}

/// Gets the reputation score between two factions
/// Returns 0 (neutral) if no standing exists
pub fn get_faction_reputation(
    ctx: &ReducerContext,
    faction_one_id: &FactionId,
    faction_two_id: &FactionId,
) -> i32 {
    let dsl = dsl(ctx);

    // Look for existing standing between the factions
    for standing in dsl.get_all_faction_standings() {
        if standing.get_faction_one_id().value() == faction_one_id.value()
            && standing.get_faction_two_id().value() == faction_two_id.value()
        {
            return *standing.get_reputation_score();
        }
    }

    // Default to neutral if no standing exists
    0
}

/// Checks if two factions are hostile to each other (reputation < -50)
pub fn are_factions_hostile(
    ctx: &ReducerContext,
    faction_one_id: &FactionId,
    faction_two_id: &FactionId,
) -> bool {
    get_faction_reputation(ctx, faction_one_id, faction_two_id) < -50
}

/// Checks if two factions are allied (reputation > 50)
pub fn are_factions_allied(
    ctx: &ReducerContext,
    faction_one_id: &FactionId,
    faction_two_id: &FactionId,
) -> bool {
    get_faction_reputation(ctx, faction_one_id, faction_two_id) > 50
}

/// Gets all stations belonging to a specific faction
pub fn get_faction_stations(ctx: &ReducerContext, faction_id: &FactionId) -> Vec<Station> {
    let dsl = dsl(ctx);

    dsl.get_all_stations()
        .filter(|station| station.get_owner_faction_id().value() == faction_id.value())
        .collect()
}

/// Gets all ships belonging to a specific faction
pub fn get_faction_ships(ctx: &ReducerContext, faction_id: &FactionId) -> Vec<Ship> {
    let dsl = dsl(ctx);

    dsl.get_all_ships()
        .filter(|ship| ship.get_faction_id().value() == faction_id.value())
        .collect()
}

/// Counts the total number of stations controlled by a faction
pub fn count_faction_stations(ctx: &ReducerContext, faction_id: &FactionId) -> usize {
    get_faction_stations(ctx, faction_id).len()
}

/// Counts the total number of ships controlled by a faction
pub fn count_faction_ships(ctx: &ReducerContext, faction_id: &FactionId) -> usize {
    get_faction_ships(ctx, faction_id).len()
}

/// Gets all sectors where a faction has presence (stations or significant ship activity)
pub fn get_faction_controlled_sectors(
    ctx: &ReducerContext,
    faction_id: &FactionId,
) -> Vec<SectorId> {
    let dsl = dsl(ctx);
    let mut controlled_sectors = Vec::new();

    // Add sectors with faction stations
    for station in get_faction_stations(ctx, faction_id) {
        let sector_id = station.get_sector_id();
        if !controlled_sectors.contains(&sector_id) {
            controlled_sectors.push(sector_id.clone());
        }
    }

    // TODO: Could also add sectors with significant ship presence
    // TODO: Could add sectors with faction-controlled jump gates

    controlled_sectors
}

/// Initializes faction timers for all existing factions
/// Should be called during server startup
pub fn initialize_faction_timers(ctx: &ReducerContext) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    info!("Initializing faction timers...");

    for faction in dsl.get_all_factions() {
        // Only create timers for Galactic-tier factions (the main NPC factions)
        // Factionless is excluded as it's a player-only neutral faction
        if *faction.get_tier() == FactionTier::Galactic {
            // Check if timer already exists
            if dsl
                .get_faction_station_check_timer_by_faction_id(faction.get_id())
                .is_err()
            {
                create_station_check_timer_for_faction(ctx, &faction.get_id())?;
                info!(
                    "Created station check timer for faction: {}",
                    faction.get_name()
                );
            }
        }
    }

    info!("Faction timer initialization complete");
    Ok(())
}

/// Handles when a faction ship is destroyed - creates reaction timer
pub fn handle_faction_ship_destroyed(
    ctx: &ReducerContext,
    destroyed_ship: &Ship,
    aggressor_faction_id: Option<&FactionId>,
    destruction_sector_id: &SectorId,
) -> Result<(), String> {
    try_server_only(ctx)?;

    let faction_id = destroyed_ship.get_faction_id();

    // Only react if it's a faction ship (not player-owned)
    if destroyed_ship.get_player_id().value() == Identity::ONE {
        create_ship_reaction_timer_for_faction(
            ctx,
            &faction_id,
            aggressor_faction_id,
            &destroyed_ship.get_shiptype_id(),
            destruction_sector_id,
        )?;

        info!(
            "Faction {} will react to the destruction of their {} in sector #{}",
            get_faction_name(ctx, &faction_id),
            destroyed_ship.get_shiptype_id().value(),
            destruction_sector_id.value()
        );
    }

    Ok(())
}

/// Updates faction standing between two factions
/// Creates a new standing if one doesn't exist
pub fn update_faction_standing(
    ctx: &ReducerContext,
    faction_one_id: &FactionId,
    faction_two_id: &FactionId,
    reputation_change: i32,
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    // Look for existing standing
    for mut standing in dsl.get_all_faction_standings() {
        if standing.get_faction_one_id().value() == faction_one_id.value()
            && standing.get_faction_two_id().value() == faction_two_id.value()
        {
            let new_reputation = standing.get_reputation_score() + reputation_change;
            // Clamp reputation between -100 and 100
            let clamped_reputation = new_reputation.max(-100).min(100);

            standing.set_reputation_score(clamped_reputation);
            dsl.update_faction_standing_by_id(standing)?;

            info!(
                "Updated standing between {} and {}: {} -> {}",
                get_faction_name(ctx, faction_one_id),
                get_faction_name(ctx, faction_two_id),
                new_reputation - reputation_change,
                clamped_reputation
            );
            return Ok(());
        }
    }

    // If no standing exists, create a new one
    let initial_reputation = reputation_change.max(-100).min(100);
    dsl.create_faction_standing(faction_one_id, faction_two_id, initial_reputation)?;

    info!(
        "Created new standing between {} and {}: {}",
        get_faction_name(ctx, faction_one_id),
        get_faction_name(ctx, faction_two_id),
        initial_reputation
    );

    Ok(())
}

/// Gets a list of all joinable factions for player selection
pub fn get_joinable_factions(ctx: &ReducerContext) -> Vec<Faction> {
    let dsl = dsl(ctx);

    dsl.get_all_factions()
        .filter(|faction| *faction.get_joinable())
        .collect()
}

/// Gets faction capital station if it exists
pub fn get_faction_capital_station(
    ctx: &ReducerContext,
    faction_id: &FactionId,
) -> Option<Station> {
    let dsl = dsl(ctx);

    if let Ok(faction) = dsl.get_faction_by_id(faction_id) {
        if let Some(capital_station_id) = faction.get_capital_station_id() {
            if let Ok(station) = dsl.get_station_by_id(&StationId::new(*capital_station_id)) {
                return Some(station);
            }
        }
    }

    None
}
