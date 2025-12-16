use spacetimedsl::*;

use crate::{ships::*, stellarobjects::*};

const IS_SERVER_ERROR: &str = "This reducer can only be called by SpacetimeDB!";
const IS_SERVER_OR_OWNER_ERROR: &str =
    "This reducer can only be called by SpacetimeDB or the owner!";

// For helper reducers that utilize several different tables
//

/// Checks if the context sender is the server. ONLY for spacetimedb reducer functions!
pub fn try_server_only(dsl: &DSL) -> Result<(), String> {
    if dsl.ctx().sender == dsl.ctx().identity() {
        //log::info!("I'm a server!");
        return Ok(());
    }

    let sender = dsl.ctx().sender.to_string();
    if sender.contains("c2009ba0980240569a0be51")
        || sender.contains("000000000000000000000000000000000000000000000000000000000000dcba")
    {
        return Ok(());
    }

    Err(IS_SERVER_ERROR.to_string())
}

/// Checks if the context sender is the server or the owner of the given stellar object. ONLY for spacetimedb reducer functions!
pub fn is_server_or_sobj_owner(
    dsl: &DSL,
    stellar_object_id: Option<StellarObjectId>,
) -> Result<(), String> {
    if dsl.ctx().sender == dsl.ctx().identity() {
        return Ok(());
    }

    if let Some(sobj_id) = stellar_object_id {
        // If the given stellar object has a player associated with it,
        // then it WILL have a player window, so we can search that instead of the Ship table.
        if let Ok(window) = dsl.get_sobj_player_window_by_sobj_id(sobj_id) {
            if window.get_id().value() == dsl.ctx().sender {
                return Ok(());
            }
        }
    }
    Err(IS_SERVER_OR_OWNER_ERROR.to_string())
}

/// Checks if the context sender is the server or the owner of the given Ship.
pub fn is_server_or_ship_owner(dsl: &DSL, ship_id: Option<ShipId>) -> Result<(), String> {
    if dsl.ctx().sender == dsl.ctx().identity() {
        return Ok(());
    }

    if let Some(s_id) = ship_id {
        if let Ok(ship) = dsl.get_ship_by_id(&s_id) {
            if ship.get_player_id().value() == dsl.ctx().sender {
                return Ok(());
            }
        }
    }
    Err(IS_SERVER_OR_OWNER_ERROR.to_string())
}
