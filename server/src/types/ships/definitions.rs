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

    fighters(&dsl)?;

    info!("Ship Defs Loaded: {}", dsl.get_count_of_ship_type_definitions());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn fighters(dsl: &DSL) -> Result<(), String> {
    dsl.create_ship_type_definition(1000,
        "Phalanx", 
        Some("The frontline fightercraft for the Lrak Combine.".into()),
        super::ShipClass::Fighter,
        100, 100, 100,
        150.0, 0.0167, PI / 224.0,
        8,
        3, 1, 1,
        0, 0,
        Some("lc.phalanx".into())
    )?;
    dsl.create_ship_type_definition(1001,
        "Javelin", 
        Some("The frontline fightercraft for the Rediar Federation.".into()),
        super::ShipClass::Fighter,
        150, 50, 125,
        140.0, 0.0167, PI / 256.0,
        8,
        2, 1, 1,
        0, 0,
        Some("rf.javelin".into())
    )?;

    Ok(())
}
