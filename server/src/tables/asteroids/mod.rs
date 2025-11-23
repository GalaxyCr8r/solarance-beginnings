use glam::Vec2;
use log::info;
use spacetimedb::{rand::Rng, table, ReducerContext};
use spacetimedsl::dsl;

use crate::tables::{
    items::ItemDefinitionId,
    sectors::SectorId,
    stellarobjects::{utility::*, *},
};

#[dsl(plural_name = asteroids)]
#[table(name = asteroid, public)]
pub struct Asteroid {
    #[primary_key]
    #[use_wrapper(path = StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    #[index(btree)] // To find asteroids in a specific sector
    #[use_wrapper(path = crate::tables::sectors::SectorId)]
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Delete)]
    /// FK to Sector.id // Because asteroid_sector.id exists, this can't be named sector_id.
    pub current_sector_id: u64,

    pub size_radius: f32, // For collision

    #[use_wrapper(path = crate::tables::items::ItemDefinitionId)]
    #[index(btree)]
    #[foreign_key(path = crate::tables::items, table = item_definition, column = id, on_delete = Delete)]
    /// FK to ItemDefinition (e.g., Iron Ore, Silicon)
    pub resource_item_id: u32,

    pub current_resources: u16, // Amount of resources left
    pub initial_resources: u16, // Original amount, for reference or respawn logic

    pub gfx_key: Option<String>, // For client side
}

/////////////////////////////////////////
/// Utility

pub fn create_asteroid(
    ctx: &ReducerContext,
    position: Vec2,
    sector: SectorId,
    item: ItemDefinitionId,
    resource_amount: u16,
) -> Option<Asteroid> {
    let dsl = dsl(ctx);

    let gfx_key = format!("asteroid.{}", ctx.rng().gen_range(1..=5)).to_string();

    let sobj = create_sobj_internal(
        ctx,
        StellarObjectKinds::Asteroid,
        &sector,
        StellarObjectTransformInternal::default().from_vec2(position),
    );
    if sobj.is_err() {
        info!(
            "ERROR: Couldn't create stellar object for asteroid: {}",
            sobj.unwrap_err()
        );
        return None;
    }

    match dsl.create_asteroid(
        &sobj.unwrap(),
        sector,
        16.0,
        item,
        resource_amount,
        resource_amount,
        Some(gfx_key),
    ) {
        Ok(ast) => Some(ast),
        Err(e) => {
            info!("ERROR: Failed to create asteroid: {}", e);
            None
        }
    }
}
