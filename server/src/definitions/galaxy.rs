use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use solarance_shared::Vec2;

use crate::{
    definitions::{
        factions::{
            FACTION_FACTIONLESS, FACTION_INDEPENDENT_WORLDS_ALLIANCE, FACTION_LRAK_COMBINE, FACTION_REDIAR_FEDERATION
        },
        item_types::{ITEM_CARBON_ORE, ITEM_GOLD_ORE, ITEM_IRON_ORE, ITEM_SILICON_ORE},
    },
    logic::{sectors::asteroid_fields::fill_asteroid_sector, stations::{contribution::create_construction_site, *}, stellarobjects::stellar_object_creation::create_sobj},
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

/// Seeds the MVP ten-sector Procyon system (#106). One star system, ten
/// functionally-differentiated sectors bracketed by the Lrak (east) and Rediar
/// (west) capitals with a neutral IWA middle, fully connected by a hub/spoke
/// jumpgate network.
///
/// "Neutral"/"None" sector control in the design table maps to the
/// **Independent Worlds Alliance** (IWA) — the unaligned governing faction —
/// since `Sector.controlling_faction_id` is a non-null FK.
fn demo_sectors(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let lrak = FactionId::new(FACTION_LRAK_COMBINE);
    let rediar = FactionId::new(FACTION_REDIAR_FEDERATION);
    let neutral = FactionId::new(FACTION_FACTIONLESS);

    let procyon = create_procyon_star_system(dsl, &lrak)?;
    let sectors = create_procyon_sectors(dsl, &procyon, &lrak, &rediar, &neutral)?;

    setup_sector_connections(dsl, &sectors)?;
    populate_sectors_with_asteroids(dsl, &sectors)?;
    create_sector_stations(dsl, &sectors, &lrak, &rediar, &neutral)?;
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

/// The ten seeded sectors, held by name so the wiring steps below
/// (connections, asteroids, stations) read clearly instead of juggling a
/// positional tuple. IDs 0..3 are kept stable for the four pre-existing
/// sectors so non-cleared databases don't orphan ships.
struct ProcyonSectors {
    tarols_belt: Sector,   // 0 — neutral, asteroid belt (was "Alpha")
    ore_trench: Sector,    // 1 — Lrak, rare-ore rich (was "Beta")
    lrakhold: Sector,      // 2 — Lrak capital (was "Gamma/Homeworld")
    echo_bay: Sector,      // 3 — Rediar capital (was "Delta/Federation Prime")
    the_hinge: Sector,     // 4 — neutral hub, central traffic
    karrens_reach: Sector, // 5 — Lrak frontier refinery (under construction)
    stilwater: Sector,     // 6 — Rediar outer hub
    quiet_belt: Sector,    // 7 — neutral mining spoke
    iron_furrow: Sector,   // 8 — Rediar refinery (under construction)
    pale_crossing: Sector, // 9 — Rediar outer hub
}

/// Creates the ten Procyon sectors per the #106 design table.
fn create_procyon_sectors(
    dsl: &DSL<'_, ReducerContext>,
    procyon: &StarSystem,
    lrak: &FactionId,
    rediar: &FactionId,
    neutral: &FactionId,
) -> Result<ProcyonSectors, String> {
    // Small helper so each sector is one readable line of differentiators.
    let mk = |id: u64,
                  name: &str,
                  faction: &FactionId,
                  security: u8,
                  sunlight: f32,
                  anomalous: f32,
                  nebula: f32,
                  rare_ore: f32,
                  x: f32,
                  y: f32|
     -> Result<Sector, String> {
        dsl.create_sector(CreateSector {
            id,
            system_id: procyon.get_id(),
            name: name.to_string(),
            description: None,
            controlling_faction_id: faction.clone(),
            security_level: security,
            sunlight,
            anomalous,
            nebula,
            rare_ore,
            x,
            y,
            background_gfx_key: None,
        })
        .map_err(|e| e.to_string())
    };

    // Pre-existing four (IDs kept; Tarol's Belt flipped Lrak -> neutral).
    let tarols_belt = mk(0, "Tarol's Belt", neutral, 5, 0.9, 0.1, 0.1, 0.2, 4.0, -30.0)?;
    let ore_trench = mk(1, "Ore Trench", lrak, 6, 0.8, 0.0, 0.1, 0.6, 75.0, -20.0)?;
    let lrakhold = mk(2, "Lrakhold", lrak, 10, 0.9, 0.1, 0.1, 0.1, 126.0, 8.0)?;
    let echo_bay = mk(3, "Echo Bay", rediar, 9, 0.9, 0.0, 0.1, 0.1, -120.0, 8.0)?;

    // Six new MVP sectors.
    let the_hinge = mk(4, "The Hinge", neutral, 5, 0.4, 0.0, 0.6, 0.0, -4.0, 10.0)?;
    let karrens_reach = mk(5, "Karren's Reach", lrak, 7, 0.7, 0.0, 0.1, 0.2, 100.0, 30.0)?;
    let stilwater = mk(6, "Stilwater", rediar, 8, 0.6, 0.4, 0.3, 0.0, -70.0, 40.0)?;
    let quiet_belt = mk(7, "Quiet Belt", neutral, 4, 0.5, 0.1, 0.2, 0.8, 40.0, -12.0)?;
    let iron_furrow = mk(8, "Iron Furrow", rediar, 6, 0.8, 0.0, 0.0, 0.1, -48.0, 5.0)?;
    let pale_crossing = mk(9, "Pale Crossing", rediar, 7, 0.6, 0.0, 0.4, 0.0, -85.0, -5.0)?;

    Ok(ProcyonSectors {
        tarols_belt,
        ore_trench,
        lrakhold,
        echo_bay,
        the_hinge,
        karrens_reach,
        stilwater,
        quiet_belt,
        iron_furrow,
        pale_crossing,
    })
}

/// Wires the hub/spoke jumpgate network. Every sector is reachable; hubs (The
/// Hinge, Pale Crossing, Stilwater, Ore Trench) carry 3+ gates, spokes
/// (Karren's Reach, Quiet Belt, Iron Furrow) 1-2. The two capitals bracket the
/// chain. Connectivity is validated separately by #108.
fn setup_sector_connections(
    dsl: &DSL<'_, ReducerContext>,
    s: &ProcyonSectors,
) -> Result<(), String> {
    let link = |a: &Sector, b: &Sector| connect_sectors_with_warpgates(dsl, a, b);

    // Rediar (west) cluster.
    link(&s.echo_bay, &s.pale_crossing)?;
    link(&s.echo_bay, &s.stilwater)?;
    link(&s.pale_crossing, &s.stilwater)?;
    link(&s.pale_crossing, &s.iron_furrow)?;
    link(&s.stilwater, &s.iron_furrow)?;

    // Bridge through the neutral middle.
    link(&s.pale_crossing, &s.the_hinge)?;
    link(&s.the_hinge, &s.tarols_belt)?;
    link(&s.the_hinge, &s.quiet_belt)?;

    // Lrak (east) cluster.
    link(&s.tarols_belt, &s.ore_trench)?;
    link(&s.quiet_belt, &s.ore_trench)?;
    link(&s.ore_trench, &s.lrakhold)?;
    link(&s.ore_trench, &s.karrens_reach)?;
    link(&s.lrakhold, &s.karrens_reach)?;

    Ok(())
}

/// Stocks the mining sectors with asteroid fields. Yield "type" is approximated
/// via density (`sparseness`) and rare-ore skew (`rarity`) — the spawn roll
/// distribution is global, so per-sector ore *composition* (e.g. "water +
/// viveium") isn't expressible yet; tracked as a follow-up to the asteroid
/// spawn logic.
fn populate_sectors_with_asteroids(
    dsl: &DSL<'_, ReducerContext>,
    s: &ProcyonSectors,
) -> Result<(), String> {
    let field = |sector: &Sector,
                 sparseness: u8,
                 rarity: u8,
                 cluster_extent: f32,
                 cluster_inner: Option<f32>|
     -> Result<(), String> {
        let created = dsl.create_asteroid_sector(CreateAsteroidSector {
            id: sector.get_id(),
            sparseness,
            rarity,
            cluster_extent,
            cluster_inner,
        })?;
        fill_asteroid_sector(dsl, &created)
    };

    field(&s.tarols_belt, 2, 25, 3000.0, Some(1000.0))?; // moderate, mixed
    field(&s.ore_trench, 5, 50, 5000.0, None)?; // dense, rare-ore rich
    field(&s.quiet_belt, 6, 70, 5000.0, None)?; // high yield
    field(&s.iron_furrow, 4, 10, 4000.0, None)?; // iron-heavy, common
    field(&s.karrens_reach, 3, 40, 3500.0, Some(800.0))?; // medium

    Ok(())
}

/// Places the seeded stations: two faction capitals (stamped onto their
/// factions as spawn anchors), two operational neutral/Lrak service stations,
/// and two under-construction refinery sites for the contribution loop.
fn create_sector_stations(
    dsl: &DSL<'_, ReducerContext>,
    s: &ProcyonSectors,
    lrak: &FactionId,
    rediar: &FactionId,
    neutral: &FactionId,
) -> Result<(), String> {
    // --- Faction capitals (spawn anchors) ---------------------------------
    let lrak_capital = create_station_with_modules(
        dsl,
        StationSize::Capital,
        &s.lrakhold,
        &create_sobj(dsl, StellarObjectKinds::Station, &s.lrakhold.get_id())?,
        lrak.clone(),
        "Lrakhold City",
        None,
        Vec2::new(455.0, -1337.0),
        0.0,
        vec![create_trading_module()],
    )?;
    stamp_capital(dsl, lrak, &lrak_capital)?;

    let rediar_capital = create_station_with_modules(
        dsl,
        StationSize::Capital,
        &s.echo_bay,
        &create_sobj(dsl, StellarObjectKinds::Station, &s.echo_bay.get_id())?,
        rediar.clone(),
        "Echo Bay Prime",
        None,
        Vec2::new(-455.0, 1337.0),
        0.0,
        vec![create_trading_module()],
    )?;
    stamp_capital(dsl, rediar, &rediar_capital)?;

    // --- Operational service stations -------------------------------------
    // Tarol's Belt: the neutral refinery rest-stop (kept from the original seed).
    let _tarols_refinery = create_station_with_modules(
        dsl,
        StationSize::Outpost,
        &s.tarols_belt,
        &create_sobj(dsl, StellarObjectKinds::Station, &s.tarols_belt.get_id())?,
        neutral.clone(),
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

    // The Hinge: the large neutral trading bazaar at the network's crossroads.
    let _hinge_bazaar = create_station_with_modules(
        dsl,
        StationSize::Large,
        &s.the_hinge,
        &create_sobj(dsl, StellarObjectKinds::Station, &s.the_hinge.get_id())?,
        neutral.clone(),
        "The Hinge Bazaar",
        None,
        Vec2::new(0.0, 0.0),
        0.0,
        vec![create_trading_module()],
    )?;

    // Ore Trench: a Lrak trading exchange feeding off the rare-ore field.
    let _ore_trench_exchange = create_station_with_modules(
        dsl,
        StationSize::Medium,
        &s.ore_trench,
        &create_sobj(dsl, StellarObjectKinds::Station, &s.ore_trench.get_id())?,
        lrak.clone(),
        "Ore Trench Exchange",
        None,
        Vec2::new(613.0, 1337.0),
        0.0,
        vec![create_trading_module()],
    )?;

    // --- Under-construction refinery sites (contribution loop targets) ----
    let _karren_site = create_construction_site(
        dsl,
        StationSize::Medium,
        &s.karrens_reach,
        &create_sobj(dsl, StellarObjectKinds::Station, &s.karrens_reach.get_id())?,
        lrak.clone(),
        "Karren Refinery (Under Construction)",
        Vec2::new(1500.0, 0.0),
        0.0,
        vec![
            ResourceAmount::new(ITEM_IRON_ORE, 150),
            ResourceAmount::new(ITEM_SILICON_ORE, 100),
            ResourceAmount::new(ITEM_CARBON_ORE, 50),
        ],
    )?;

    let _furrow_site = create_construction_site(
        dsl,
        StationSize::Medium,
        &s.iron_furrow,
        &create_sobj(dsl, StellarObjectKinds::Station, &s.iron_furrow.get_id())?,
        rediar.clone(),
        "Iron Furrow Refinery (Under Construction)",
        Vec2::new(-1500.0, 0.0),
        0.0,
        vec![
            ResourceAmount::new(ITEM_IRON_ORE, 200),
            ResourceAmount::new(ITEM_GOLD_ORE, 50),
        ],
    )?;

    Ok(())
}

/// Stamps a station as the faction's capital — the spawn anchor for that
/// faction's new players.
fn stamp_capital(
    dsl: &DSL<'_, ReducerContext>,
    faction_id: &FactionId,
    station: &Station,
) -> Result<(), String> {
    let mut faction = dsl.get_faction_by_id(faction_id)?;
    faction.set_capital_station_id(Some(station.get_id().value()));
    dsl.update_faction_by_id(faction)?;
    Ok(())
}
