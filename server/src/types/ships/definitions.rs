use std::f32::consts::PI;

use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::{dsl, DSL};

use crate::types::ships::*;

use super::CreateShipTypeDefinitionRow;

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    fighters(&dsl)?;

    info!(
        "Ship Defs Loaded: {}",
        dsl.count_of_all_ship_type_definitions()
    );
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn fighters(dsl: &DSL) -> Result<(), String> {
    dsl.create_ship_type_definition(
        1000,
        "Phalanx",
        Some("The frontline fightercraft for the Lrak Combine.".into()),
        super::ShipClass::Fighter,
        100,
        100,
        100,
        50.0,
        0.167,
        PI / 224.0,
        8,
        3,
        0,
        0,
        0,
        1,
        1,
        0,
        1,
        41, // sprite_width
        51, // sprite_height
        Some("lc.phalanx".into()),
    )?;
    dsl.create_ship_type_definition(
        1001,
        "Column",
        Some(
            "A workhorse corvette. This chunky design has been in use for hundreds of years by all factions.".into()
        ),
        super::ShipClass::Shuttle,
        500,
        300,
        200,
        45.0,
        0.117,
        PI / 365.0,
        64,
        2,
        0,
        0,
        0,
        2,
        2,
        1,
        3,
        64,  // sprite_width
        64,  // sprite_height
        Some("lc.column".into())
    )?;
    dsl.create_ship_type_definition(
        1011,
        "Javelin",
        Some("The frontline fightercraft for the Rediar Federation.".into()),
        super::ShipClass::Fighter,
        150,
        50,
        125,
        35.0,
        0.167,
        PI / 256.0,
        8,
        2,
        0,
        0,
        0,
        1,
        1,
        0,
        0,
        46, // sprite_width
        29, // sprite_height
        Some("rf.javelin".into()),
    )?;

    Ok(())
}
