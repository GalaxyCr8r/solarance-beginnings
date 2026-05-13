/*
Rough draft of timings/tiers:

    Input(INPUT: Change Velocity. event-driven, no tick)
    Movement(MOVEMENT: client-side prediction via solarance_shared::predict_movement)
    Interactions(INTERACTIONS: Mine Asteroid, Use Jumpgate, etc. 5fps)
    Combat(COMBAT: Shield updates, Spawn missiles, etc. 2fps)
    Economy(ECONOMY: Update station modules, factories produce things, etc. 1 per minute)
    Sector(SECTOR: Respawn asteroids, enemies, etc. every 30 minutes)
    Faction(FACTION: Send fleets. Build fleets. Begin new stations. etc. every hour)
    Waves(WAVES: Gameplay ebbs and flows, Vancellan/Raider invasions, etc. days)
*/

use std::time::Duration;

use log::info;
use spacetimedsl::*;

use crate::{
    logic::{cargo_crates::*, combat::combat_cooldown::*, factions::*, sectors::*},
    tables::factions::*,
};

/// Called only once when the module is first initialized.
pub fn initialize<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    // Sectors
    dsl.create_sector_upkeep_timer(CreateSectorUpkeepTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_hours(12).into()), // Every Minute
    })?;

    // Factions
    faction_timers(dsl)?;

    // Combat
    // Schedule the cooldown update timer to run every 100ms
    let _cooldown_timer = dsl.create_combat_cooldown_timer(CreateCombatCooldownTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_micros(100_000).into()), // 100ms = 100,000 microseconds
    })?;

    // Cargo crate despawn sweeper (every 30 minutes). Replaces the per-crate
    // despawn check that used to ride on the 20 Hz transform tick.
    dsl.create_cargo_crate_despawn_sweeper_timer(CreateCargoCrateDespawnSweeperTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_secs(30 * 60).into()),
    })?;

    Ok(())
}

fn faction_timers<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    for faction in dsl.get_all_factions() {
        // Only create timers for Galactic-tier factions
        // Factionless is excluded as it's a player-only neutral faction
        if *faction.get_tier() == FactionTier::Galactic {
            // Check if timer already exists
            if dsl
                .get_faction_station_check_timer_by_faction_id(faction.get_id())
                .is_err()
            {
                create_station_check_timer_for_faction(dsl, &faction.get_id())?;
                info!(
                    "Created station check timer for faction: {}",
                    faction.get_name()
                );
            }
        }
    }
    let _timer = dsl.create_faction_management_timer(CreateFactionManagementTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_hours(12).into()), // 12 hours
        last_management_timestamp: spacetimedb::Timestamp::UNIX_EPOCH,
    })?;
    Ok(())
}
