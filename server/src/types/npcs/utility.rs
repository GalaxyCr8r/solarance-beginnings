use spacetimedb::ReducerContext;
use spacetimedsl::{dsl, Wrapper};

use crate::types::stellarobjects::{GetStellarObjectRowOptionById, StellarObjectId};

use super::{
    CreateNpcShipControllerRow, GetNpcShipControllerRowOptionById, NpcBehavior, NpcShipController,
    NpcShipControllerId, UpdateNpcShipControllerRowById,
};

/// Create a new NPC ship controller for a stellar object
/// This is a utility function that can be called by other reducers
pub fn create_npc_ship_controller(
    ctx: &ReducerContext,
    stellar_object_id: u64,
    initial_behavior: NpcBehavior,
) -> Result<NpcShipController, String> {
    let dsl = dsl(ctx);

    // Validate that the stellar object exists
    let _stellar_object = dsl.get_stellar_object_by_id(StellarObjectId::new(stellar_object_id))?;

    // Create the NPC ship controller
    let npc_controller = dsl.create_npc_ship_controller(
        StellarObjectId::new(stellar_object_id),
        false, // fire_weapons
        false, // fire_missiles
        None,  // targetted_sobj_id
        initial_behavior.clone(),
    )?;

    spacetimedb::log::info!(
        "Created NPC ship controller {} for stellar object {} with behavior {:?}",
        npc_controller.get_id().value(),
        stellar_object_id,
        initial_behavior
    );

    Ok(npc_controller)
}

/// Set the target for an NPC ship controller
pub fn set_npc_target(
    ctx: &ReducerContext,
    npc_controller_id: u64,
    target_sobj_id: Option<u64>,
) -> Result<(), String> {
    let dsl = dsl(ctx);

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
pub fn set_npc_behavior(
    ctx: &ReducerContext,
    npc_controller_id: u64,
    behavior: NpcBehavior,
) -> Result<(), String> {
    let dsl = dsl(ctx);

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
pub fn trigger_npc_weapon_fire(ctx: &ReducerContext, npc_controller_id: u64) -> Result<(), String> {
    let dsl = dsl(ctx);

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
pub fn trigger_npc_missile_fire(
    ctx: &ReducerContext,
    npc_controller_id: u64,
) -> Result<(), String> {
    let dsl = dsl(ctx);

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
