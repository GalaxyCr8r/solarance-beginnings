use spacetimedb::ReducerContext;
use crate::spacetimedsl::prelude::*;

use crate::{
    logic::stellarobjects::movement::write_ship_movement_snapshot,
    tables::{players::*, ships::*, stellarobjects::*},
};

/// Player-input reducer. Records the latest control flags into the
/// `ShipMovementController` mirror row, then writes one new `MovementState`
/// snapshot on `Ship.movement` with the corresponding linear/angular
/// accelerations. Damping is always-on inside `predict_movement`, so we
/// don't have to schedule a tick for inertia.
#[spacetimedb::reducer]
pub fn update_ship_movement_controller(
    ctx: &ReducerContext,
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let player_id = PlayerId::new(ctx.sender());

    // Mutual exclusion: pressing both directions on an axis cancels out. Doing
    // it here keeps the snapshot writer agnostic of the input convention.
    let (forward, backward) = if forward && backward {
        (false, false)
    } else {
        (forward, backward)
    };
    let (left, right) = if left && right {
        (false, false)
    } else {
        (left, right)
    };

    // No-op early-return: key repeats fire this reducer at OS-level rates;
    // we mustn't emit a fresh snapshot per repeat or the dead-reckoning
    // would jitter.
    let mut controller = dsl.get_ship_movement_controller_by_id(&player_id)?;
    if controller.forward == forward
        && controller.backward == backward
        && controller.left == left
        && controller.right == right
    {
        return Ok(());
    }

    controller.forward = forward;
    controller.backward = backward;
    controller.left = left;
    controller.right = right;
    dsl.update_ship_movement_controller_by_id(controller)?;

    // Look up the player's ship + ship_type without going through the
    // controller's stellar_object_id (which was dropped this phase).
    let ship = dsl
        .get_ships_by_player_id(&player_id)
        .find(|s| *s.get_location() == ShipLocation::Sector)
        .ok_or_else(|| {
            format!(
                "Player {} has no in-sector ship to apply movement input to",
                player_id.value().to_abbreviated_hex()
            )
        })?;
    let ship_type = dsl.get_ship_type_definition_by_id(ship.get_shiptype_id())?;

    // Map (forward, backward) → linear acceleration sign × base_acceleration.
    // Map (left, right) → angular acceleration sign × base_angular_acceleration.
    // Released keys yield 0, so the always-on dampening inside
    // `predict_movement` bleeds the relevant velocity back toward zero.
    let linear_a = match (forward, backward) {
        (true, false) => *ship_type.get_base_acceleration(),
        (false, true) => -*ship_type.get_base_acceleration(),
        _ => 0.0,
    };
    let angular_a = match (left, right) {
        (false, true) => *ship_type.get_base_angular_acceleration(),
        (true, false) => -*ship_type.get_base_angular_acceleration(),
        _ => 0.0,
    };

    write_ship_movement_snapshot(&dsl, &ship.get_id(), |state| {
        state.acceleration = linear_a;
        state.angular_acceleration = angular_a;
    })?;

    Ok(())
}

pub fn initialize_controller_for_player<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player: &PlayerId,
    _sobj: &StellarObject,
) -> Result<(), String> {
    dsl.create_ship_movement_controller(CreateShipMovementController {
        id: player.clone(),
        forward: false,
        backward: false,
        left: false,
        right: false,
    })?;
    Ok(())
}

// The legacy 20 Hz `timer_update_all_ship_movement_controllers` tick was
// retired by the dead-reckoning rewrite — motion is now event-driven
// (snapshot on input change / dock / undock / jumpgate) with damping
// living inside `solarance_shared::predict_movement`.
