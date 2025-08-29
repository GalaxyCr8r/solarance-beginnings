use std::time::Duration;

use log::info;
use spacetimedb::*;
use spacetimedsl::{dsl, Wrapper};

use crate::types::{common::utility::try_server_only, sectors::SectorId, ships::*, stations::*};

use super::*;

/// Timer for periodic faction station checks - runs every 4 hours
#[dsl(plural_name = faction_station_check_timers)]
#[spacetimedb::table(name = faction_station_check_timer, scheduled(faction_station_check_timer_reducer))]
pub struct FactionStationCheckTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[unique]
    #[use_wrapper(path = FactionId)]
    /// FK to Faction
    pub faction_id: u32,

    /// Last check timestamp for tracking
    pub last_check_timestamp: Timestamp,
}

/// Overall faction management timer - runs every 12 hours to maintain faction timers
#[dsl(plural_name = faction_management_timers)]
#[spacetimedb::table(name = faction_management_timer, scheduled(faction_management_timer_reducer))]
pub struct FactionManagementTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    /// Last management check timestamp
    pub last_management_timestamp: Timestamp,
}

/// Timer for faction ship destruction reactions - runs after a delay when ships are destroyed
#[dsl(plural_name = faction_ship_reaction_timers)]
#[spacetimedb::table(name = faction_ship_reaction_timer, scheduled(faction_ship_reaction_timer_reducer))]
pub struct FactionShipReactionTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[use_wrapper(path = FactionId)]
    /// FK to Faction - the faction that lost the ship
    pub faction_id: u32,

    #[use_wrapper(path = FactionId)]
    /// FK to Faction - the faction that destroyed the ship (if known)
    pub aggressor_faction_id: Option<u32>,

    #[use_wrapper(path = ShipTypeDefinitionId)]
    /// The type of ship that was destroyed
    pub destroyed_ship_type_id: u32,

    /// Location where the ship was destroyed (sector ID)
    #[use_wrapper(path = SectorId)]
    pub destruction_sector_id: u64,

    /// Timestamp when the ship was destroyed
    pub destruction_timestamp: Timestamp,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    // Create the overall faction management timer if it doesn't exist
    create_faction_management_timer_if_needed(ctx)?;
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

/// Creates the faction management timer if it doesn't already exist
pub fn create_faction_management_timer_if_needed(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Check if management timer already exists
    if dsl.get_all_faction_management_timers().next().is_none() {
        let timer = dsl.create_faction_management_timer(
            spacetimedb::ScheduleAt::Interval(Duration::from_secs(12 * 60 * 60).into()), // 12 hours
            ctx.timestamp,
        )?;

        info!(
            "Created faction management timer #{} - will run every 12 hours",
            timer.get_id().value()
        );
    }

    Ok(())
}

/// Creates a station check timer for a faction - runs every 4 hours
pub fn create_station_check_timer_for_faction(
    ctx: &ReducerContext,
    faction_id: &FactionId,
) -> Result<FactionStationCheckTimer, String> {
    let dsl = dsl(ctx);

    let timer = dsl.create_faction_station_check_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_secs(4 * 60 * 60).into()), // 4 hours
        faction_id,
        ctx.timestamp,
    )?;

    info!(
        "Created station check timer for faction #{}",
        faction_id.value()
    );
    Ok(timer)
}

/// Creates a ship reaction timer for when a faction ship is destroyed
pub fn create_ship_reaction_timer_for_faction(
    ctx: &ReducerContext,
    faction_id: &FactionId,
    aggressor_faction_id: Option<&FactionId>,
    destroyed_ship_type_id: &ShipTypeDefinitionId,
    destruction_sector_id: &SectorId,
) -> Result<FactionShipReactionTimer, String> {
    let dsl = dsl(ctx);

    let timer = dsl.create_faction_ship_reaction_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_secs(30).into()), // 30 second delay
        faction_id,
        aggressor_faction_id.cloned(),
        destroyed_ship_type_id,
        destruction_sector_id,
        ctx.timestamp,
    )?;

    info!(
        "Created ship reaction timer for faction #{} - ship type #{} destroyed in sector #{}",
        faction_id.value(),
        destroyed_ship_type_id.value(),
        destruction_sector_id.value()
    );
    Ok(timer)
}

//////////////////////////////////////////////////////////////
// Timer Reducers
//////////////////////////////////////////////////////////////

/// Scheduled reducer that checks faction stations every 4 hours
/// Monitors station health, resource levels, and operational status
#[spacetimedb::reducer]
pub fn faction_station_check_timer_reducer(
    ctx: &ReducerContext,
    mut timer: FactionStationCheckTimer,
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    info!("Running station check for faction #{}", timer.faction_id);

    // Get all stations belonging to this faction
    let faction_stations: Vec<Station> = dsl
        .get_all_stations()
        .filter(|station| station.get_owner_faction_id().value() == timer.faction_id)
        .collect();

    info!(
        "Faction #{} has {} stations to check",
        timer.faction_id,
        faction_stations.len()
    );

    // TODO: Implement station checking logic
    // - Check station health/integrity
    // - Monitor resource stockpiles
    // - Verify defensive capabilities
    // - Plan expansion or reinforcement
    // - React to threats in the area

    for station in faction_stations {
        info!(
            "Checking station #{}: {} in sector #{}",
            station.get_id().value(),
            station.get_name(),
            station.get_sector_id().value()
        );

        // Placeholder for station analysis logic
        // This is where faction AI would evaluate:
        // - Station's strategic importance
        // - Resource production efficiency
        // - Defensive needs
        // - Trade route security
    }

    // Update the last check timestamp
    timer.set_last_check_timestamp(ctx.timestamp);
    dsl.update_faction_station_check_timer_by_id(timer)?;

    Ok(())
}

/// Scheduled reducer that handles faction reactions to ship destruction
/// Processes diplomatic consequences and potential retaliation
#[spacetimedb::reducer]
pub fn faction_ship_reaction_timer_reducer(
    ctx: &ReducerContext,
    timer: FactionShipReactionTimer,
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    // Remove the timer as it's a one-time reaction
    dsl.delete_faction_ship_reaction_timer_by_id(&timer)?;

    info!(
        "Processing ship destruction reaction for faction #{} - ship type #{} destroyed by faction #{:?}",
        timer.faction_id,
        timer.destroyed_ship_type_id,
        timer.aggressor_faction_id
    );

    // TODO: Implement faction reaction logic
    // - Adjust faction standings based on the aggressor
    // - Plan retaliatory actions if appropriate
    // - Increase security in the affected sector
    // - Send reinforcements if needed
    // - Update faction AI priorities

    if let Some(aggressor_id) = timer.aggressor_faction_id {
        info!(
            "Faction #{} will remember that faction #{} destroyed their ship",
            timer.faction_id, aggressor_id
        );

        // Placeholder for diplomatic consequences
        // This could involve:
        // - Reducing faction standing
        // - Marking sectors as hostile
        // - Planning counter-attacks
        // - Alerting allied factions
    } else {
        info!(
            "Faction #{} lost a ship to unknown causes (pirates, accidents, etc.)",
            timer.faction_id
        );

        // Placeholder for general security responses
        // This could involve:
        // - Increasing patrols in the area
        // - Investigating the cause
        // - Improving defenses
    }

    Ok(())
}

/// Scheduled reducer that manages faction timers every 12 hours
/// Ensures all factions have their required timers and removes orphaned timers
#[spacetimedb::reducer]
pub fn faction_management_timer_reducer(
    ctx: &ReducerContext,
    mut timer: FactionManagementTimer,
) -> Result<(), String> {
    try_server_only(ctx)?;
    let dsl = dsl(ctx);

    info!("Running faction management timer - checking all faction timers");

    // Get all existing factions
    let all_factions: Vec<Faction> = dsl.get_all_factions().collect();
    let faction_ids: std::collections::HashSet<u32> =
        all_factions.iter().map(|f| f.get_id().value()).collect();

    info!("Found {} active factions", all_factions.len());

    // 1. Ensure all Galactic-tier factions have station check timers
    for faction in &all_factions {
        if *faction.get_tier() == FactionTier::Galactic {
            // Check if station check timer exists for this faction
            if dsl
                .get_faction_station_check_timer_by_faction_id(faction.get_id())
                .is_err()
            {
                create_station_check_timer_for_faction(ctx, &faction.get_id())?;
                info!(
                    "Created missing station check timer for faction: {}",
                    faction.get_name()
                );
            }
        }
    }

    // 2. Remove orphaned station check timers for factions that no longer exist
    let orphaned_station_timers: Vec<FactionStationCheckTimer> = dsl
        .get_all_faction_station_check_timers()
        .filter(|timer| !faction_ids.contains(&timer.faction_id))
        .collect();

    for orphaned_timer in orphaned_station_timers {
        info!(
            "Removing orphaned station check timer for deleted faction #{}",
            orphaned_timer.faction_id
        );
        dsl.delete_faction_station_check_timer_by_id(&orphaned_timer)?;
    }

    // 3. Remove orphaned ship reaction timers for factions that no longer exist
    let orphaned_reaction_timers: Vec<FactionShipReactionTimer> = dsl
        .get_all_faction_ship_reaction_timers()
        .filter(|timer| !faction_ids.contains(&timer.faction_id))
        .collect();

    for orphaned_timer in orphaned_reaction_timers {
        info!(
            "Removing orphaned ship reaction timer for deleted faction #{}",
            orphaned_timer.faction_id
        );
        dsl.delete_faction_ship_reaction_timer_by_id(&orphaned_timer)?;
    }

    // Update the last management timestamp
    timer.set_last_management_timestamp(ctx.timestamp);
    dsl.update_faction_management_timer_by_id(timer)?;

    info!("Faction management timer completed successfully");
    Ok(())
}
