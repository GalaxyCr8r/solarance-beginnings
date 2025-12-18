use std::time::Duration;

use spacetimedb::*;
use spacetimedsl::*;

use crate::{tables::ships::*, utility::try_server_only};

#[dsl(plural_name = ship_status_timers)]
#[spacetimedb::table(name = ship_status_timer, scheduled(ship_status_timer_reducer))]
pub struct ShipStatusTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[unique]
    #[use_wrapper(ShipId)]
    /// FK to Ship
    pub ship_id: u64,

    #[use_wrapper(ShipTypeDefinitionId)]
    /// FK to Ship Type
    pub ship_type_id: u32,
}

pub fn create_status_timer_for_ship(
    dsl: &DSL,
    ship_id: &ShipId,
    type_id: &ShipTypeDefinitionId,
) -> Result<ShipStatusTimer, String> {
    let timer = dsl.create_ship_status_timer(
        spacetimedb::ScheduleAt::Interval(Duration::from_millis(500).into()),
        ship_id,
        type_id,
    )?;

    Ok(timer)
}

/// Scheduled reducer that handles ship status updates like shield and energy regeneration.
/// Runs every 500ms to gradually restore shields and energy based on ship type specifications.
#[spacetimedb::reducer]
pub fn ship_status_timer_reducer(
    ctx: &ReducerContext,
    timer: ShipStatusTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    // Get ship rows
    let ship_type = dsl.get_ship_type_definition_by_id(timer.get_ship_type_id())?;
    let mut ship_status = dsl.get_ship_status_by_id(timer.get_ship_id())?;

    // TODO: Grab shield regen from attached shield modules and the current ship type
    if *ship_status.get_shields() < (*ship_type.get_max_shields() as f32) {
        ship_status.set_shields(*ship_status.get_shields() + 0.525175);
    }
    if *ship_status.get_energy() > (*ship_type.get_max_shields() as f32) {
        ship_status.set_shields(*ship_type.get_max_shields() as f32);
    }

    // TODO: Grab energy regen from attached special modules and the current ship type
    if *ship_status.get_energy() < (*ship_type.get_max_energy() as f32) {
        ship_status.set_energy(ship_status.get_energy() + 0.1275);
    }
    if *ship_status.get_energy() > (*ship_type.get_max_energy() as f32) {
        ship_status.set_energy(*ship_type.get_max_energy() as f32);
    }

    dsl.update_ship_status_by_id(ship_status)?;

    Ok(())
}
