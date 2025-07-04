use std::fmt::{ self, Debug };

use macroquad::prelude::glam;
use spacetimedb_sdk::*;

use crate::module_bindings::*;

/// Impls ///

impl StellarObjectVelocity {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectVelocity {
        StellarObjectVelocity {
            x: vec.x,
            y: vec.y,
            ..*self
        }
    }
}

impl StellarObjectTransformHiRes {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectTransformHiRes {
        StellarObjectTransformHiRes {
            x: vec.x,
            y: vec.y,
            ..*self
        }
    }
}

impl StellarObjectTransformLowRes {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectTransformLowRes {
        StellarObjectTransformLowRes {
            x: vec.x,
            y: vec.y,
            ..*self
        }
    }
}

impl Player {
    pub fn get_controlled_stellar_object_id(&self, ctx: &DbConnection) -> Option<u64> {
        if let Some(player_window) = ctx.db().sobj_player_window().id().find(&self.id) {
            Some(player_window.sobj_id)
        } else {
            None
        }
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

// impl PlayerId {
//     /// Create a new PlayerId from STDB Identity struct.
//     pub fn new(identity: &Identity) -> Self {
//         PlayerId {}
//     }
// }
