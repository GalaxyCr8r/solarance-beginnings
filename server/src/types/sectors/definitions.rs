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

  dsl.create_sector(0, "Newarth", Some("Homeworld of all humans.".to_string()), 0.0, 0.0, None)?;

  Ok(())
}