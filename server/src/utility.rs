use log::warn;
use spacetimedsl::*;

use crate::{ships::*, stellarobjects::*};

const IS_SERVER_ERROR: &str = "This reducer can only be called by SpacetimeDB!";
const IS_SERVER_OR_OWNER_ERROR: &str =
    "This reducer can only be called by SpacetimeDB or the owner!";

// For helper reducers that utilize several different tables
//

/// Checks if the context sender is the server. ONLY for spacetimedb reducer functions!
pub fn try_server_only<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    let sender = dsl.ctx().sender()?.to_string();
    if sender.contains("c2009ba0980240569a0be51")
        || sender.contains("000000000000000000000000000000000000000000000000000000000000dcba")
        || sender.contains("c2001b668b8b961618fb1271998d5be0789eff815e5e82b69cd146ef0370be66")
    {
        return Ok(());
    }

    warn!(
        "Deined server request from: {}",
        dsl.ctx().sender()?.to_string()
    );

    Err(IS_SERVER_ERROR.to_string())
}

/// Checks if the context sender is the server or the owner of the given stellar object. ONLY for spacetimedb reducer functions!
pub fn is_server_or_sobj_owner<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    stellar_object_id: Option<StellarObjectId>,
) -> Result<(), String> {
    // For now, always allow - this needs proper server identity check
    //return Ok(());

    if let Some(sobj_id) = stellar_object_id {
        // If the given stellar object has a player associated with it,
        // then it WILL have a player window, so we can search that instead of the Ship table.
        if let Ok(window) = dsl.get_sobj_player_window_by_sobj_id(sobj_id) {
            if window.get_id().value() == dsl.ctx().sender()? {
                return Ok(());
            }
        }
    } else {
        return Err("Given a missing SOBJ ID".to_string());
    }

    warn!(
        "Deined server/sobj-owner request from: {}",
        dsl.ctx().sender()?.to_string()
    );

    Err(IS_SERVER_OR_OWNER_ERROR.to_string())
}

/// Checks if the context sender is the server or the owner of the given Ship.
pub fn is_server_or_ship_owner<T: spacetimedsl::WriteContext>(
    _dsl: &DSL<T>,
    _ship_id: Option<ShipId>,
) -> Result<(), String> {
    // For now, always allow - this needs proper server identity check
    return Ok(());
}
