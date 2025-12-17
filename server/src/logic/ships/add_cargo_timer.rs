use std::time::Duration;

use log::info;
use spacetimedb::{table, ReducerContext};
use spacetimedsl::*;

use crate::{
    logic::ships::cargo::attempt_to_load_cargo_into_ship,
    tables::{items::*, ships::*},
    utility::try_server_only,
};

/// Adds a cargo item to a ship's cargo after a delay. If there isn't room, it creates a cargo crate instead.
#[dsl(plural_name = ship_add_cargo_timers, method(update = true))]
#[spacetimedb::table(name = ship_add_cargo_timer, scheduled(ship_add_cargo_timer_reducer))]
pub struct ShipAddCargoTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[index(btree)]
    #[use_wrapper(ShipId)]
    /// FK to Ship
    pub ship_id: u64,

    #[use_wrapper(ItemDefinitionId)]
    /// FK to Item Definition
    pub item_id: u32,

    pub amount: u16,
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

pub fn create_timer_to_add_cargo_to_ship<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship_id: ShipId,
    item_id: ItemDefinitionId,
    amount: u16,
) -> Result<ShipAddCargoTimer, String> {
    Ok(dsl.create_ship_add_cargo_timer(CreateShipAddCargoTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_secs(1).into()),
        ship_id: ship_id,
        item_id: item_id,
        amount,
    })?)
}

//////////////////////////////////////////////////////////////
// Timer Reducers
//////////////////////////////////////////////////////////////

/// Scheduled reducer that adds mined cargo to a ship's inventory after a delay.
/// If the ship's cargo bay is full, creates a cargo crate in space instead.
#[spacetimedb::reducer]
pub fn ship_add_cargo_timer_reducer(
    ctx: &ReducerContext,
    timer: ShipAddCargoTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    info!(
        "Attempting to add {}x #{:?} to ship id #{:?}",
        timer.amount,
        timer.get_item_id(),
        timer.get_ship_id()
    );

    // Either way, we don't want this to continue.
    dsl.delete_ship_add_cargo_timer_by_id(&timer)?;

    let ship_object = dsl.get_ship_by_id(timer.get_ship_id())?;
    let mut ship_status = dsl.get_ship_status_by_id(timer.get_ship_id())?;

    // Get the item definition
    let item_def = dsl.get_item_definition_by_id(ItemDefinitionId::new(timer.item_id))?;

    // Attempt to load it into the ship
    attempt_to_load_cargo_into_ship(
        ctx,
        &dsl,
        &mut ship_status,
        &ship_object.get_id(),
        &item_def,
        timer.amount,
        true,
    )?;

    Ok(())
}
