//! Dead-reckoning snapshot helpers.
//!
//! Every call site that touches an entity's motion routes through one of the
//! pairs below. The `get_*` helper predicts forward to `ctx.timestamp` without
//! writing — use it for range / proximity checks. The `write_*` helper does
//! the same prediction, then applies a caller-supplied mutation, re-stamps
//! caps from the underlying definition (so hot edits to `ShipTypeDefinition`
//! / `global_config` take effect on the next input change), and writes the
//! row back.
//!
//! These are the only sanctioned writers of `Ship.movement` /
//! `CargoCrate.movement`. The Phase 6 audit confirms no caller bypasses them.

use solarance_shared::{predict_movement, MovementState, Vec2};
use crate::spacetimedsl::prelude::*;

use crate::tables::{
    asteroids::*, global_config::*, items::*, jumpgates::*, sectors::*, ships::*, stations::*,
    stellarobjects::*,
};

// ── Ships ───────────────────────────────────────────────────────────────────

/// Predicts a ship's current motion forward to `ctx.timestamp` without
/// writing. Use for distance / range / proximity checks (docking, mining,
/// jumpgate proximity).
pub fn get_ship_movement_snapshot<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship_id: &ShipId,
) -> Result<MovementState, String> {
    let ship = dsl.get_ship_by_id(ship_id)?;
    let now = dsl.ctx().timestamp()?.to_micros_since_unix_epoch();
    let (pos, rotation, velocity, angular_velocity) = predict_movement(&ship.movement, now);

    Ok(MovementState {
        pos,
        rotation,
        velocity,
        angular_velocity,
        last_update_time: now,
        ..ship.movement
    })
}

/// Predicts a ship's motion forward, applies `mutate`, re-stamps the caps
/// from `ShipTypeDefinition`, and writes back. The closure must not touch
/// caps (`max_speed`, `max_turn_rate`) — re-stamping is this helper's job.
///
/// Returns the freshly-written movement state for callers that want to inspect
/// it (e.g. for log lines).
pub fn write_ship_movement_snapshot<T, F>(
    dsl: &DSL<T>,
    ship_id: &ShipId,
    mutate: F,
) -> Result<MovementState, String>
where
    T: spacetimedsl::WriteContext,
    F: FnOnce(&mut MovementState),
{
    let mut ship = dsl.get_ship_by_id(ship_id)?;
    let ship_type: ShipTypeDefinition = dsl.get_ship_type_definition_by_id(ship.get_shiptype_id())?;
    let now = dsl.ctx().timestamp()?.to_micros_since_unix_epoch();

    let (pos, rotation, velocity, angular_velocity) = predict_movement(&ship.movement, now);

    let mut next = MovementState {
        pos,
        rotation,
        velocity,
        angular_velocity,
        last_update_time: now,
        // Preserve the existing accelerations — only the closure changes them.
        acceleration: ship.movement.acceleration,
        angular_acceleration: ship.movement.angular_acceleration,
        // Caps are unconditionally re-stamped below; values here don't matter.
        max_speed: 0.0,
        max_turn_rate: 0.0,
    };

    mutate(&mut next);

    // Re-stamp caps from the ship's definition AFTER the mutation so callers
    // can't accidentally override them.
    next.max_speed = *ship_type.get_base_speed();
    next.max_turn_rate = *ship_type.get_base_max_turn_rate();

    ship.movement = next;
    dsl.update_ship_by_id(ship)?;
    Ok(next)
}

/// Atomically transitions a ship to a new sector at `arrival_pos` /
/// `arrival_rotation`, zeroing dynamics. Updates `Ship.sector_id`,
/// `ShipStatus.sector_id`, and the underlying `StellarObject.sector_id` in
/// one helper. Distinct from `write_ship_movement_snapshot` because
/// cross-sector transit has different invariants than same-sector mutation.
pub fn transit_ship_to_sector<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship_id: &ShipId,
    destination_sector_id: &SectorId,
    arrival_pos: Vec2,
    arrival_rotation: f32,
) -> Result<(), String> {
    // Read everything first so a failure on lookup doesn't leave the ship
    // half-transitioned.
    let mut ship: Ship = dsl.get_ship_by_id(ship_id)?;
    let mut status: ShipStatus = dsl.get_ship_status_by_id(ship_id)?;
    let mut sobj = dsl.get_stellar_object_by_id(&ship.get_sobj_id())?;

    ship.set_sector_id(destination_sector_id);
    status.set_sector_id(destination_sector_id);
    sobj.set_sector_id(destination_sector_id);

    dsl.update_ship_by_id(ship)?;
    dsl.update_ship_status_by_id(status)?;
    dsl.update_stellar_object_by_id(sobj)?;

    write_ship_movement_snapshot(dsl, ship_id, |state| {
        state.pos = arrival_pos;
        state.rotation = arrival_rotation;
        state.velocity = 0.0;
        state.angular_velocity = 0.0;
        state.acceleration = 0.0;
        state.angular_acceleration = 0.0;
    })?;

    Ok(())
}

// ── Cargo crates ────────────────────────────────────────────────────────────

/// Predicts a cargo crate's current motion forward to `ctx.timestamp` without
/// writing. Use for pickup-range checks.
pub fn get_cargo_crate_movement_snapshot<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    crate_id: &CargoCrateId,
) -> Result<MovementState, String> {
    let crate_row: CargoCrate = dsl.get_cargo_crate_by_id(crate_id)?;
    let now = dsl.ctx().timestamp()?.to_micros_since_unix_epoch();
    let (pos, rotation, velocity, angular_velocity) = predict_movement(&crate_row.movement, now);

    Ok(MovementState {
        pos,
        rotation,
        velocity,
        angular_velocity,
        last_update_time: now,
        ..crate_row.movement
    })
}

/// Predicts a cargo crate's motion forward, applies `mutate`, re-stamps the
/// `max_turn_rate` cap from `global_config`, and writes back. The crate's
/// `max_speed` is denormalised per-crate (set at jettison from the chosen
/// toss speed) and is intentionally not re-stamped here.
pub fn write_cargo_crate_movement_snapshot<T, F>(
    dsl: &DSL<T>,
    crate_id: &CargoCrateId,
    mutate: F,
) -> Result<MovementState, String>
where
    T: spacetimedsl::WriteContext,
    F: FnOnce(&mut MovementState),
{
    let mut crate_row = dsl.get_cargo_crate_by_id(crate_id)?;
    let config = dsl.get_global_config_by_id(GlobalConfigId::new(0))?;
    let now = dsl.ctx().timestamp()?.to_micros_since_unix_epoch();

    let (pos, rotation, velocity, angular_velocity) = predict_movement(&crate_row.movement, now);

    let mut next = MovementState {
        pos,
        rotation,
        velocity,
        angular_velocity,
        last_update_time: now,
        acceleration: crate_row.movement.acceleration,
        angular_acceleration: crate_row.movement.angular_acceleration,
        max_speed: crate_row.movement.max_speed,
        max_turn_rate: 0.0,
    };

    mutate(&mut next);

    next.max_turn_rate = config.cargo_crate_max_turn_rate;

    crate_row.movement = next;
    dsl.update_cargo_crate_by_id(crate_row)?;
    Ok(next)
}

// ── Convenience for sobj-keyed callers ─────────────────────────────────────

/// Look up a ship by its `StellarObjectId` and predict its motion. Helpful
/// for callers that only have the sobj in hand (e.g. mining range checks).
pub fn get_ship_movement_snapshot_by_sobj<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    sobj_id: &StellarObjectId,
) -> Result<MovementState, String> {
    let ship = dsl
        .get_ships_by_sobj_id(sobj_id)
        .next()
        .ok_or_else(|| format!("No ship found for sobj_id {}", sobj_id.value()))?;
    get_ship_movement_snapshot(dsl, &ship.get_id())
}

/// Returns the current world-space position of any stellar object, dispatching
/// to the right source per kind:
/// * Ship → predict-forward via `Ship.movement`
/// * CargoCrate → predict-forward via `CargoCrate.movement`
/// * Asteroid / Station / JumpGate → static `position` column
///
/// Used by combat range checks, mining proximity checks, dock distance —
/// any code that needs "where is sobj X right now?" without caring what kind it is.
pub fn get_sobj_position<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    sobj_id: &StellarObjectId,
) -> Result<Vec2, String> {
    let sobj = dsl.get_stellar_object_by_id(sobj_id)?;
    use crate::tables::stellarobjects::StellarObjectKinds;
    match sobj.get_kind() {
        StellarObjectKinds::Ship => {
            let snapshot = get_ship_movement_snapshot_by_sobj(dsl, sobj_id)?;
            Ok(snapshot.pos)
        }
        StellarObjectKinds::CargoCrate => {
            let crate_row = dsl
                .get_cargo_crate_by_sobj_id(sobj_id)
                .map_err(|e| format!("Cargo crate lookup for sobj {} failed: {}", sobj_id.value(), e))?;
            let snapshot = get_cargo_crate_movement_snapshot(dsl, &crate_row.get_id())?;
            Ok(snapshot.pos)
        }
        StellarObjectKinds::Asteroid => {
            let asteroid = dsl
                .get_asteroid_by_id(sobj_id)
                .map_err(|e| format!("Asteroid lookup for sobj {} failed: {}", sobj_id.value(), e))?;
            Ok(*asteroid.get_position())
        }
        StellarObjectKinds::Station => {
            let station = dsl
                .get_station_by_sobj_id(sobj_id)
                .map_err(|e| format!("Station lookup for sobj {} failed: {}", sobj_id.value(), e))?;
            Ok(*station.get_position())
        }
        StellarObjectKinds::JumpGate => {
            let gate = dsl
                .get_jump_gate_by_id(sobj_id)
                .map_err(|e| format!("JumpGate lookup for sobj {} failed: {}", sobj_id.value(), e))?;
            Ok(*gate.get_position())
        }
    }
}

/// Like `get_sobj_position` but also returns rotation. For ships and crates
/// the rotation comes from the predicted movement snapshot; for static
/// entities it comes from the dedicated `rotation` column. Asteroids return 0
/// (rotation is derived client-side from id⊕time).
pub fn get_sobj_pose<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    sobj_id: &StellarObjectId,
) -> Result<(Vec2, f32), String> {
    let sobj = dsl.get_stellar_object_by_id(sobj_id)?;
    use crate::tables::stellarobjects::StellarObjectKinds;
    match sobj.get_kind() {
        StellarObjectKinds::Ship => {
            let snapshot = get_ship_movement_snapshot_by_sobj(dsl, sobj_id)?;
            Ok((snapshot.pos, snapshot.rotation))
        }
        StellarObjectKinds::CargoCrate => {
            let crate_row = dsl.get_cargo_crate_by_sobj_id(sobj_id)?;
            let snapshot = get_cargo_crate_movement_snapshot(dsl, &crate_row.get_id())?;
            Ok((snapshot.pos, snapshot.rotation))
        }
        StellarObjectKinds::Asteroid => {
            let asteroid = dsl.get_asteroid_by_id(sobj_id)?;
            Ok((*asteroid.get_position(), 0.0))
        }
        StellarObjectKinds::Station => {
            let station = dsl.get_station_by_sobj_id(sobj_id)?;
            Ok((*station.get_position(), *station.get_rotation()))
        }
        StellarObjectKinds::JumpGate => {
            let gate = dsl.get_jump_gate_by_id(sobj_id)?;
            Ok((*gate.get_position(), *gate.get_rotation()))
        }
    }
}
