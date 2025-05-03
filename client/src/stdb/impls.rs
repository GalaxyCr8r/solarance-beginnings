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