/*
    Input(INPUT: Change Velocity. 30fps)
    Movement(MOVEMENT: Translate/Rotate. 20fps)
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
    logic::{
        combat::combat_cooldown::{CreateCombatCooldownTimer, CreateCombatCooldownTimerRow},
        factions::{
            create_station_check_timer_for_faction, CreateFactionManagementTimer,
            CreateFactionManagementTimerRow, GetFactionStationCheckTimerRowOptionByFactionId,
        },
        sectors::{CreateSectorUpkeepTimer, CreateSectorUpkeepTimerRow},
        stellarobjects::{
            player_windows::{CreatePlayerWindowsTimer, CreatePlayerWindowsTimerRow},
            transforms::{CreateAllTransformsTimer, CreateAllTransformsTimerRow},
        },
    },
    tables::factions::{FactionTier, GetAllFactionRows},
};

pub fn initialize<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    // Stellar Objects
    dsl.create_all_transforms_timer(CreateAllTransformsTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_millis(1000 / 20).into()),
        current_update: 0,
    })?;
    dsl.create_player_windows_timer(CreatePlayerWindowsTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_millis(750).into()),
    })?;

    // Sectors
    dsl.create_sector_upkeep_timer(CreateSectorUpkeepTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_secs(60).into()), // Every Minute
    })?;

    // Factions
    for faction in dsl.get_all_factions() {
        // Only create timers for Galactic-tier factions (the main NPC factions)
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

    // Factions
    let _timer = dsl.create_faction_management_timer(CreateFactionManagementTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_secs(12 * 60 * 60).into()), // 12 hours
        last_management_timestamp: spacetimedb::Timestamp::UNIX_EPOCH,
    })?;

    // Combat
    // Schedule the cooldown update timer to run every 100ms
    let _cooldown_timer = dsl.create_combat_cooldown_timer(CreateCombatCooldownTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_micros(100_000).into()), // 100ms = 100,000 microseconds
    })?;

    Ok(())
}
