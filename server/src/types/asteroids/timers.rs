use std::f32::consts::PI;

use glam::Vec2;
use spacetimedb::{ rand::Rng, * };
use spacetimedsl::*;

use crate::types::{
    items::{
        definitions::{ ITEM_IRON_ORE, ITEM_SILICON_ORE, ITEM_URANIUM_ORE, ITEM_WATER },
        ItemDefinitionId,
    },
    sectors::GetAsteroidSectorRowOptionById,
};

use super::{ utility::create_asteroid, GetAsteroidRowsByCurrentSectorId };

#[dsl(plural_name = asteroid_sector_upkeep_timers)]
#[spacetimedb::table(name = asteroid_sector_upkeep_timer, scheduled(asteroid_sector_upkeep))]
pub struct AsteroidSectorUpkeepTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[index(btree)] // To find asteroids in a specific sector
    #[use_wrapper(path = crate::types::sectors::SectorId)]
    /// FK to Sector.id
    pub sector_id: u64,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx); // Waiting for DSL implementation of timers

    // Timers are created by Sector Upkeep

    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

/// Scheduled reducer that maintains asteroid populations in sectors.
/// Creates new asteroids when the count falls below the sector's sparseness threshold.
#[spacetimedb::reducer]
pub fn asteroid_sector_upkeep(
    ctx: &ReducerContext,
    timer: AsteroidSectorUpkeepTimer
) -> Result<(), String> {
    let dsl = dsl(ctx);

    let asteroid_sector = dsl.get_asteroid_sector_by_id(timer.get_sector_id())?;
    if
        dsl.get_asteroids_by_current_sector_id(timer.get_sector_id()).count() <
        (asteroid_sector.get_sparseness() * 10).into()
    {
        // 100
        let field = asteroid_sector.cluster_extent;
        let dist = match asteroid_sector.cluster_inner {
            Some(inner_extent) => ctx.rng().gen_range(inner_extent..field), // Pick a distance between inner and outer bounds.,
            None => ctx.rng().gen_range(0.0..field),
        };
        let pos = Vec2::from_angle(ctx.rng().gen_range(0.0..2.0 * PI)) * dist;

        let item = ItemDefinitionId::new(match ctx.rng().gen_range(0..100) {
            0..25 => ITEM_WATER,
            25..75 => ITEM_IRON_ORE,
            75..99 => ITEM_SILICON_ORE,
            _ => ITEM_URANIUM_ORE,
        });

        let amount = ctx.rng().gen_range(500..2000);

        create_asteroid(ctx, pos, timer.get_sector_id(), item, amount);
    }

    Ok(())
}
