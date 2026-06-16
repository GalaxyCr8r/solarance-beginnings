use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::*;

use crate::logic::stations::contribution::{create_construction_site, reset_construction_site};
use crate::logic::stations::*;
use crate::logic::stellarobjects::stellar_object_creation::create_sobj;
use crate::tables::{
    economy::ResourceAmount, factions::FactionId, sectors::*, stations::*,
    stellarobjects::StellarObjectKinds,
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
    size: StationSize,
    owner_faction_id: u32,
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
        size.clone(),
        &sector,
        &sobj,
        FactionId::new(owner_faction_id),
        &name,
        solarance_shared::Vec2::new(x, y),
        0.0,
        requirements.clone(),
    )?;

    info!(
        "admin_create_construction_site: caller={} sector_id={} station_id={} name={:?} size={:?} faction={} pos=({:.1},{:.1}) requirements={}",
        ctx.sender().to_abbreviated_hex(),
        sector_id,
        station.get_id().value(),
        name,
        size,
        owner_faction_id,
        x,
        y,
        requirements.len(),
    );
    Ok(())
}

/// Map a well-known module key to its creator function. Shared by
/// `admin_place_station` (fitting a new station) and `admin_add_station_module`
/// (fitting an existing one) so the two paths can't drift. The error lists the
/// valid keys so a typo is self-diagnosing in the logs.
fn module_creator_from_key(key: &str) -> Result<ModuleCreationFn<ReducerContext>, String> {
    Ok(match key {
        "trading" => create_trading_module(),
        "iron_refinery" => create_iron_refinery_module(),
        "ice_refinery" => create_ice_refinery_module(),
        "silicon_refinery" => create_silicon_refinery_module(),
        "solar_array" => create_small_solar_array_module(),
        "advanced_manufacturing" => create_advanced_manufacturing_module(),
        other => {
            return Err(format!(
                "unknown module key {:?} (known: trading, iron_refinery, ice_refinery, silicon_refinery, solar_array, advanced_manufacturing)",
                other
            ));
        }
    })
}

/// Galaxy Creator (#34): place a *finished*, operational station directly —
/// bypassing the construction phase — optionally fitted with modules. Mirrors
/// the init-time `create_station_with_modules` seeds so the designer can drop a
/// working station (trading post, refinery, etc.) without republishing.
///
/// `modules` is a list of well-known module keys; an unknown key is a hard
/// error so typos surface in the logs rather than silently producing an empty
/// station.
#[spacetimedb::reducer]
pub fn admin_place_station(
    ctx: &ReducerContext,
    sector_id: u64,
    name: String,
    size: StationSize,
    owner_faction_id: u32,
    x: f32,
    y: f32,
    modules: Vec<String>,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    if name.trim().is_empty() {
        return Err("admin_place_station: name must not be empty".to_string());
    }

    let sector = dsl.get_sector_by_id(&SectorId::new(sector_id))?;
    let sobj = create_sobj(&dsl, StellarObjectKinds::Station, &sector.get_id())?;

    let mut module_creators: Vec<ModuleCreationFn<ReducerContext>> =
        Vec::with_capacity(modules.len());
    for key in &modules {
        module_creators
            .push(module_creator_from_key(key).map_err(|e| format!("admin_place_station: {e}"))?);
    }

    let station = create_station_with_modules(
        &dsl,
        size.clone(),
        &sector,
        &sobj,
        FactionId::new(owner_faction_id),
        name.trim(),
        None,
        solarance_shared::Vec2::new(x, y),
        0.0,
        module_creators,
    )?;

    info!(
        "admin_place_station: caller={} sector_id={} station_id={} name={:?} size={:?} faction={} modules={} pos=({:.1},{:.1})",
        ctx.sender().to_abbreviated_hex(),
        sector_id,
        station.get_id().value(),
        name,
        size,
        owner_faction_id,
        modules.len(),
        x,
        y,
    );
    Ok(())
}

/// Galaxy Creator (#34): fit a module onto an *existing* station. This is the
/// counterpart to `admin_place_station` for stations that already exist — most
/// importantly construction sites, which complete with **zero** modules
/// (`create_construction_site` seeds an empty module list and completion only
/// flips `is_operational`). Without this, a finished construction site would
/// have nothing to produce.
///
/// `verify` runs afterward so the size's module cap is enforced — and because
/// reducers are transactional, an over-cap add rolls back rather than
/// persisting a half-fitted station.
#[spacetimedb::reducer]
pub fn admin_add_station_module(
    ctx: &ReducerContext,
    station_id: u64,
    module_key: String,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    let station = dsl.get_station_by_id(&StationId::new(station_id))?;

    let creator = module_creator_from_key(&module_key)
        .map_err(|e| format!("admin_add_station_module: {e}"))?;
    creator(&dsl, &station)?;
    verify(&dsl, &station)?;

    info!(
        "admin_add_station_module: caller={} station_id={} module={:?}",
        ctx.sender().to_abbreviated_hex(),
        station_id,
        module_key,
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
