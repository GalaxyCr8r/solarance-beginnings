use std::fmt::{ self, Debug };

use spacetimedb_sdk::*;

use crate::server::bindings::*;

// ── Bindings ↔ shared physics boundary ─────────────────────────────────────
//
// `spacetime generate` emits a parallel `bindings::Vec2` / `bindings::MovementState`
// pair on the client — structurally identical to the server's
// `solarance_shared::{Vec2, MovementState}` but *different Rust types*. There
// is no way around that duplication: generated bindings can't be mapped onto
// a pre-existing Rust type. The `From` impls below are the only place where
// the two universes meet — call sites convert at the boundary and then talk
// shared types from there on.

impl From<&Vec2> for solarance_shared::Vec2 {
    fn from(v: &Vec2) -> Self {
        solarance_shared::Vec2 { x: v.x, y: v.y }
    }
}

impl From<Vec2> for solarance_shared::Vec2 {
    fn from(v: Vec2) -> Self {
        solarance_shared::Vec2 { x: v.x, y: v.y }
    }
}

impl From<&MovementState> for solarance_shared::MovementState {
    fn from(m: &MovementState) -> Self {
        solarance_shared::MovementState {
            pos: (&m.pos).into(),
            velocity: m.velocity,
            rotation: m.rotation,
            angular_velocity: m.angular_velocity,
            last_update_time: m.last_update_time,
            acceleration: m.acceleration,
            angular_acceleration: m.angular_acceleration,
            max_speed: m.max_speed,
            max_turn_rate: m.max_turn_rate,
        }
    }
}

impl From<MovementState> for solarance_shared::MovementState {
    fn from(m: MovementState) -> Self {
        (&m).into()
    }
}

/// Impls ///

// Legacy StellarObjectVelocity / StellarObjectTransformHiRes /
// StellarObjectTransformLowRes impls were removed: the server-side tables
// still exist (deleted in Phase 9 of movement_system_plan), but the client
// no longer subscribes to them — positions come from `Ship.movement` /
// `CargoCrate.movement` / static `position` columns.

impl Player {
    /// Sobj id of the player's currently-controlled (in-sector) ship.
    /// Replaces the legacy `sobj_player_window` lookup.
    pub fn get_controlled_stellar_object_id(&self, ctx: &DbConnection) -> Option<u64> {
        let identity = self.id;
        ctx.db()
            .ship()
            .iter()
            .find(|s| s.player_id == identity && s.location == ShipLocation::Sector)
            .map(|s| s.sobj_id)
    }
}

impl Ship {
    pub fn status(&self, ctx: &DbConnection) -> Option<ShipStatus> {
        ctx.db().ship_status().id().find(&self.id)
    }
}

impl fmt::Display for ShipClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl fmt::Display for EquipmentSlotType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl StationSize {
    /// How many modules can this szie support?
    pub fn modules(&self) -> u8 {
        match self {
            StationSize::Capital => 13,
            StationSize::Large => 9,
            StationSize::Medium => 7,
            StationSize::Small => 5,
            StationSize::Outpost => 3,
            StationSize::Satellite => 1,
        }
    }

    pub fn base_cost(&self) -> u32 {
        (self.modules().pow(2) as u32) * 100_000 + 300_000
    }

    /// Retooling a space station to a larger size should be possible, but discouraged.
    pub fn upgrade_cost(&self, new_size: StationSize) -> u32 {
        new_size.base_cost() - self.base_cost() + ((new_size.modules() - self.modules()) as u32)
    }

    pub fn base_health(&self) -> u32 {
        (self.modules().pow(2) as u32) * 25_000 + 100_000
    }

    pub fn base_shields(&self) -> u32 {
        (self.modules().pow(2) as u32) * 50_000 + 200_000
    }
}

/// STDSL ID Type Impls ///
//
// Regex to match Id struct names.
/*
(\w+Id) (\w+)

impl $1 {
    pub fn new(id: $2) -> Self {
        $1 { value: id }
    }
}

OR:
impl From<$2> for $1 {
    fn from(id: $2) -> Self {
        $1 { value: id }
    }
}

*/

impl From<u32> for FactionId {
    fn from(id: u32) -> Self {
        FactionId { value: id }
    }
}

impl From<u32> for ItemDefinitionId {
    fn from(id: u32) -> Self {
        ItemDefinitionId { value: id }
    }
}

impl From<u64> for SectorId {
    fn from(id: u64) -> Self {
        SectorId { value: id }
    }
}

impl From<u64> for ShipId {
    fn from(id: u64) -> Self {
        ShipId { value: id }
    }
}

impl From<u64> for StationModuleId {
    fn from(id: u64) -> Self {
        StationModuleId { value: id }
    }
}

impl From<u64> for StellarObjectId {
    fn from(id: u64) -> Self {
        StellarObjectId { value: id }
    }
}
