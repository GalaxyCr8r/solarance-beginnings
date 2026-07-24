//! Stellar-object creation helpers.
//!
//! After Phase 9 of the dead-reckoning rewrite the only thing on a
//! `StellarObject` row is `(kind, sector_id)`. Position / rotation live on
//! the per-kind table (`Ship.movement`, `Asteroid.position`, etc.), so the
//! legacy `create_sobj_internal` / `create_sobj_pos` / `create_sobj_vec2`
//! helpers were collapsed into the single thin wrapper below.

use crate::tables::sectors::SectorId;
use crate::tables::stellarobjects::*;
use crate::utility::try_server_only;
use spacetimedb::ReducerContext;
use crate::spacetimedsl::prelude::*;

/// Creates a stellar object with no position payload. The caller is
/// responsible for populating the per-kind position field on the
/// corresponding ship / asteroid / station / jumpgate / cargo crate row.
pub fn create_sobj<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    kind: StellarObjectKinds,
    sector_id: &SectorId,
) -> Result<StellarObject, String> {
    Ok(dsl.create_stellar_object(CreateStellarObject {
        kind,
        sector_id: sector_id.clone(),
    })?)
}

/// Admin-only reducer for creating a stellar object directly. Used by
/// debug / scripting; gameplay code should create through the higher-level
/// per-kind helpers.
#[spacetimedb::reducer]
pub fn create_stellar_object(
    ctx: &ReducerContext,
    kind: StellarObjectKinds,
    sector_id: SectorId,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;
    create_sobj(&dsl, kind, &sector_id).map(|_| ())
}
