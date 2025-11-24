use spacetimedb::ReducerContext;
use spacetimedsl::*;

pub mod factions;
pub mod galaxy;
pub mod item_types;
pub mod ship_types;
pub mod station_module_types;

pub fn init(dsl: &DSL) -> Result<(), String> {
    factions::init(dsl)?;
    galaxy::init(dsl)?;
    item_types::init(dsl)?;
    ship_types::init(dsl)?;
    station_module_types::init(dsl)?;

    Ok(())
}
