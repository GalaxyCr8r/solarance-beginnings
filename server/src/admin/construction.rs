use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::*;

use crate::logic::stations::contribution::{create_construction_site, reset_construction_site};
use crate::logic::stellarobjects::stellar_object_creation::create_sobj;
use crate::tables::{
    economy::ResourceAmount, sectors::*, stations::*, stellarobjects::StellarObjectKinds,
};
use crate::utility::try_server_only;

/// Drop a fresh construction site into the world at runtime. Mirrors the
/// init-time seed in `definitions/galaxy.rs` so the designer can spawn extra
/// test targets without republishing the module.
#[spacetimedb::reducer]
pub fn admin_create_construction_site(
    ctx: &ReducerContext,
    sector_id: u64,
    name: String,
    x: f32,
    y: f32,
    requirements: Vec<ResourceAmount>,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    if name.trim().is_empty() {
        return Err("admin_create_construction_site: name must not be empty".to_string());
    }

    let sector = dsl.get_sector_by_id(&SectorId::new(sector_id))?;
    let sobj = create_sobj(&dsl, StellarObjectKinds::Station, &sector.get_id())?;
    let station = create_construction_site(
        &dsl,
        StationSize::Small,
        &sector,
        &sobj,
        sector.get_controlling_faction_id().clone(),
        &name,
        solarance_shared::Vec2::new(x, y),
        0.0,
        requirements.clone(),
    )?;

    info!(
        "admin_create_construction_site: caller={} sector_id={} station_id={} name={:?} pos=({:.1},{:.1}) requirements={}",
        ctx.sender().to_abbreviated_hex(),
        sector_id,
        station.get_id().value(),
        name,
        x,
        y,
        requirements.len(),
    );
    Ok(())
}

/// Wipe the contribution log for a station and zero its progress bar so the
/// completion moment can be replayed without `--clear-database`.
#[spacetimedb::reducer]
pub fn admin_reset_construction_site(
    ctx: &ReducerContext,
    station_id: u64,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    let station_id = StationId::new(station_id);
    // Surface a clean error if the target isn't actually a construction site.
    dsl.get_station_under_construction_by_id(&station_id).map_err(|e| {
        format!(
            "admin_reset_construction_site: station {} is not under construction ({})",
            station_id.value(),
            e
        )
    })?;

    reset_construction_site(&dsl, &station_id)?;

    info!(
        "admin_reset_construction_site: caller={} station_id={}",
        ctx.sender().to_abbreviated_hex(),
        station_id.value(),
    );
    Ok(())
}
