use std::f32::consts::PI;

use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::{dsl};

use crate::types::{common::Vec2, stations::CreateStationRow, stellarobjects::{utility::create_sobj_internal, StellarObjectTransformInternal}};

use super::*;

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    demo_sectors(ctx)?;

    info!("Sectors Loaded: {}", dsl.get_all_sectors().count());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn demo_sectors(ctx: &ReducerContext) -> Result<(), String> {
  let dsl = dsl(ctx);

  let faction_none = FactionId::new(0);

  let procyon = dsl.create_star_system("Procyon", Vec2::new(13.,37.), SpectralKind::G, 5, &faction_none)?;

  let _star = dsl.create_star_system_object(&procyon, StarSystemObjectKind::Star, 0., 0., Some("star.1".to_string()));
  let _planet1 = dsl.create_star_system_object(&procyon, StarSystemObjectKind::Planet, 128., 0., None);
  let _planet2 = dsl.create_star_system_object(&procyon, StarSystemObjectKind::Planet, -24., -PI, None);
  let _moon = dsl.create_star_system_object(&procyon, StarSystemObjectKind::Planet, 128., 24., None);
  let _astbelt = dsl.create_star_system_object(&procyon, StarSystemObjectKind::AsteroidBelt, 48., 12., None);
  let _nebbelt = dsl.create_star_system_object(&procyon, StarSystemObjectKind::NebulaBelt, 12., 8., None);

  let a = dsl.create_sector(
    0, &procyon, "Alpha Sector", None, &faction_none, 
    0, 0.9, 0.1, 0.1, 0.1, 
    8.0, -72.0, None)?;
  let b = dsl.create_sector(
    1, &procyon, "Beta Sector", None, &faction_none, 
    0, 0.9, 0.1, 0.1, 0.1, 
    -8.0, -24.0, None)?;
  let c = dsl.create_sector(
    2, &procyon, "Gamma Sector", None, &faction_none, 
    0, 0.9, 0.1, 0.1, 0.1, 
    128.0, 16.0, None)?;

  connect_sectors_with_warpgates(ctx, &a, &b)?;
  connect_sectors_with_warpgates(ctx, &b, &c)?;
  
  dsl.create_asteroid_sector(&a, 1, 3000.0, Some(1000.0))?;
  dsl.create_asteroid_sector(&b, 5, 5000.0, None)?;

  dsl.create_station(crate::types::stations::StationSize::Medium,
    &b,
    &create_sobj_internal(ctx,
      crate::types::stellarobjects::StellarObjectKinds::Station,
      &b.get_id(),
      StellarObjectTransformInternal::default().from_xy(613., 1337.)
    )?,
    FactionId::new(0),
    "Shining Beacon Station",
    None)?;

  dsl.create_station(crate::types::stations::StationSize::Outpost,
    &a,
    &create_sobj_internal(ctx,
      crate::types::stellarobjects::StellarObjectKinds::Station,
      &a.get_id(),
      StellarObjectTransformInternal::default()
    )?,
    FactionId::new(0),
    "Tarol Station",
    None)?;

  dsl.create_station(crate::types::stations::StationSize::Capital,
    &c,
    &create_sobj_internal(ctx,
      crate::types::stellarobjects::StellarObjectKinds::Station,
      &c.get_id(),
      StellarObjectTransformInternal::default().from_xy(455., -1337.)
    )?,
    FactionId::new(0),
    "Homeworld Station",
    None)?;

  Ok(())
}