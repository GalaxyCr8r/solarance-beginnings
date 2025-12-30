use glam::Vec2;
use log::info;
use spacetimedb::table;
use spacetimedsl::*;

use crate::logic::stellarobjects::stellar_object_creation::*;
use crate::tables::{items::ItemDefinitionId, sectors::SectorId, stellarobjects::*};

#[dsl(plural_name = asteroids, method(update = true))]
#[table(name = asteroid, public)]
pub struct Asteroid {
    #[primary_key]
    #[use_wrapper(StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    #[index(btree)] // To find asteroids in a specific sector
    #[use_wrapper(crate::tables::sectors::SectorId)]
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Delete)]
    /// FK to Sector.id // Because asteroid_sector.id exists, this can't be named sector_id.
    current_sector_id: u64,

    size_radius: f32, // For collision

    #[use_wrapper(crate::tables::items::ItemDefinitionId)]
    #[index(btree)]
    #[foreign_key(path = crate::tables::items, table = item_definition, column = id, on_delete = Delete)]
    /// FK to ItemDefinition (e.g., Iron Ore, Silicon)
    resource_item_id: u32,

    pub current_resources: u16, // Amount of resources left
    initial_resources: u16,     // Original amount, for reference or respawn logic

    gfx_key: Option<String>, // For client side
}

/////////////////////////////////////////
/// Utility

pub fn create_asteroid<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    position: Vec2,
    sector: SectorId,
    gfx_key: String,
    item: ItemDefinitionId,
    resource_amount: u16,
) -> Option<Asteroid> {
    let sobj = create_sobj_internal(
        &dsl,
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

    match dsl.create_asteroid(CreateAsteroid {
        id: sobj.unwrap().get_id(), // u64 or wrapped? Assuming wrapper based on use_wrapper
        current_sector_id: sector.clone(),
        size_radius: 16.0,
        resource_item_id: item.clone(),
        current_resources: resource_amount,
        initial_resources: resource_amount,
        gfx_key: Some(gfx_key),
    }) {
        Ok(ast) => Some(ast),
        Err(e) => {
            info!("ERROR: Failed to create asteroid: {}", e);
            None
        }
    }
}
