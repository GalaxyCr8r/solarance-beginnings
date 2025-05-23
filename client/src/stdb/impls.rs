use std::fmt::{self, Debug};

use macroquad::prelude::{glam};

use crate::module_bindings::*;

/// Impls ///

impl StellarObjectVelocity {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 { x: self.x, y: self.y }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectVelocity {
        StellarObjectVelocity { x: vec.x, y: vec.y, ..*self }
    }
}

impl StellarObjectTransformHiRes {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 { x: self.x, y: self.y }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectTransformHiRes {
        StellarObjectTransformHiRes { x: vec.x, y: vec.y, ..*self }
    }
}

impl Player {
    pub fn get_controlled_stellar_object(&self, ctx: &DbConnection) -> Option<u64> {
        if let Some(player_window) = ctx.db.sobj_player_window().identity().find(&self.identity) {
            Some(player_window.sobj_id)
        } else {
            None
        }
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