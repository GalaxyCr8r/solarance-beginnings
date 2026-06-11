use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use solarance_shared::Vec2;

use crate::{
    definitions::{
        factions::{FACTION_LRAK_COMBINE, FACTION_REDIAR_FEDERATION},
        item_types::{ITEM_IRON_ORE, ITEM_SILICON_ORE},
    },
    logic::sectors::asteroid_fields::fill_asteroid_sector,
    logic::stations::{contribution::create_construction_site, *},
    logic::stellarobjects::stellar_object_creation::create_sobj,
    tables::{
        economy::ResourceAmount, factions::*, sectors::*, star_system::*, stations::*,
        stellarobjects::*,
    },
};

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    demo_sectors(dsl)?;

    info!("Sectors Loaded: {}", dsl.get_all_sectors().count());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn demo_sectors(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let faction_lrak = FactionId::new(FACTION_LRAK_COMBINE);
    let faction_rediar = FactionId::new(FACTION_REDIAR_FEDERATION);
    let procyon = create_procyon_star_system(dsl, &faction_lrak)?;
    let (alpha, beta, gamma) = create_procyon_sectors(dsl, &procyon, &faction_lrak)?;
    let delta = create_rediar_sector(dsl, &procyon, &faction_rediar)?;

    setup_sector_connections(dsl, &alpha, &beta, &gamma, &delta)?;
    populate_sectors_with_asteroids(dsl, &alpha, &beta)?;
    create_sector_stations(dsl, &alpha, &beta, &gamma, &faction_lrak)?;
    create_delta_capital_station(dsl, &delta, &faction_rediar)?;
    Ok(())
}

/// Creates the Procyon star system with all its celestial objects
fn create_procyon_star_system(
    dsl: &DSL<'_, ReducerContext>,
    faction_id: &FactionId,
) -> Result<StarSystem, String> {
    let procyon = dsl.create_star_system(CreateStarSystem {
        name: "Procyon".to_string(),
        map_coordinates: Vec2::new(13.0, 37.0),
        spectral: SpectralKind::G,
        luminosity: 5,
        controlling_faction_id: faction_id.clone(),
    })?;

    // Create celestial objects in the star system
    let _star = dsl.create_star_system_object(CreateStarSystemObject {
        system_id: procyon.get_id(),
        kind: StarSystemObjectKind::Star,
        orbit_au: 0.0,
        rotation_or_width_km: 0.0,
        gfx_key: Some("star.1".to_string()),
    });
    let _planet1 = dsl.create_star_system_object(CreateStarSystemObject {
        system_id: procyon.get_id(),
        kind: StarSystemObjectKind::Planet,
        orbit_au: 128.0,
        rotation_or_width_km: 0.0,
        gfx_key: Some("planet.1".to_string()),
    });
    let _planet2 = dsl.create_star_system_object(CreateStarSystemObject {
        system_id: procyon.get_id(),
        kind: StarSystemObjectKind::Planet,
        orbit_au: -24.0,
        rotation_or_width_km: (90f32).to_radians(),
        gfx_key: Some("planet.2".to_string()),
    });
    let _moon = dsl.create_star_system_object(CreateStarSystemObject {
        system_id: procyon.get_id(),
        kind: StarSystemObjectKind::Moon,
        orbit_au: 130.0,
        rotation_or_width_km: (3.0_f32).to_radians(),
        gfx_key: None,
    });
    let _astbelt = dsl.create_star_system_object(CreateStarSystemObject {
        system_id: procyon.get_id(),
        kind: StarSystemObjectKind::AsteroidBelt,
        orbit_au: 48.0,
        rotation_or_width_km: 12.0,
        gfx_key: None,
    });

    let _nebbelt = dsl.create_star_system_object(CreateStarSystemObject {
        system_id: procyon.get_id(),
        kind: StarSystemObjectKind::NebulaBelt,
        orbit_au: 12.0,
        rotation_or_width_km: 8.0,
        gfx_key: None,
    });

    Ok(procyon)
}

/// Creates the three main sectors in the Procyon system
fn create_procyon_sectors(
    dsl: &DSL<'_, ReducerContext>,
    procyon: &StarSystem,
    faction_id: &FactionId,
) -> Result<(Sector, Sector, Sector), String> {
    let alpha = dsl.create_sector(CreateSector {
        id: 0,
        system_id: procyon.get_id(),
        name: "Alpha Sector".to_string(),
        description: None,
        controlling_faction_id: faction_id.clone(),
        security_level: 5,
        sunlight: 0.9,
        anomalous: 0.1,
        nebula: 0.1,
        rare_ore: 0.1,
        x: 4.0,
        y: 0.0,
        background_gfx_key: None,
    })?;

    let beta = dsl.create_sector(CreateSector {
        id: 1,
        system_id: procyon.get_id(),
        name: "Beta Sector".to_string(),
        description: None,
        controlling_faction_id: faction_id.clone(),
        security_level: 9,
        sunlight: 0.9,
        anomalous: 0.1,
        nebula: 0.1,
        rare_ore: 0.1,
        x: 2.0,
        y: -24.0,
        background_gfx_key: None,
    })?;

    let gamma = dsl.create_sector(CreateSector {
        id: 2,
        system_id: procyon.get_id(),
        name: "Homeworld Sector".to_string(),
        description: None,
        controlling_faction_id: faction_id.clone(),
        security_level: 10,
        sunlight: 0.9,
        anomalous: 0.1,
        nebula: 0.1,
        rare_ore: 0.1,
        x: 126.0,
        y: 0.0,
        background_gfx_key: None,
    })?;

    Ok((alpha, beta, gamma))
}

/// Creates the Rediar Federation home sector (#105). Sits on the far side of
/// the system from Lrak's Homeworld Sector so the two capitals bracket the
/// neutral middle sectors.
fn create_rediar_sector(
    dsl: &DSL<'_, ReducerContext>,
    procyon: &StarSystem,
    faction_rediar: &FactionId,
) -> Result<Sector, String> {
    let delta = dsl.create_sector(CreateSector {
        id: 3,
        system_id: procyon.get_id(),
        name: "Federation Prime Sector".to_string(),
        description: None,
        controlling_faction_id: faction_rediar.clone(),
        security_level: 10,
        sunlight: 0.8,
        anomalous: 0.1,
        nebula: 0.2,
        rare_ore: 0.1,
        x: -120.0,
        y: 8.0,
        background_gfx_key: None,
    })?;
    Ok(delta)
}

/// Sets up warp gate connections between sectors
fn setup_sector_connections(
    dsl: &DSL<'_, ReducerContext>,
    alpha: &Sector,
    beta: &Sector,
    gamma: &Sector,
    delta: &Sector,
) -> Result<(), String> {
    connect_sectors_with_warpgates(dsl, alpha, beta)?;
    connect_sectors_with_warpgates(dsl, beta, gamma)?;
    // Rediar's capital connects through Alpha — both faction capitals sit two
    // jumps apart with the neutral sectors between them.
    connect_sectors_with_warpgates(dsl, delta, alpha)?;
    Ok(())
}

/// Populates sectors with asteroid fields, then stocks each field to its full
/// target population so asteroids exist from t=0. The hourly `sector_upkeep`
/// timer only replenishes fields after they've been mined down.
fn populate_sectors_with_asteroids(
    dsl: &DSL<'_, ReducerContext>,
    alpha: &Sector,
    beta: &Sector,
) -> Result<(), String> {
    let alpha_field = dsl.create_asteroid_sector(CreateAsteroidSector {
        id: alpha.get_id(),
        sparseness: 1,
        rarity: 25,
        cluster_extent: 3000.0,
        cluster_inner: Some(1000.0),
    })?;
    let beta_field = dsl.create_asteroid_sector(CreateAsteroidSector {
        id: beta.get_id(),
        sparseness: 5,
        rarity: 0,
        cluster_extent: 5000.0,
        cluster_inner: None,
    })?;

    fill_asteroid_sector(dsl, &alpha_field)?;
    fill_asteroid_sector(dsl, &beta_field)?;

    Ok(())
}

/// Creates stations in each sector with appropriate modules
fn create_sector_stations(
    dsl: &DSL<'_, ReducerContext>,
    alpha: &Sector,
    beta: &Sector,
    gamma: &Sector,
    faction_id: &FactionId,
) -> Result<(), String> {
    create_beta_trading_station(dsl, beta, faction_id)?;
    create_alpha_refinery_station(dsl, alpha, faction_id)?;
    create_alpha_construction_site(dsl, alpha, faction_id)?;
    create_gamma_capital_station(dsl, gamma, faction_id)?;
    Ok(())
}

/// Creates a single under-construction site in Alpha for the M1 spike.
/// Spike-grade quantities (100 iron, 50 silicon) — small enough to drive to
/// 100% in one playtest. M4 will replace this with proper per-sector
/// construction sites; for now this is the only test target the
/// `contribute_to_station` reducer can point at after `--clear-database`.
fn create_alpha_construction_site(
    dsl: &DSL<'_, ReducerContext>,
    alpha: &Sector,
    faction_id: &FactionId,
) -> Result<(), String> {
    let _station = create_construction_site(
        dsl,
        StationSize::Small,
        alpha,
        &create_sobj(dsl, StellarObjectKinds::Station, &alpha.get_id())?,
        faction_id.clone(),
        "Alpha Outpost",
        Vec2::new(2000.0, 0.0),
        0.0,
        vec![
            ResourceAmount::new(ITEM_IRON_ORE, 100),
            ResourceAmount::new(ITEM_SILICON_ORE, 50),
        ],
    )?;
    Ok(())
}

/// Creates the trading station in Beta sector
fn create_beta_trading_station(
    dsl: &DSL<'_, ReducerContext>,
    beta: &Sector,
    faction_id: &FactionId,
) -> Result<(), String> {
    let _station = create_station_with_modules(
        dsl,
        StationSize::Medium,
        beta,
        &create_sobj(dsl, StellarObjectKinds::Station, &beta.get_id())?,
        faction_id.clone(),
        format!("{} Trading Station", beta.get_name()).as_str(),
        None,
        Vec2::new(613.0, 1337.0),
        0.0,
        vec![create_trading_module()],
    )?;
    Ok(())
}

/// Creates the refinery station in Alpha sector
fn create_alpha_refinery_station(
    dsl: &DSL<'_, ReducerContext>,
    alpha: &Sector,
    faction_id: &FactionId,
) -> Result<(), String> {
    let _station = create_station_with_modules(
        dsl,
        StationSize::Outpost,
        alpha,
        &create_sobj(dsl, StellarObjectKinds::Station, &alpha.get_id())?,
        faction_id.clone(),
        "Tarol's Rest & Refinery Stop",
        None,
        Vec2::new(0.0, 0.0),
        0.0,
        vec![
            create_iron_refinery_module(),
            create_ice_refinery_module(),
            create_silicon_refinery_module(),
        ],
    )?;
    Ok(())
}

/// Creates the Gamma capital station
fn create_gamma_capital_station(
    dsl: &DSL<'_, ReducerContext>,
    gamma: &Sector,
    faction_id: &FactionId,
) -> Result<(), String> {
    let station = create_station_with_modules(
        dsl,
        StationSize::Capital,
        gamma,
        &create_sobj(dsl, StellarObjectKinds::Station, &gamma.get_id())?,
        faction_id.clone(),
        "Homeworld City",
        None,
        Vec2::new(455.0, -1337.0),
        0.0,
        vec![create_trading_module()],
    )?;

    let mut faction = dsl.get_faction_by_id(faction_id)?;
    faction.set_capital_station_id(Some(station.get_id().value()));

    dsl.update_faction_by_id(faction)?;

    Ok(())
}

/// Creates the Rediar Federation capital station in Federation Prime Sector
/// and stamps it as the faction's Capital — the spawn anchor for new Rediar
/// players (#105).
fn create_delta_capital_station(
    dsl: &DSL<'_, ReducerContext>,
    delta: &Sector,
    faction_rediar: &FactionId,
) -> Result<(), String> {
    let station = create_station_with_modules(
        dsl,
        StationSize::Capital,
        delta,
        &create_sobj(dsl, StellarObjectKinds::Station, &delta.get_id())?,
        faction_rediar.clone(),
        "Federation Prime",
        None,
        Vec2::new(-455.0, 1337.0),
        0.0,
        vec![create_trading_module()],
    )?;

    let mut faction = dsl.get_faction_by_id(faction_rediar)?;
    faction.set_capital_station_id(Some(station.get_id().value()));

    dsl.update_faction_by_id(faction)?;

    Ok(())
}
