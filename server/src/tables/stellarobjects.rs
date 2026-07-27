use spacetimedb::{table, SpacetimeType};
use crate::spacetimedsl::prelude::*;

/// What kind of stellar object it is. OBE with the advent of asteroid/ship/station tables?
#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum StellarObjectKinds {
    Ship,
    Asteroid,
    CargoCrate,
    Station,
    JumpGate,
}

/// An object that exists inside a sector.
///
/// After the Phase 9 dead-reckoning rewrite this row only carries the kind
/// and sector — position/rotation live on the per-kind tables
/// (`Ship.movement`, `CargoCrate.movement`, `Asteroid.position`,
/// `Station.position`/`rotation`, `JumpGate.position`/`rotation`).
#[spacetimedsl::dsl(plural_name = stellar_objects, method(update = true))]
#[table(accessor = stellar_object, public)]
pub struct StellarObject {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::tables::asteroids, table = asteroid)]
    #[referenced_by(path = crate::tables::ships, table = ship)]
    #[referenced_by(path = crate::tables::stations, table = station)]
    #[referenced_by(path = crate::tables::jumpgates, table = jump_gate)]
    #[referenced_by(path = crate::tables::items, table = cargo_crate)]
    id: u64,

    #[index(btree)]
    pub kind: StellarObjectKinds,

    #[index(btree)]
    #[use_wrapper(crate::tables::sectors::SectorId)]
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Delete)]
    /// FK to SectorLocation
    pub sector_id: u64,
}

//////////////////////////////////////////////////////////////
// Utilities

pub fn same_sector_from_ids<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    id1: &StellarObjectId,
    id2: &StellarObjectId,
) -> bool {
    if let Ok(sobj1) = dsl.get_stellar_object_by_id(id1) {
        if let Ok(sobj2) = dsl.get_stellar_object_by_id(id2) {
            return sobj1.sector_id == sobj2.sector_id;
        }
    }
    false
}

//////////////////////////////////////////////////////////////
// Impls
//////////////////////////////////////////////////////////////

impl StellarObject {
    pub fn id(&self) -> u64 {
        self.id
    }

    // `distance_squared(other)` was removed during the dead-reckoning rewrite.
    // Use `logic::stellarobjects::movement::get_sobj_position` to fetch the
    // current position for each sobj, then compute the distance yourself.
}
