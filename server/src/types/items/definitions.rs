use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::{dsl, DSL};

use super::*;

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    commodity_definitions(&dsl)?;

    info!("Item Defs Loaded: {}", dsl.get_count_of_item_definitions());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn commodity_definitions(dsl: &DSL) -> Result<(), String> {
    // Energy Cells
    dsl.create_item_definition(
        1000,
        "Energy Cell",
        Some("Energy Cells are the unified energy storage used throughout the whole known universe. The unification of the energy storage specifications allows for interstellar trading with this most basic of all products.".into()),
        ItemCategory::Commodity,
        20,
        1,
        None
    )?;

    // Raw Ore
    dsl.create_item_definition(
        1001,
        "Raw Ore",
        Some("Raw ore from a variety of sources. It's a common mineral that all metal components need.".into()),
        ItemCategory::Commodity,
        100,
        8,
        None
    )?;

    // Silicon Ore
    dsl.create_item_definition(
        1002,
        "Silicon Ore",
        Some("Silicon ore to be processed. Used to create microchips and other advanced goods.".into()),
        ItemCategory::Commodity,
        100,
        8,
        None
    )?;

    // Space "Fuel"
    dsl.create_item_definition(
        1003,
        "Space Fuel",
        Some("Commonly called 'Space Fuel'. It's actually a variant of vodka using common supplies found in spaceships to make. Usually illegal due to its propensity to turn the users blind.".into()),
        ItemCategory::Commodity,
        50,
        1,
        None
    )?;


    Ok(())
}