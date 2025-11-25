use spacetimedb::ReducerContext;
use spacetimedsl::*;

use crate::utility::try_server_only;

use super::utility::{ create_npc_ship_controller, set_npc_behavior, set_npc_target };
use super::{
    DeleteNpcShipControllerRowById,
    GetNpcShipControllerRowOptionById,
    NpcBehavior,
    NpcShipControllerId,
};

/// Create a new NPC ship controller (server-only)
/// This reducer allows the server to create NPC controllers for ships
#[spacetimedb::reducer]
pub fn create_npc_controller(
    ctx: &ReducerContext,
    stellar_object_id: u64,
    initial_behavior: NpcBehavior
) -> Result<(), String> {
    let dsl = dsl(ctx);

    try_server_only(&dsl)?;

    create_npc_ship_controller(&dsl, stellar_object_id, initial_behavior)?;

    Ok(())
}

/// Update NPC behavior (server-only)
/// This reducer allows the server to change NPC AI behavior
#[spacetimedb::reducer]
pub fn update_npc_behavior(
    ctx: &ReducerContext,
    npc_controller_id: u64,
    new_behavior: NpcBehavior
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    set_npc_behavior(&dsl, npc_controller_id, new_behavior)?;

    Ok(())
}

/// Set NPC target (server-only)
/// This reducer allows the server to set targets for NPCs
#[spacetimedb::reducer]
pub fn set_npc_controller_target(
    ctx: &ReducerContext,
    npc_controller_id: u64,
    target_sobj_id: Option<u64>
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(dsl)?;

    set_npc_target(dsl, npc_controller_id, target_sobj_id)?;

    Ok(())
}

/// Remove an NPC ship controller (server-only)
/// This reducer allows the server to clean up NPC controllers
#[spacetimedb::reducer]
pub fn remove_npc_controller(ctx: &ReducerContext, npc_controller_id: u64) -> Result<(), String> {
    let dsl = dsl(ctx);

    try_server_only(dsl)?;
    let npc_controller = dsl.get_npc_ship_controller_by_id(
        NpcShipControllerId::new(npc_controller_id)
    )?;

    spacetimedb::log::info!(
        "Removing NPC controller {} for stellar object {}",
        npc_controller_id,
        npc_controller.get_stellar_object_id().value()
    );

    dsl.delete_npc_ship_controller_by_id(NpcShipControllerId::new(npc_controller_id))?;

    Ok(())
}
