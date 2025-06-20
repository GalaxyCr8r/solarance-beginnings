use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::{dsl};

use crate::types::common::Vec2;

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
  let _belt = dsl.create_star_system_object(&procyon, StarSystemObjectKind::AsteroidBelt, 64., 12., None);

  let a = dsl.create_sector(
    0, &procyon, "Alpha Sector", None, &faction_none, 
    0, 0.9, 0.1, 0.1, 0.1, 
    0.0, -64.0, None)?;
  let b = dsl.create_sector(
    1, &procyon, "Beta Sector", None, &faction_none, 
    0, 0.9, 0.1, 0.1, 0.1, 
    -8.0, -56.0, None)?;
  let c = dsl.create_sector(
    2, &procyon, "Gamma Sector", None, &faction_none, 
    0, 0.9, 0.1, 0.1, 0.1, 
    128.0, 16.0, None)?;

  connect_sectors_with_warpgates(ctx, &a, &b)?;
  connect_sectors_with_warpgates(ctx, &b, &c)?;
  
  dsl.create_asteroid_sector(SectorId::new(0), 1, 3000.0, Some(1000.0))?;
  dsl.create_asteroid_sector(SectorId::new(1), 5, 5000.0, None)?;

  Ok(())
}