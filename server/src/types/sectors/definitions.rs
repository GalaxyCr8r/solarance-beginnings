use std::f32::consts::PI;

use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::{dsl, DSL};

use crate::types::ships::GetCountOfShipTypeDefinitionRows;

use super::CreateShipTypeDefinitionRow;

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    demo_sectors(&dsl)?;

    info!("Sectors Loaded: {}", dsl.getsector());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////