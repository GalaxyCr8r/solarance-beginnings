use crate::{tables::players::PlayerId, utility::try_server_only};
use log::info;
use spacetimedb::{table, Identity, ReducerContext, SpacetimeType, Timestamp};
use spacetimedsl::*;

use crate::tables::global_config::*;
use crate::tables::ships::*;
use crate::tables::stellarobjects::{CreateSobjPlayerWindow, *};

#[dsl(plural_name = player_windows_timers, method(update = true))]
#[spacetimedb::table(name = player_windows_timer, scheduled(recalculate_player_windows))]
pub struct PlayerWindowsTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

/// Creates a viewing window for a player around a specific stellar object.
/// This defines the area of space that the player can see and interact with.
#[spacetimedb::reducer]
pub fn create_sobj_player_window_for(
    ctx: &ReducerContext,
    identity: Identity,
    sobj_id: StellarObjectId,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let pid = PlayerId::new(identity);

    if dsl.get_sobj_player_window_by_id(&pid).is_ok() {
        return Err("Player Window already exists".to_string());
    }

    dsl.create_sobj_player_window(CreateSobjPlayerWindow {
        id: pid,
        sobj_id: sobj_id.clone(),
        window: 4000.0,
        margin: 2000.0,
        tl_x: -2000.0,
        tl_y: -2000.0,
        br_x: 2000.0,
        br_y: 2000.0,
    })?;
    info!(
        "Created player window for {} and object #{}!",
        identity.to_abbreviated_hex().to_string(),
        sobj_id.value()
    );
    Ok(())
}

/// Scheduled reducer that updates player viewing windows based on ship movement.
/// Runs every 750ms to recalculate viewing boundaries when players move near window margins.
/// Only processes if there are active players connected to optimize performance.
#[spacetimedb::reducer]
pub fn recalculate_player_windows(
    ctx: &ReducerContext,
    _timer: PlayerWindowsTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    try_server_only(&dsl)?;

    // Bail out ASAP if there's no players connected.
    if !global_config_any_active_players(&dsl) {
        return Ok(());
    }

    for window in dsl.get_all_sobj_player_windows() {
        if let Some(ship_obj) = dsl.get_ships_by_player_id(&window.get_id()).next() {
            let transform = dsl.get_sobj_internal_transform_by_id(&ship_obj.get_sobj_id())?;
            // Check to see if the player has moved too close to window's margin and recalculate the window if needed.
            if *transform.get_x() < *window.get_tl_x() + *window.get_margin()
                || *transform.get_x() > *window.get_br_x() - *window.get_margin()
                || *transform.get_y() < *window.get_tl_y() + *window.get_margin()
                || *transform.get_y() > *window.get_br_y() - *window.get_margin()
            {
                let mut new_window = window.clone();
                new_window.set_tl_x(transform.get_x() - window.get_window());
                new_window.set_tl_y(transform.get_y() - window.get_window());
                new_window.set_br_x(transform.get_x() + window.get_window());
                new_window.set_br_y(transform.get_y() + window.get_window());
                dsl.update_sobj_player_window_by_id(new_window)?;
                //info!("Recalcuating window for player stellar obj #{}: [({}, {}) ({}, {})]", player.sobj_id, result.tl_x, result.tl_y, result.br_x, result.br_y);
            }
        }
    }
    Ok(())
}
