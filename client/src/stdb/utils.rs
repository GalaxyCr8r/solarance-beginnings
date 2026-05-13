use macroquad::{miniquad::date::now, prelude::glam};
use spacetimedb_sdk::{DbContext, Identity, Table};

use crate::server::bindings::*;

/// Render-time pose of a stellar object — a single `(position, rotation, id)`
/// tuple that the per-class draw functions consume regardless of where the
/// pose came from (predicted snapshot, static `position` column, or
/// id⊕time-derived rotation for asteroids).
#[derive(Clone, Copy, Debug)]
pub struct RenderPose {
    pub sobj_id: u64,
    pub pos: glam::Vec2,
    pub rotation_radians: f32,
}

impl RenderPose {
    pub fn to_vec2(&self) -> glam::Vec2 {
        self.pos
    }
}

/// Current wall clock in microseconds since the Unix epoch. Used as the
/// `current_time` argument to `solarance_shared::predict_movement`.
/// Note: clock skew between server and client manifests as jitter in
/// predicted positions; the Phase 10 follow-up adds a `server_offset`
/// estimator to compensate.
pub fn now_unix_micros() -> i64 {
    (now() * 1_000_000.0) as i64
}

/// Builds a `RenderPose` for any stellar object the client knows about,
/// dispatching to the right data source per kind. Returns `None` if the
/// dependent rows haven't been received yet (e.g. asteroid subscribed but
/// row not yet in the cache).
pub fn pose_for_object(
    ctx: &DbConnection,
    object: &StellarObject,
    now_micros: i64,
) -> Option<RenderPose> {
    let db = ctx.db();
    match object.kind {
        StellarObjectKinds::Ship => {
            let ship = db.ship().iter().find(|s| s.sobj_id == object.id)?;
            let movement: solarance_shared::MovementState = (&ship.movement).into();
            let (pos, rot, _, _) = solarance_shared::predict_movement(&movement, now_micros);
            Some(RenderPose {
                sobj_id: object.id,
                pos: glam::Vec2::new(pos.x, pos.y),
                rotation_radians: rot,
            })
        }
        StellarObjectKinds::CargoCrate => {
            let crate_row = db.cargo_crate().sobj_id().find(&object.id)?;
            let movement: solarance_shared::MovementState = (&crate_row.movement).into();
            let (pos, rot, _, _) = solarance_shared::predict_movement(&movement, now_micros);
            Some(RenderPose {
                sobj_id: object.id,
                pos: glam::Vec2::new(pos.x, pos.y),
                rotation_radians: rot,
            })
        }
        StellarObjectKinds::Asteroid => {
            let asteroid = db.asteroid().id().find(&object.id)?;
            // Static position; rotation derived deterministically from
            // id⊕time for cheap visual variety.
            let rotation =
                asteroid_visual_rotation(object.id, now_micros);
            Some(RenderPose {
                sobj_id: object.id,
                pos: glam::Vec2::new(asteroid.position.x, asteroid.position.y),
                rotation_radians: rotation,
            })
        }
        StellarObjectKinds::Station => {
            let station = db.station().sobj_id().find(&object.id)?;
            Some(RenderPose {
                sobj_id: object.id,
                pos: glam::Vec2::new(station.position.x, station.position.y),
                rotation_radians: station.rotation,
            })
        }
        StellarObjectKinds::JumpGate => {
            let gate = db.jump_gate().id().find(&object.id)?;
            Some(RenderPose {
                sobj_id: object.id,
                pos: glam::Vec2::new(gate.position.x, gate.position.y),
                rotation_radians: gate.rotation,
            })
        }
    }
}

/// Predicts the player's current motion snapshot (shared type, ready for
/// `solarance_shared::predict_movement`).
pub fn predicted_player_snapshot(
    ctx: &DbConnection,
) -> Option<(Ship, solarance_shared::MovementState)> {
    let ship = get_player_ship(ctx)?;
    let movement: solarance_shared::MovementState = (&ship.movement).into();
    let now_micros = now_unix_micros();
    let (pos, rot, vel, ang_vel) = solarance_shared::predict_movement(&movement, now_micros);
    Some((
        ship,
        solarance_shared::MovementState {
            pos,
            rotation: rot,
            velocity: vel,
            angular_velocity: ang_vel,
            last_update_time: now_micros,
            ..movement
        },
    ))
}

/// Cheap deterministic rotation for asteroid sprites — driven by id⊕time
/// so each asteroid spins independently without the server storing a
/// per-asteroid angle.
fn asteroid_visual_rotation(asteroid_id: u64, now_micros: i64) -> f32 {
    let seed = asteroid_id as i64 ^ now_micros;
    let seconds = (now_micros as f64) / 1_000_000.0;
    // Per-asteroid offset based on id so they don't all start aligned.
    let offset = ((asteroid_id as f64) * 0.137).rem_euclid(std::f64::consts::TAU);
    let _ = seed;
    ((seconds * 0.25 + offset) as f32).rem_euclid(std::f32::consts::TAU)
}

pub fn get_username(ctx: &DbConnection, id: &Identity) -> String {
    if let Some(player) = ctx.db().player().id().find(id) {
        player.username
    } else {
        id.to_abbreviated_hex().to_string()
    }
}

pub fn get_faction_shortname(ctx: &DbConnection, id: &u32) -> String {
    if let Some(faction) = ctx.db().faction().id().find(id) {
        if let Some(p_id) = faction.parent_id {
            format!(
                "{}, {}",
                get_faction_shortname(ctx, &p_id.value),
                faction.short_name
            )
        } else {
            faction.short_name
        }
    } else {
        "UFX".to_string()
    }
}

pub fn get_sector_name(ctx: &DbConnection, id: &u64) -> String {
    if let Some(sector) = ctx.db().sector().id().find(&id) {
        sector.name
    } else {
        format!("Sector #{}", id)
    }
}

pub fn get_current_player(ctx: &DbConnection) -> Option<Player> {
    get_player(&ctx.db, &ctx.identity())
}

pub fn get_player(db: &RemoteTables, id: &Identity) -> Option<Player> {
    let this = db.player().id().find(id);
    match this {
        Some(p) => Some(p),
        None => None,
    }
}

/// Returns the sobj id of the player's currently-controlled ship. Replaces
/// the old `sobj_player_window`-based lookup — the player owns at most one
/// in-sector ship.
pub fn get_player_sobj_id(ctx: &DbConnection) -> Option<u64> {
    get_player_ship(ctx).map(|s| s.sobj_id)
}

pub fn get_player_ship(ctx: &DbConnection) -> Option<Ship> {
    let identity = ctx.identity();
    ctx.db()
        .ship()
        .iter()
        .find(|s| s.player_id == identity && s.location == ShipLocation::Sector)
}

pub fn get_all_equipped_of_type(
    ctx: &DbConnection,
    ship_id: u64,
    slot_type: EquipmentSlotType,
) -> Vec<ShipEquipmentSlot> {
    let mut equipment = Vec::new();
    for slot in ctx.db().ship_equipment_slot().iter() {
        if slot.ship_id == ship_id {
            if slot.slot_type == slot_type {
                equipment.push(slot);
            }
        }
    }
    equipment
}

pub fn get_player_ship_status(ctx: &DbConnection) -> Option<ShipStatus> {
    if let Some(this) = get_player_ship(ctx) {
        this.status(ctx)
    } else {
        None
    }
}

/// Predicted-forward pose of the player's controlled ship. Returns `None`
/// when the player is docked (no in-sector ship).
pub fn get_player_pose(ctx: &DbConnection) -> Option<RenderPose> {
    let (ship, snapshot) = predicted_player_snapshot(ctx)?;
    Some(RenderPose {
        sobj_id: ship.sobj_id,
        pos: glam::Vec2::new(snapshot.pos.x, snapshot.pos.y),
        rotation_radians: snapshot.rotation,
    })
}

pub fn get_player_transform_vec2(ctx: &DbConnection, default: glam::Vec2) -> glam::Vec2 {
    get_player_pose(ctx).map(|p| p.pos).unwrap_or(default)
}

/// Convenience wrapper around `pose_for_object` for callers that have a
/// raw sobj id. Used in the minimap / status widgets / player input code
/// that previously called the legacy `get_transform`.
pub fn get_transform(ctx: &DbConnection, sobj_id: u64) -> Result<RenderPose, String> {
    let object = ctx
        .db()
        .stellar_object()
        .id()
        .find(&sobj_id)
        .ok_or_else(|| format!("Stellar object #{} not in cache", sobj_id))?;
    pose_for_object(ctx, &object, now_unix_micros())
        .ok_or_else(|| format!("No pose source available for sobj #{}", sobj_id))
}

/// Predicted-forward pose of the player's controlled ship. Returns `None`
/// while the player is docked.
pub fn get_player_transform(ctx: &DbConnection) -> Option<RenderPose> {
    get_player_pose(ctx)
}
