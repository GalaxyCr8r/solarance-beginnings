use crate::types::{
    asteroids::*,
    items::*,
    jumpgates::*,
    players::DeletePlayerShipControllerRowByPlayerId,
    ships::{timers::*, *},
    stations::*,
};

use super::*;

impl StellarObject {
    /// Attempts to delete the StellarObject. Returns the object if found.
    pub fn delete_from_id(
        ctx: &ReducerContext,
        id: &StellarObjectId,
        delete_kind_specific_rows: bool,
    ) -> Option<StellarObject> {
        dsl(ctx)
            .get_stellar_object_by_id(id)
            .inspect(|sobj| sobj.delete(ctx, delete_kind_specific_rows))
    }

    pub fn distance_squared(&self, ctx: &ReducerContext, target: &StellarObject) -> Option<f32> {
        let dsl = dsl(ctx);

        let transform = dsl.get_sobj_internal_transform_by_sobj_id(self);
        let target_transform = dsl.get_sobj_internal_transform_by_sobj_id(target);

        if transform.is_some() && target_transform.is_some() {
            Some(
                transform
                    .unwrap()
                    .to_vec2()
                    .distance_squared(target_transform.unwrap().to_vec2()),
            )
        } else {
            None
        }
    }

    /// Attempts to smartly delete everything related to this stellar object.
    pub fn delete(&self, ctx: &ReducerContext, delete_kind_specific_rows: bool) {
        let dsl = dsl(ctx);

        if let Some(window) = dsl.get_sobj_player_window_by_sobj_id(self) {
            dsl.delete_player_ship_controller_by_player_id(&window.player_id);
        }

        dsl.delete_stellar_object_by_id(self);
        dsl.delete_sobj_player_window_by_sobj_id(self);
        dsl.delete_sobj_internal_transform_by_sobj_id(self);
        dsl.delete_sobj_low_res_transform_by_sobj_id(self);
        dsl.delete_sobj_hi_res_transform_by_sobj_id(self);
        dsl.delete_sobj_turn_left_controller_by_sobj_id(self);

        // Timers
        dsl.delete_ship_mining_timers_by_ship_sobj_id(self);

        // Kind-specific
        if delete_kind_specific_rows {
            match self.kind {
                StellarObjectKinds::Ship => {
                    if let Some(ship_object) = dsl.get_ship_by_sobj_id(self) {
                        dsl.delete_ship_status_by_id(ship_object.get_id());
                        dsl.delete_ship_cargo_items_by_ship_id(ship_object.get_id());
                        dsl.delete_ship_equipment_slots_by_ship_id(ship_object.get_id());

                        dsl.delete_ship_status_by_id(ship_object.get_id());
                        dsl.delete_ship_add_cargo_timers_by_ship_id(ship_object.get_id());
                    }
                    dsl.delete_ship_by_sobj_id(self);
                }
                StellarObjectKinds::Asteroid => {
                    dsl.delete_asteroid_by_sobj_id(self);
                }
                StellarObjectKinds::CargoCrate => {
                    dsl.delete_cargo_crate_by_sobj_id(self);
                }
                StellarObjectKinds::Station => {
                    dsl.delete_station_by_sobj_id(self);
                }
                StellarObjectKinds::JumpGate => {
                    dsl.delete_jump_gate_by_sobj_id(self);
                }
            }
        }
    }
}

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

impl StellarObjectTransformInternal {
    pub fn get(
        ctx: &ReducerContext,
        id: &StellarObjectId,
    ) -> Option<StellarObjectTransformInternal> {
        dsl(ctx).get_sobj_internal_transform_by_sobj_id(id)
    }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectTransformInternal {
        StellarObjectTransformInternal {
            x: vec.x,
            y: vec.y,
            ..*self
        }
    }

    pub fn from_xy(&self, x: f32, y: f32) -> StellarObjectTransformInternal {
        StellarObjectTransformInternal {
            x: x,
            y: y,
            ..*self
        }
    }
}
