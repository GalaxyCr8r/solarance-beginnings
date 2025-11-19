use spacetimedb::ReducerContext;

pub mod factions;
pub mod galaxy;
pub mod item_types;
pub mod ship_types;
pub mod station_module_types;

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    factions::init(ctx)?;
    galaxy::init(ctx)?;
    item_types::init(ctx)?;
    ship_types::init(ctx)?;
    station_module_types::init(ctx)?;

    Ok(())
}
