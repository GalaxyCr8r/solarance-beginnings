use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::*;

use crate::{
    definitions::factions::FACTION_LRAK_COMBINE,
    logic::stations::*,
    logic::stellarobjects::stellar_object_creation::*,
    tables::{
        common_types::Vec2, factions::*, sectors::*, star_system::*, stations::*, stellarobjects::*,
    },
};

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    demo_sectors(dsl)?;

    info!("Sectors Loaded: {}", dsl.get_all_sectors().count());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn demo_sectors<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    let faction_lrak = FactionId::new(FACTION_LRAK_COMBINE);
    let procyon = create_procyon_star_system(dsl, &faction_lrak)?;
    let (alpha, beta, gamma) = create_procyon_sectors(dsl, &procyon, &faction_lrak)?;

    setup_sector_connections(dsl, &alpha, &beta, &gamma)?;
    populate_sectors_with_asteroids(dsl, &alpha, &beta)?;
    create_sector_stations(dsl, &alpha, &beta, &gamma, &faction_lrak)?;
    Ok(())
}

/// Creates the Procyon star system with all its celestial objects
fn create_procyon_star_system(dsl: &DSL<T>, faction_id: &FactionId) -> Result<StarSystem, String> {
    let procyon = dsl.create_star_system(
        "Procyon",
        Vec2::new(13.0, 37.0),
        SpectralKind::G,
        5,
        faction_id,
    )?;

    // Create celestial objects in the star system
    let _star = dsl.create_star_system_object(
        &procyon,
        StarSystemObjectKind::Star,
        0.0,
        0.0,
        Some("star.1".to_string()),
    );
    let _planet1 = dsl.create_star_system_object(
        &procyon,
        StarSystemObjectKind::Planet,
        128.0,
        0.0,
        Some("planet.1".to_string()),
    );
    let _planet2 = dsl.create_star_system_object(
        &procyon,
        StarSystemObjectKind::Planet,
        -24.0,
        (90f32).to_radians(),
        Some("planet.2".to_string()),
    );
    let _moon = dsl.create_star_system_object(
        &procyon,
        StarSystemObjectKind::Moon,
        130.0,
        (3.0_f32).to_radians(),
        None,
    );
    let _astbelt = dsl.create_star_system_object(
        &procyon,
        StarSystemObjectKind::AsteroidBelt,
        48.0,
        12.0,
        None,
    );

    let _nebbelt =
        dsl.create_star_system_object(&procyon, StarSystemObjectKind::NebulaBelt, 12.0, 8.0, None);

    Ok(procyon)
}

/// Creates the three main sectors in the Procyon system
fn create_procyon_sectors(
    dsl: &DSL<T>,
    procyon: &StarSystem,
    faction_id: &FactionId,
) -> Result<(Sector, Sector, Sector), String> {
    let alpha = dsl.create_sector(
        0,
        procyon,
        "Alpha Sector",
        None,
        faction_id,
        5,
        0.9,
        0.1,
        0.1,
        0.1,
        4.0,
        0.0,
        None,
    )?;

    let beta = dsl.create_sector(
        1,
        procyon,
        "Beta Sector",
        None,
        faction_id,
        9,
        0.9,
        0.1,
        0.1,
        0.1,
        2.0,
        -24.0,
        None,
    )?;

    let gamma = dsl.create_sector(
        2,
        procyon,
        "Homeworld Sector",
        None,
        faction_id,
        10,
        0.9,
        0.1,
        0.1,
        0.1,
        126.0,
        0.0,
        None,
    )?;

    Ok((alpha, beta, gamma))
}

/// Sets up warp gate connections between sectors
fn setup_sector_connections(
    dsl: &DSL<T>,
    alpha: &Sector,
    beta: &Sector,
    gamma: &Sector,
) -> Result<(), String> {
    connect_sectors_with_warpgates(dsl, alpha, beta)?;
    connect_sectors_with_warpgates(dsl, beta, gamma)?;
    Ok(())
}

/// Populates sectors with asteroid fields
fn populate_sectors_with_asteroids(dsl: &DSL<T>, alpha: &Sector, beta: &Sector) -> Result<(), String> {
    dsl.create_asteroid_sector(alpha, 1, 25, 3000.0, Some(1000.0))?;
    dsl.create_asteroid_sector(beta, 5, 0, 5000.0, None)?;

    Ok(())
}

/// Creates stations in each sector with appropriate modules
fn create_sector_stations(
    dsl: &DSL<T>,
    alpha: &Sector,
    beta: &Sector,
    gamma: &Sector,
    faction_id: &FactionId,
) -> Result<(), String> {
    create_beta_trading_station(dsl, beta, faction_id)?;
    create_alpha_refinery_station(dsl, alpha, faction_id)?;
    create_gamma_capital_station(dsl, gamma, faction_id)?;
    Ok(())
}

/// Creates the trading station in Beta sector
fn create_beta_trading_station(
    dsl: &DSL<T>,
    beta: &Sector,
    faction_id: &FactionId,
) -> Result<(), String> {
    let _station = create_station_with_modules(
        dsl,
        StationSize::Medium,
        beta,
        &create_sobj_internal(
            dsl,
            StellarObjectKinds::Station,
            &beta.get_id(),
            StellarObjectTransformInternal::default().from_xy(613.0, 1337.0),
        )?,
        faction_id.clone(),
        format!("{} Trading Station", beta.get_name()).as_str(),
        None,
        vec![create_trading_module()], //TODO:, create_metal_plate_module()],
    )?;
    Ok(())
}

/// Creates the refinery station in Alpha sector
fn create_alpha_refinery_station(
    dsl: &DSL<T>,
    alpha: &Sector,
    faction_id: &FactionId,
) -> Result<(), String> {
    let _station = create_station_with_modules(
        dsl,
        StationSize::Outpost,
        alpha,
        &create_sobj_internal(
            dsl,
            StellarObjectKinds::Station,
            &alpha.get_id(),
            StellarObjectTransformInternal::default(),
        )?,
        faction_id.clone(),
        "Tarol's Rest & Refinery Stop",
        None,
        vec![
            create_iron_refinery_module(),
            create_ice_refinery_module(),
            create_silicon_refinery_module(),
        ],
    )?;
    Ok(())
}

/// Creates the capital station in Gamma sector
fn create_gamma_capital_station(
    dsl: &DSL<T>,
    gamma: &Sector,
    faction_id: &FactionId,
) -> Result<(), String> {
    let station = create_station_with_modules(
        dsl,
        StationSize::Capital,
        gamma,
        &create_sobj_internal(
            dsl,
            StellarObjectKinds::Station,
            &gamma.get_id(),
            StellarObjectTransformInternal::default().from_xy(455.0, -1337.0),
        )?,
        faction_id.clone(),
        "Homeworld City",
        None,
        vec![create_trading_module()], // No modules for this capital station yet
    )?;

    let mut faction = dsl.get_faction_by_id(faction_id)?;
    faction.set_capital_station_id(Some(station.get_id().value()));

    dsl.update_faction_by_id(faction)?;

    Ok(())
}
