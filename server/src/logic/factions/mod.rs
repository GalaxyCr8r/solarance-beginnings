use std::time::Duration;

use log::info;
use spacetimedb::{ReducerContext, Timestamp};
use crate::spacetimedsl::prelude::*;

use crate::{
    tables::{factions::*, stations::*},
    utility::try_server_only,
};

///////////////////////////////////////////////////////////////////////
/// Timers

/// Timer for periodic faction station checks - runs every 4 hours
#[spacetimedsl::dsl(plural_name = faction_station_check_timers, method(update = true))]
#[spacetimedb::table(
    accessor = faction_station_check_timer,
    scheduled(faction_station_check_timer_reducer)
)]
pub struct FactionStationCheckTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[unique]
    #[use_wrapper(FactionId)]
    /// FK to Faction
    faction_id: u32,

    /// Last check timestamp for tracking
    pub last_check_timestamp: Timestamp,
}

/// Overall faction management timer - runs every 12 hours to maintain faction timers
#[spacetimedsl::dsl(plural_name = faction_management_timers, method(update = true))]
#[spacetimedb::table(accessor = faction_management_timer, scheduled(faction_management_timer_reducer))]
pub struct FactionManagementTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    /// Last management check timestamp
    pub last_management_timestamp: Timestamp,
}

///////////////////////////////////////////////////////////////
/// Timer Reducers
///

/// Scheduled reducer that checks faction stations every 4 hours
/// Monitors station health, resource levels, and operational status
#[spacetimedb::reducer]
pub fn faction_station_check_timer_reducer(
    ctx: &ReducerContext,
    mut timer: FactionStationCheckTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    info!(
        "Running station check for faction #{}",
        timer.get_faction_id()
    );

    // Get all stations belonging to this faction
    let faction_stations: Vec<Station> = dsl
        .get_all_stations()
        .filter(|station| station.get_owner_faction_id().value() == timer.get_faction_id().value())
        .collect();

    info!(
        "Faction #{} has {} stations to check",
        timer.get_faction_id(),
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
    timer.set_last_check_timestamp(dsl.ctx().timestamp);
    dsl.update_faction_station_check_timer_by_id(timer)?;

    Ok(())
}

/// Scheduled reducer that manages faction timers every 12 hours
/// Ensures all factions have their required timers and removes orphaned timers
#[spacetimedb::reducer]
pub fn faction_management_timer_reducer(
    ctx: &ReducerContext,
    mut timer: FactionManagementTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    info!("Running faction management timer - checking all faction timers");

    // Get all existing factions
    let all_factions: Vec<Faction> = dsl.get_all_factions().collect();
    let faction_ids: std::collections::HashSet<u32> =
        all_factions.iter().map(|f| f.get_id().value()).collect();

    info!("Found {} active factions", all_factions.len());

    // 1. Ensure all Galactic-tier factions have station check timers
    // OBE

    // 2. Remove orphaned station check timers for factions that no longer exist
    let orphaned_station_timers: Vec<FactionStationCheckTimer> = dsl
        .get_all_faction_station_check_timers()
        .filter(|timer| !faction_ids.contains(&timer.get_faction_id().value()))
        .collect();

    for orphaned_timer in orphaned_station_timers {
        info!(
            "Removing orphaned station check timer for deleted faction #{}",
            orphaned_timer.get_faction_id()
        );
        dsl.delete_faction_station_check_timer_by_id(&orphaned_timer)?;
    }

    // Update the last management timestamp
    timer.set_last_management_timestamp(dsl.ctx().timestamp);
    dsl.update_faction_management_timer_by_id(timer)?;

    info!("Faction management timer completed successfully");
    Ok(())
}

///////////////////////////////////////////////////
/// Logic utilities

/// Creates the faction management timer if it doesn't already exist
pub fn create_faction_management_timer_if_needed<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
) -> Result<(), String> {
    // Check if management timer already exists
    if dsl.get_all_faction_management_timers().next().is_none() {
        let timer = dsl.create_faction_management_timer(CreateFactionManagementTimer {
            scheduled_at: spacetimedb::ScheduleAt::Interval(
                Duration::from_secs(12 * 60 * 60).into(),
            ), // 12 hours
            last_management_timestamp: spacetimedb::Timestamp::UNIX_EPOCH,
        })?;

        info!(
            "Created faction management timer #{} - will run every 12 hours",
            timer.get_id().value()
        );
    }

    Ok(())
}

/// Creates a station check timer for a faction - runs every 4 hours
pub fn create_station_check_timer_for_faction<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_id: &FactionId,
) -> Result<FactionStationCheckTimer, String> {
    let timer = dsl.create_faction_station_check_timer(CreateFactionStationCheckTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_secs(4 * 60 * 60).into()), // 4 hours
        faction_id: faction_id.clone(),
        last_check_timestamp: dsl.ctx().timestamp()?,
    })?;

    info!(
        "Created station check timer for faction #{}",
        faction_id.value()
    );
    Ok(timer)
}

/// Updates faction standing between two factions
/// Creates a new standing if one doesn't exist
pub fn update_faction_standing<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_one_id: &FactionId,
    faction_two_id: &FactionId,
    reputation_change: i32,
) -> Result<(), String> {
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
                get_faction_name(dsl, faction_one_id),
                get_faction_name(dsl, faction_two_id),
                new_reputation - reputation_change,
                clamped_reputation
            );
            return Ok(());
        }
    }

    // If no standing exists, create a new one
    let initial_reputation = reputation_change.max(-100).min(100);
    dsl.create_faction_standing(CreateFactionStanding {
        faction_one_id: faction_one_id.clone(),
        faction_two_id: faction_two_id.clone(),
        reputation_score: initial_reputation,
    })?;

    info!(
        "Created new standing between {} and {}: {}",
        get_faction_name(dsl, faction_one_id),
        get_faction_name(dsl, faction_two_id),
        initial_reputation
    );

    Ok(())
}
