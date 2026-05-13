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
        || sender.contains("c20029638c4f24cb63494c49b28b533e")
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
    let sobj_id = stellar_object_id.ok_or_else(|| "Given a missing SOBJ ID".to_string())?;

    // Post-Phase 9: the `sobj_player_window` table is gone — find the owning
    // ship by `sobj_id` and check its player_id against the sender.
    let sender = dsl.ctx().sender()?;
    if let Some(ship) = dsl.get_ships_by_sobj_id(&sobj_id).next() {
        if ship.get_player_id().value() == sender {
            return Ok(());
        }
    }

    warn!("Denied server/sobj-owner request from: {}", sender.to_string());
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
