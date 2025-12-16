use std::f32::consts::PI;

use glam::Vec2;
use spacetimedb::rand::Rng;
use spacetimedsl::*;

use crate::{
    definitions::item_types::*,
    tables::{asteroids::*, items::*, sectors::*},
    utility::try_server_only,
};

/// Function that maintains asteroid populations in sectors.
/// Creates new asteroids when the count falls below the sector's sparseness threshold.
pub fn asteroid_sector_upkeep(dsl: &DSL<T>, sector_id: &SectorId) -> Result<(), String> {
    let ctx = dsl.ctx();
    try_server_only(dsl)?;

    let asteroid_sector = dsl.get_asteroid_sector_by_id(sector_id)?;
    if dsl.get_asteroids_by_current_sector_id(sector_id).count()
        < (asteroid_sector.get_sparseness() * 10).into()
    {
        // 100
        let field = *asteroid_sector.get_cluster_extent();
        let dist = match asteroid_sector.get_cluster_inner() {
            Some(inner_extent) => ctx.rng().gen_range(*inner_extent..field), // Pick a distance between inner and outer bounds.,
            None => ctx.rng().gen_range(0.0..field),
        };
        let pos = Vec2::from_angle(ctx.rng().gen_range(0.0..2.0 * PI)) * dist;

        let roll_with_disadvantage = {
            let a = ctx.rng().gen_range(0..100);
            // Only ONE of the 'rolls' should be effected by rarity, so that there's always a chance for lower rarities.
            let b = ctx
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

        let amount = ctx.rng().gen_range(500..2000);

        create_asteroid(dsl, pos, sector_id.into(), item, amount);
    }

    Ok(())
}
