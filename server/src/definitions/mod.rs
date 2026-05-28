use spacetimedb::ReducerContext;
use spacetimedsl::*;

pub mod factions;
pub mod galaxy;
pub mod item_types;
pub mod ship_types;
pub mod station_module_types;

pub fn init(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    factions::init(dsl)?;
    station_module_types::init(dsl)?;
    item_types::init(dsl)?;
    ship_types::init(dsl)?;

    // Init galaxy only AFTER all other definitions (it seeds asteroids, which need ctx.rng()).
    galaxy::init(dsl)?;

    Ok(())
}
