use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::{dsl};

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

  let a = dsl.create_sector(0, "Alpha Sector", None, 0.0, -8.0, None)?;
  let b = dsl.create_sector(1, "Beta Sector", None, 0.0, 32.0, None)?;
  let c = dsl.create_sector(2, "Gamma Sector", None, 128.0, 16.0, None)?;

  connect_sectors_with_warpgates(ctx, &a, &b)?;
  connect_sectors_with_warpgates(ctx, &b, &c)?;
  
  dsl.create_asteroid_sector(SectorId::new(0), 1, 3000.0, Some(1000.0))?;
  dsl.create_asteroid_sector(SectorId::new(1), 5, 5000.0, None)?;

  Ok(())
}