use log::info;
use spacetimedb::SpacetimeType;
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

pub fn init<T: spacetimedsl::WriteContext + 'static>(dsl: &DSL<T>) -> Result<(), String> {
    demo_sectors(dsl)?;

    info!("Sectors Loaded: {}", dsl.get_all_sectors().count());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn demo_sectors<T: spacetimedsl::WriteContext + 'static>(dsl: &DSL<T>) -> Result<(), String> {
    let faction_lrak = FactionId::new(FACTION_LRAK_COMBINE);
    let procyon = create_procyon_star_system(dsl, &faction_lrak)?;
    let (alpha, beta, gamma) = create_procyon_sectors(dsl, &procyon, &faction_lrak)?;

    setup_sector_connections(dsl, &alpha, &beta, &gamma)?;
    populate_sectors_with_asteroids(dsl, &alpha, &beta)?;
    create_sector_stations(dsl, &alpha, &beta, &gamma, &faction_lrak)?;
    Ok(())
}

/// Creates the Procyon star system with all its celestial objects
fn create_procyon_star_system<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
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
fn create_procyon_sectors<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
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

/// Sets up warp gate connections between sectors
fn setup_sector_connections<T: spacetimedsl::WriteContext>(
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
fn populate_sectors_with_asteroids<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    alpha: &Sector,
    beta: &Sector,
) -> Result<(), String> {
    dsl.create_asteroid_sector(CreateAsteroidSector {
        id: alpha.get_id(),
        sparseness: 1,
        rarity: 25,
        cluster_extent: 3000.0,
        cluster_inner: Some(1000.0),
    })?;
    dsl.create_asteroid_sector(CreateAsteroidSector {
        id: beta.get_id(),
        sparseness: 5,
        rarity: 0,
        cluster_extent: 5000.0,
        cluster_inner: None,
    })?;

    Ok(())
}

/// Creates stations in each sector with appropriate modules
fn create_sector_stations<T: spacetimedsl::WriteContext + 'static>(
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
fn create_beta_trading_station<T: spacetimedsl::WriteContext + 'static>(
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
fn create_alpha_refinery_station<T: spacetimedsl::WriteContext + 'static>(
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

/// Creates the Gamma capital station
fn create_gamma_capital_station<T: spacetimedsl::WriteContext + 'static>(
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
