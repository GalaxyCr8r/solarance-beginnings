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

pub fn initialize_timers(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx); // Waiting for DSL implementation of timers

    dsl.create_sector_upkeep_timer(spacetimedb::ScheduleAt::Interval(
        Duration::from_secs(60).into(), // Every Minute
    ))?;

    Ok(())
}
