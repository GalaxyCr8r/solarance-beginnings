use log::info;
use spacetimedb::*;
use crate::spacetimedsl::prelude::*;

use crate::{
    logic::ships::add_cargo_timer::*,
    tables::{items::*, ships::*, stellarobjects::*},
    utility::try_server_only,
};

// Creates a timer to try to add the cargo to the ship if there looks like there will be enough space for it.
pub fn attempt_to_pickup_cargo_crate<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    cargo_crate: &CargoCrate,
    item_def: &ItemDefinition,
    ship_status: &ShipStatus,
) -> Result<(), String> {
    if ship_status.get_sector_id() != cargo_crate.get_current_sector_id() {
        return Err(format!(
            "Ship {} isn't in the same sector as cargo crate {}!",
            ship_status.get_id(),
            cargo_crate.get_id()
        ));
    }

    if item_def.can_any_of_this_fit_inside_this_ship(&ship_status) {
        match create_timer_to_add_cargo_to_ship(
            // Do the actual thing
            dsl,
            ship_status.get_id(),
            item_def.get_id(),
            *cargo_crate.get_quantity(),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "ERROR {} : Ship {:?} could not fit {}x #{:?} items",
                e,
                ship_status.get_id(),
                *cargo_crate.get_quantity(),
                item_def.get_id()
            )),
        }
    } else {
        Err(format!(
            "Ship {:?} could not fit {}x #{:?} items",
            ship_status.get_id(),
            *cargo_crate.get_quantity(),
            item_def.get_id()
        ))
    }
}

// ── Despawn sweeper ─────────────────────────────────────────────────────────

/// Periodic sweeper that removes expired cargo crates. Replaces the per-crate
/// despawn check that used to ride on the (now-removed) 20 Hz transform tick.
#[spacetimedsl::dsl(plural_name = cargo_crate_despawn_sweeper_timers, method(update = false))]
#[spacetimedb::table(
    accessor = cargo_crate_despawn_sweeper_timer,
    scheduled(cargo_crate_despawn_sweeper)
)]
pub struct CargoCrateDespawnSweeperTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

/// Iterates every cargo crate and deletes any past its `despawn_ts`. Cheap
/// even at thousands of crates because it's a single table scan every 30
/// minutes — no per-crate timer rows required.
#[spacetimedb::reducer]
pub fn cargo_crate_despawn_sweeper(
    ctx: &ReducerContext,
    _timer: CargoCrateDespawnSweeperTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    let now = ctx.timestamp;
    let mut swept = 0u32;
    for crate_row in dsl.get_all_cargo_crates() {
        if let Some(despawn_ts) = crate_row.get_despawn_ts() {
            if *despawn_ts < now {
                // Deleting the underlying StellarObject cascades to the
                // CargoCrate row via the `on_delete = Delete` FK.
                let sobj_id = crate_row.get_sobj_id();
                if let Err(e) = dsl.delete_stellar_object_by_id(&sobj_id) {
                    info!(
                        "cargo_crate_despawn_sweeper: failed to delete sobj #{} for expired crate #{}: {}",
                        sobj_id.value(),
                        crate_row.get_id().value(),
                        e
                    );
                    continue;
                }
                swept += 1;
            }
        }
    }
    if swept > 0 {
        info!("cargo_crate_despawn_sweeper: deleted {swept} expired crates");
    }
    Ok(())
}
