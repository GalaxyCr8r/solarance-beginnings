use spacetimedb::ReducerContext;
use spacetimedsl::*;

use crate::tables::stellarobjects::*;
use crate::utility::try_server_only;

use crate::tables::npcs::*;

/// Create a new NPC ship controller (server-only)
/// This reducer allows the server to create NPC controllers for ships
#[spacetimedb::reducer]
pub fn create_npc_controller(
    ctx: &ReducerContext,
    stellar_object_id: u64,
    initial_behavior: NpcBehavior,
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
    new_behavior: NpcBehavior,
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
    target_sobj_id: Option<u64>,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    set_npc_target(&dsl, npc_controller_id, target_sobj_id)?;

    Ok(())
}

/// Remove an NPC ship controller (server-only)
/// This reducer allows the server to clean up NPC controllers
#[spacetimedb::reducer]
pub fn remove_npc_controller(ctx: &ReducerContext, npc_controller_id: u64) -> Result<(), String> {
    let dsl = dsl(ctx);

    try_server_only(&dsl)?;
    let npc_controller =
        dsl.get_npc_ship_controller_by_id(NpcShipControllerId::new(npc_controller_id))?;

    spacetimedb::log::info!(
        "Removing NPC controller {} for stellar object {}",
        npc_controller_id,
        npc_controller.get_stellar_object_id().value()
    );

    dsl.delete_npc_ship_controller_by_id(NpcShipControllerId::new(npc_controller_id))?;

    Ok(())
}

/// Create a new NPC ship controller for a stellar object
/// This is a utility function that can be called by other reducers
pub fn create_npc_ship_controller<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    stellar_object_id: u64,
    initial_behavior: NpcBehavior,
) -> Result<NpcShipController, String> {
    // Validate that the stellar object exists
    let _stellar_object = dsl.get_stellar_object_by_id(StellarObjectId::new(stellar_object_id))?;

    // Create the NPC ship controller
    let npc_controller = dsl.create_npc_ship_controller(CreateNpcShipController {
        stellar_object_id: StellarObjectId::new(stellar_object_id),
        fire_weapons: false,
        fire_missiles: false,
        targetted_sobj_id: None,
        ai_behavior: initial_behavior.clone(),
    })?;

    spacetimedb::log::info!(
        "Created NPC ship controller {} for stellar object {} with behavior {:?}",
        npc_controller.get_id().value(),
        stellar_object_id,
        initial_behavior
    );

    Ok(npc_controller)
}

/// Set the target for an NPC ship controller
pub fn set_npc_target<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    npc_controller_id: u64,
    target_sobj_id: Option<u64>,
) -> Result<(), String> {
    let mut npc_controller =
        dsl.get_npc_ship_controller_by_id(NpcShipControllerId::new(npc_controller_id))?;

    // Validate target exists if provided
    if let Some(target_id) = target_sobj_id {
        let _target_sobj = dsl.get_stellar_object_by_id(StellarObjectId::new(target_id))?;
    }

    npc_controller.set_targetted_sobj_id(target_sobj_id);
    dsl.update_npc_ship_controller_by_id(npc_controller)?;

    spacetimedb::log::info!(
        "Set NPC controller {} target to {:?}",
        npc_controller_id,
        target_sobj_id
    );

    Ok(())
}

/// Set the behavior for an NPC ship controller
pub fn set_npc_behavior<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    npc_controller_id: u64,
    behavior: NpcBehavior,
) -> Result<(), String> {
    let mut npc_controller =
        dsl.get_npc_ship_controller_by_id(NpcShipControllerId::new(npc_controller_id))?;
    npc_controller.set_ai_behavior(behavior.clone());
    dsl.update_npc_ship_controller_by_id(npc_controller)?;

    spacetimedb::log::info!(
        "Set NPC controller {} behavior to {:?}",
        npc_controller_id,
        behavior
    );

    Ok(())
}

/// Trigger weapon firing for an NPC ship controller
pub fn trigger_npc_weapon_fire<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    npc_controller_id: u64,
) -> Result<(), String> {
    let mut npc_controller =
        dsl.get_npc_ship_controller_by_id(NpcShipControllerId::new(npc_controller_id))?;
    npc_controller.set_fire_weapons(true);
    dsl.update_npc_ship_controller_by_id(npc_controller)?;

    spacetimedb::log::info!(
        "Triggered weapon fire for NPC controller {}",
        npc_controller_id
    );

    Ok(())
}

/// Trigger missile firing for an NPC ship controller
pub fn trigger_npc_missile_fire<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    npc_controller_id: u64,
) -> Result<(), String> {
    let mut npc_controller =
        dsl.get_npc_ship_controller_by_id(NpcShipControllerId::new(npc_controller_id))?;
    npc_controller.set_fire_missiles(true);
    dsl.update_npc_ship_controller_by_id(npc_controller)?;

    spacetimedb::log::info!(
        "Triggered missile fire for NPC controller {}",
        npc_controller_id
    );

    Ok(())
}
