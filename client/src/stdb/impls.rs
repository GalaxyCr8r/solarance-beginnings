use macroquad::prelude::glam;

use crate::module_bindings::*;

/// Impls ///

impl StellarObjectTransform {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 { x: self.x, y: self.y }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectTransform {
        StellarObjectTransform { x: vec.x, y: vec.y, ..*self }
    }
}

impl Player {
    pub fn get_controlled_stellar_object(&self, ctx: &DbConnection) -> Option<u64> {
        if let Some(player_controlled_stellar_object) = ctx.db.player_controlled_stellar_object().identity().find(&self.identity) {
            Some(player_controlled_stellar_object.controlled_sobj_id)
        } else {
            None
        }
    }
}