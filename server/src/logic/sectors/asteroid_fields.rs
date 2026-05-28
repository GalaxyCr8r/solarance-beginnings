use std::f32::consts::PI;

use solarance_shared::Vec2;
use spacetimedb::rand::Rng;
use spacetimedb::ReducerContext;
use spacetimedsl::*;

use crate::{
    definitions::item_types::*,
    tables::{asteroids::*, items::*, sectors::*},
    utility::try_server_only,
};

/// Target number of asteroids a field tries to maintain, derived from its
/// sparseness (e.g. sparseness 5 → 50 asteroids).
fn target_population(asteroid_sector: &AsteroidSector) -> usize {
    (asteroid_sector.get_sparseness() * 10).into()
}

/// Spawns a single asteroid at a random position inside the field, with ore
/// type skewed by the sector's rarity. Pure spawn logic — authorization is the
/// calling reducer's responsibility. Returns `None` if the creation failed.
fn spawn_random_asteroid_in_field(
    dsl: &DSL<'_, ReducerContext>,
    asteroid_sector: &AsteroidSector,
) -> Option<Asteroid> {
    let field = *asteroid_sector.get_cluster_extent();
    let dist = match asteroid_sector.get_cluster_inner() {
        Some(inner_extent) => dsl.ctx().rng().gen_range(*inner_extent..field), // Pick a distance between inner and outer bounds.
        None => dsl.ctx().rng().gen_range(0.0..field),
    };
    let pos = Vec2::from_glam(glam::Vec2::from_angle(dsl.ctx().rng().gen_range(0.0..2.0 * PI)) * dist);

    let roll_with_disadvantage = {
        let a = dsl.ctx().rng().gen_range(0..100);
        // Only ONE of the 'rolls' should be effected by rarity, so that there's always a chance for lower rarities.
        let b = dsl.ctx()
            .rng()
            .gen_range((*asteroid_sector.get_rarity() as i32)..100);
        if a < b {
            a
        } else {
            b
        }
    };

    let item = ItemDefinitionId::new(match roll_with_disadvantage {
        0..25 => ITEM_ICE_ORE,
        25..60 => ITEM_IRON_ORE,
        60..75 => ITEM_SILICON_ORE,
        75..85 => ITEM_GOLD_ORE,
        85..90 => ITEM_VIVEIUM_ORE,
        _ => ITEM_URANIUM_ORE,
    });

    let amount = dsl.ctx().rng().gen_range(500..2000);

    create_asteroid(
        dsl,
        pos,
        asteroid_sector.get_id(),
        format!("asteroid.{}", dsl.ctx().rng().gen_range(1..=5)),
        item,
        amount,
    )
}

/// Fills a sector's asteroid field up to its target population in one pass.
/// Called at galaxy init so fields are fully stocked from t=0; the hourly
/// [`asteroid_sector_upkeep`] then only tops them back up after mining.
pub fn fill_asteroid_sector(
    dsl: &DSL<'_, ReducerContext>,
    asteroid_sector: &AsteroidSector,
) -> Result<(), String> {
    let target = target_population(asteroid_sector);
    let sector_id = asteroid_sector.get_id();

    while dsl.get_asteroids_by_current_sector_id(&sector_id).count() < target {
        if spawn_random_asteroid_in_field(dsl, asteroid_sector).is_none() {
            // Bail rather than spin forever: a persistent creation failure would
            // never advance the count and would hang the init transaction.
            return Err(format!(
                "fill_asteroid_sector: create_asteroid failed in sector {}; aborting fill toward target {}",
                sector_id.value(),
                target
            ));
        }
    }

    Ok(())
}

/// Maintains asteroid populations in a sector. When a field has dropped below
/// its target (e.g. after mining), spawns a single replacement. Runs hourly via
/// `sector_upkeep`; bulk initial population happens at init via
/// [`fill_asteroid_sector`].
pub fn asteroid_sector_upkeep(
    dsl: &DSL<'_, ReducerContext>,
    sector_id: &SectorId,
) -> Result<(), String> {
    try_server_only(dsl)?;

    let asteroid_sector = dsl.get_asteroid_sector_by_id(sector_id)?;
    if dsl.get_asteroids_by_current_sector_id(sector_id).count()
        < target_population(&asteroid_sector)
    {
        spawn_random_asteroid_in_field(dsl, &asteroid_sector).ok_or_else(|| {
            format!(
                "asteroid_sector_upkeep: create_asteroid failed in sector {}",
                sector_id.value()
            )
        })?;
    }

    Ok(())
}
