



use super::{*};

/// Verify the invariants of this class that Rust cannot guarantee due to the database limitations.
/// Should be called after modifying a station.
pub fn verify(ctx: &ReducerContext, station: Station) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Verify the station does not have more modules than it should.
    if dsl.get_station_modules_by_station_id(station.get_id()).count() > station.size.modules() as usize {
        return Err("Too many station modules attached.".to_string());
    }
    
    Ok(())
}