use macroquad::prelude::glam;
use spacetimedb_sdk::{DbContext, Identity};

use crate::module_bindings::*;


pub fn get_transform(ctx:&DbConnection, sobj_id:u64) -> Result<StellarObjectTransformHiRes, String> {
    if let Some(hr)= ctx.db.sobj_hi_res_transform().sobj_id().find(&sobj_id) {
        Ok(hr)
    } else {
        if let Some(lr) = ctx.db.sobj_low_res_transform().sobj_id().find(&sobj_id) {
            Ok(StellarObjectTransformHiRes {
                sobj_id: lr.sobj_id,
                x: lr.x,
                y: lr.y,
                rotation_radians: lr.rotation_radians,
            })
        } else {
            Err("Could not find transform, even low-rez.".to_string())
        }
    }
}

pub fn get_username(ctx: &DbConnection, id:&Identity) -> String {
    if let Some(player) = ctx.db.player().identity().find(id) {
        player.username
    } else {
        id.to_abbreviated_hex().to_string()
    }
}

pub fn get_current_player(ctx: &DbConnection) -> Option<Player> {
    get_player(&ctx.db, &ctx.identity())
}

pub fn get_player(db: &RemoteTables, id: &Identity) -> Option<Player> {
    let this = db.player().identity().find(id);
    match this {
        Some(p) => Some(p),
        None => None,
    }
}

pub fn get_player_sobj_id(ctx: &DbConnection) -> Option<u64> {
    if let Some(this) = get_current_player(ctx) {
        this.get_controlled_stellar_object(ctx)
    } else {
        None
    }
}

pub fn get_player_ship_object(ctx: &DbConnection) -> Option<ShipObject> {
    if let Some(this) = ctx.db.ship_object().sobj_id().find(&get_player_sobj_id(ctx)?) {
        Some(this)
    } else {
        None
    }
}

pub fn get_player_ship_instance(ctx: &DbConnection) -> Option<ShipInstance> {
    if let Some(this) = get_player_ship_object(ctx) {
        match ctx.db.ship_instance().id().find(&this.ship_id) {
            Some(instance) => Some(instance),
            None => None
        }
    } else {
        None
    }
}

pub fn get_player_transform(ctx: &DbConnection) -> Option<StellarObjectTransformHiRes> {
    if let Some(this) = get_player_sobj_id(ctx) {
        get_transform(ctx, this).ok()
    } else {
        None
    }
}

pub fn get_player_transform_vec2(ctx: &DbConnection, default: glam::Vec2) -> glam::Vec2 {
    if let Some(this) = get_player_sobj_id(ctx) {
        match get_transform(ctx, this) {
            Ok(t) => t.to_vec2(),
            Err(_) => default,
        }
    } else {
        default
    }
}