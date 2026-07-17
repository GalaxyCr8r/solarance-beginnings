use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use solarance_shared::Vec2;

use crate::{
    definitions::{
        factions::{
            FACTION_FACTIONLESS, FACTION_INDEPENDENT_WORLDS_ALLIANCE, FACTION_LRAK_COMBINE, FACTION_REDIAR_FEDERATION
        },
        item_types::{
            ITEM_CARBON_ORE, ITEM_GOLD_ORE, ITEM_ICE_ORE, ITEM_IRON_ORE, ITEM_SILICON_ORE,
            ITEM_URANIUM_ORE, ITEM_VIVEIUM_ORE,
        },
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
    let iwa = FactionId::new(FACTION_INDEPENDENT_WORLDS_ALLIANCE);

    let procyon = create_procyon_star_system(dsl, &lrak)?;
    let procyon_sectors = create_procyon_sectors(dsl, &procyon, &lrak, &rediar, &neutral)?;

    setup_sector_connections(dsl, &procyon_sectors)?;
    populate_sectors_with_asteroids(dsl, &procyon_sectors)?;
    create_sector_stations(dsl, &procyon_sectors, &lrak, &rediar, &iwa)?;
    place_sector_nebulae(dsl, &procyon_sectors)?;

    // Omicron — the second star system (#121). One cross-system gate out of
    // The Hinge exercises every multi-system code path (system-map filtering,
    // "[System] Sector" gate labels, cross-system travel).
    let omicron_sectors = seed_omicron_system(dsl, &neutral)?;
    connect_sectors_with_warpgates(dsl, &procyon_sectors.the_hinge, &omicron_sectors.dark_sun)?;

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

/// Stocks the mining sectors with asteroid fields. Each field can declare an
/// explicit per-sector ore composition (`ore_weights`); sectors left with an
/// empty composition fall back to the global rarity-skewed distribution, so
/// `sparseness`/`rarity` still drive density and rare-ore skew there.
fn populate_sectors_with_asteroids(
    dsl: &DSL<'_, ReducerContext>,
    s: &ProcyonSectors,
) -> Result<(), String> {
    // Relative ore weights within a sector — values need not sum to 100.
    let ore = |pairs: &[(u32, u16)]| -> Vec<OreWeight> {
        pairs
            .iter()
            .map(|(item_id, weight)| OreWeight {
                item_id: *item_id,
                weight: *weight,
            })
            .collect()
    };

    let field = |sector: &Sector,
                 sparseness: u8,
                 rarity: u8,
                 cluster_extent: f32,
                 cluster_inner: Option<f32>,
                 ore_weights: Vec<OreWeight>|
     -> Result<(), String> {
        let created = dsl.create_asteroid_sector(CreateAsteroidSector {
            id: sector.get_id(),
            sparseness,
            rarity,
            cluster_extent,
            cluster_inner,
            ore_weights,
        })?;
        fill_asteroid_sector(dsl, &created)
    };

    // Empty composition → global default distribution (rarity-skewed).
    field(&s.tarols_belt, 2, 25, 3000.0, Some(1000.0), vec![])?; // moderate, mixed (default mix)
    field(
        &s.ore_trench,
        5,
        50,
        5000.0,
        None,
        ore(&[
            (ITEM_URANIUM_ORE, 30),
            (ITEM_VIVEIUM_ORE, 25),
            (ITEM_GOLD_ORE, 20),
            (ITEM_IRON_ORE, 25),
        ]),
    )?; // dense, rare-ore rich
    field(
        &s.quiet_belt,
        6,
        70,
        5000.0,
        None,
        ore(&[
            (ITEM_GOLD_ORE, 30),
            (ITEM_SILICON_ORE, 30),
            (ITEM_VIVEIUM_ORE, 20),
            (ITEM_ICE_ORE, 20),
        ]),
    )?; // high yield
    field(
        &s.iron_furrow,
        4,
        10,
        4000.0,
        None,
        ore(&[
            (ITEM_IRON_ORE, 70),
            (ITEM_SILICON_ORE, 20),
            (ITEM_GOLD_ORE, 10),
        ]),
    )?; // iron-heavy, common
    field(
        &s.karrens_reach,
        3,
        40,
        3500.0,
        Some(800.0),
        ore(&[
            (ITEM_CARBON_ORE, 50),
            (ITEM_ICE_ORE, 30),
            (ITEM_IRON_ORE, 20),
        ]),
    )?; // carbon-bearing

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

/// Seeds decorative in-sector nebulae (#107). The Hinge sits inside the
/// system's nebula belt (orbit_au 12.0) and gets significant coverage;
/// Lrakhold and Echo Bay each get a single wisp. Pure render flavor —
/// nothing reads these rows server-side.
fn place_sector_nebulae(
    dsl: &DSL<'_, ReducerContext>,
    s: &ProcyonSectors,
) -> Result<(), String> {
    let nebula = |sector: &Sector,
                  x: f32,
                  y: f32,
                  gfx_key: &str,
                  scale: f32,
                  rotation_deg: f32,
                  tint: u32|
     -> Result<(), String> {
        dsl.create_sector_nebula(CreateSectorNebula {
            sector_id: sector.get_id(),
            position: Vec2::new(x, y),
            gfx_key: gfx_key.to_string(),
            scale,
            rotation_radians: rotation_deg.to_radians(),
            tint,
        })
        .map(|_| ())
        .map_err(|e| e.to_string())
    };

    // The Hinge — the only sector within the nebula belt; blanket it.
    nebula(&s.the_hinge, -2600.0, 1900.0, "nebula.1", 6.0, 20.0, 0xFFFFFFA0)?;
    nebula(&s.the_hinge, 1400.0, -2300.0, "nebula.1", 5.0, 140.0, 0xE0E8FF90)?;
    nebula(&s.the_hinge, 3300.0, 1000.0, "nebula.1", 4.0, 275.0, 0xFFFFFF80)?;
    nebula(&s.the_hinge, -900.0, -3400.0, "nebula.1", 4.5, 65.0, 0xD8E0FF88)?;
    nebula(&s.the_hinge, 400.0, 2900.0, "nebula.1", 3.5, 200.0, 0xFFFFFF70)?;

    // Single wisps on the belt-adjacent capitals.
    nebula(&s.lrakhold, 2800.0, -1600.0, "nebula.7", 4.0, 310.0, 0xFFFFFF90)?;
    nebula(&s.echo_bay, -2200.0, 2400.0, "nebula.6", 4.0, 45.0, 0xFFFFFF90)?;

    Ok(())
}

/// The Omicron sectors, mirrored from `ProcyonSectors` so future wiring reads
/// the same way. Sector IDs continue after Procyon's 0..9.
struct OmicronSectors {
    dark_sun: Sector, // 10 — factionless, the lightless inner sector
    ashfall: Sector,  // 11 — factionless, dim outer ember
}

/// Seeds the Omicron star system (#121): a dim M-dwarf neighbor of Procyon
/// holding two factionless frontier sectors, wired to each other by a normal
/// gate. Kept as structured as the Procyon seed so multi-system testing
/// (map filtering, cross-system labels, travel) has a real second system to
/// lean on. The cross-system gate (Dark Sun <-> The Hinge) is wired by
/// `demo_sectors`, next to the rest of the gate network.
fn seed_omicron_system(
    dsl: &DSL<'_, ReducerContext>,
    neutral: &FactionId,
) -> Result<OmicronSectors, String> {
    let omicron = dsl.create_star_system(CreateStarSystem {
        name: "Omicron".to_string(),
        map_coordinates: Vec2::new(21.0, 34.0),
        spectral: SpectralKind::M,
        luminosity: 6,
        controlling_faction_id: neutral.clone(),
    })?;

    let _star = dsl.create_star_system_object(CreateStarSystemObject {
        system_id: omicron.get_id(),
        kind: StarSystemObjectKind::Star,
        orbit_au: 0.0,
        rotation_or_width_km: 0.0,
        gfx_key: Some("star.2".to_string()),
    });
    let _planet = dsl.create_star_system_object(CreateStarSystemObject {
        system_id: omicron.get_id(),
        kind: StarSystemObjectKind::Planet,
        orbit_au: 60.0,
        rotation_or_width_km: (210f32).to_radians(),
        gfx_key: Some("planet.3".to_string()),
    });
    let _nebbelt = dsl.create_star_system_object(CreateStarSystemObject {
        system_id: omicron.get_id(),
        kind: StarSystemObjectKind::NebulaBelt,
        orbit_au: 30.0,
        rotation_or_width_km: 14.0,
        gfx_key: None,
    });

    // Same one-line-per-sector helper shape as `create_procyon_sectors`,
    // plus a description so the map details panel's description path has
    // seeded data to render.
    let mk = |id: u64,
              name: &str,
              description: Option<&str>,
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
            system_id: omicron.get_id(),
            name: name.to_string(),
            description: description.map(str::to_string),
            controlling_faction_id: neutral.clone(),
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

    let dark_sun = mk(
        10,
        "Dark Sun",
        Some("Omicron's failing ember barely reaches this sector. Sensor returns are unreliable, and nobody polices what they can't see."),
        1, 0.1, 0.6, 0.3, 0.3, 0.0, 0.0,
    )?;
    let ashfall = mk(11, "Ashfall", None, 1, 0.2, 0.3, 0.2, 0.4, 45.0, 20.0)?;

    connect_sectors_with_warpgates(dsl, &dark_sun, &ashfall)?;

    Ok(OmicronSectors { dark_sun, ashfall })
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
