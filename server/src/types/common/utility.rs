use spacetimedb::ReducerContext;
use spacetimedsl::*;

use crate::{ships::*, stellarobjects::*};

const IS_SERVER_ERROR: &str = "This reducer can only be called by SpacetimeDB!";
const IS_SERVER_OR_OWNER_ERROR: &str =
    "This reducer can only be called by SpacetimeDB or the owner!";

// For helper reducers that utilize several different tables
//

/// Checks if the context sender is the server
pub fn try_server_only(ctx: &ReducerContext) -> Result<(), String> {
    if ctx.sender == ctx.identity() {
        //log::info!("I'm a server!");
        return Ok(());
    }
    if ctx.sender.to_string().contains("c2009ba0980240569a0be51") {
        //log::info!("I'm Karl's desktop!");
        return Ok(());
    }
    //info!("{} is NOT an admin!", ctx.sender.to_string());

    Err(IS_SERVER_ERROR.to_string())
}

/// Panics if the context sender is the server. To be deprecated. (NOT recommended for learn term usage)
pub fn server_only(ctx: &ReducerContext) {
    if let Err(e) = try_server_only(ctx) {
        panic!("{}", e);
    }
}

/// Checks if the context sender is the server or the owner of the given stellar object.
pub fn is_server_or_sobj_owner(
    ctx: &ReducerContext,
    stellar_object_id: Option<StellarObjectId>,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if ctx.sender == ctx.identity() {
        return Ok(());
    }

    if let Some(sobj_id) = stellar_object_id {
        // If the given stellar object has a player associated with it,
        // then it WILL have a player window, so we can search that instead of the Ship table.
        if let Ok(window) = dsl.get_sobj_player_window_by_sobj_id(sobj_id) {
            if window.get_id().value() == ctx.sender {
                return Ok(());
            }
        }
    }
    Err(IS_SERVER_OR_OWNER_ERROR.to_string())
}

/// Checks if the context sender is the server or the owner of the given Ship.
pub fn is_server_or_ship_owner(
    ctx: &ReducerContext,
    ship_id: Option<ShipGlobalId>,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    if ctx.sender == ctx.identity() {
        return Ok(());
    }

    if let Some(s_id) = ship_id {
        if let Ok(ship) = dsl.get_ship_by_id(&s_id) {
            if ship.player_id == ctx.sender {
                return Ok(());
            }
        } else if let Ok(ship) = dsl.get_docked_ship_by_id(&s_id) {
            if ship.player_id == ctx.sender {
                return Ok(());
            }
        }
    }
    Err(IS_SERVER_OR_OWNER_ERROR.to_string())
}
