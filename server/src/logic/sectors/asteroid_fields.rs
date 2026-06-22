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

/// Picks an ore `item_id` from `weights` given `roll` in `0..sum(weights)`.
/// Returns `None` when `weights` is empty or every weight is zero, signalling
/// the caller to fall back to the global distribution. Pure (no RNG) so it can
/// be unit-tested directly.
fn pick_weighted_ore(weights: &[OreWeight], roll: u32) -> Option<u32> {
    let mut remaining = roll;
    for w in weights {
        let weight = w.weight as u32;
        if weight == 0 {
            continue;
        }
        if remaining < weight {
            return Some(w.item_id);
        }
        remaining -= weight;
    }
    None
}

/// Rolls the ore type for a new asteroid. Uses the sector's explicit
/// `ore_weights` composition when set; otherwise falls back to the global
/// rarity-skewed distribution (the historical behaviour).
fn roll_ore_item(dsl: &DSL<'_, ReducerContext>, asteroid_sector: &AsteroidSector) -> u32 {
    let weights = asteroid_sector.get_ore_weights();
    let total: u32 = weights.iter().map(|w| w.weight as u32).sum();
    if total > 0 {
        let roll = dsl.ctx().rng().gen_range(0..total);
        if let Some(item_id) = pick_weighted_ore(weights, roll) {
            return item_id;
        }
    }

    // Fallback: global rarity-skewed distribution. Only ONE of the two rolls is
    // skewed by rarity, so lower rarities always retain a chance.
    let a = dsl.ctx().rng().gen_range(0..100);
    let b = dsl
        .ctx()
        .rng()
        .gen_range((*asteroid_sector.get_rarity() as i32)..100);
    match a.min(b) {
        0..25 => ITEM_ICE_ORE,
        25..60 => ITEM_IRON_ORE,
        60..75 => ITEM_SILICON_ORE,
        75..85 => ITEM_GOLD_ORE,
        85..90 => ITEM_VIVEIUM_ORE,
        _ => ITEM_URANIUM_ORE,
    }
}

/// Spawns a single asteroid at a random position inside the field, with ore
/// type drawn from the sector's composition (or the global fallback). Pure spawn
/// logic — authorization is the calling reducer's responsibility. Returns `None`
/// if the creation failed.
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

    let item = ItemDefinitionId::new(roll_ore_item(dsl, asteroid_sector));

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

#[cfg(test)]
mod tests {
    use super::pick_weighted_ore;
    use crate::tables::sectors::OreWeight;

    fn w(item_id: u32, weight: u16) -> OreWeight {
        OreWeight { item_id, weight }
    }

    #[test]
    fn weighted_pick_respects_boundaries() {
        // total = 5; rolls 0,1,2 -> item 10; rolls 3,4 -> item 20.
        let weights = vec![w(10, 3), w(20, 2)];
        assert_eq!(pick_weighted_ore(&weights, 0), Some(10));
        assert_eq!(pick_weighted_ore(&weights, 2), Some(10));
        assert_eq!(pick_weighted_ore(&weights, 3), Some(20));
        assert_eq!(pick_weighted_ore(&weights, 4), Some(20));
    }

    #[test]
    fn empty_or_zero_weights_fall_through() {
        assert_eq!(pick_weighted_ore(&[], 0), None);
        // zero-weight entries are skipped, not selected.
        let weights = vec![w(1, 0), w(2, 1)];
        assert_eq!(pick_weighted_ore(&weights, 0), Some(2));
        assert_eq!(pick_weighted_ore(&vec![w(1, 0)], 0), None);
    }
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
