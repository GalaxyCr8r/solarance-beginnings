use std::fmt::{self, Debug};

use macroquad::prelude::{glam};
use spacetimedb_sdk::Table;

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

impl StellarObjectTransformLowRes {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 { x: self.x, y: self.y }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectTransformLowRes {
        StellarObjectTransformLowRes { x: vec.x, y: vec.y, ..*self }
    }
}

impl Player {
    pub fn get_controlled_stellar_object_id(&self, ctx: &DbConnection) -> Option<u64> {
        if let Some(player_window) = ctx.db.sobj_player_window().player_id().find(&self.identifier) {
            Some(player_window.sobj_id)
        } else {
            None
        }
    }
}

impl Ship {
    pub fn get_all_equipped_of_type(&self, ctx: &DbConnection, slot_type: EquipmentSlotType) -> Vec<ShipEquipmentSlot> {
        let mut equipment = Vec::new();
        for slot in ctx.db.ship_equipment_slot().iter() {
            if slot.ship_id == self.id {
                if slot.slot_type == slot_type {
                    equipment.push(slot);
                }
            }
        }
        equipment
    }

    pub fn status(&self, ctx: &DbConnection) -> Option<ShipStatus> {
        ctx.db.ship_status().id().find(&self.id)
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