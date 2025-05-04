use macroquad::{math::Vec2, prelude::glam};
use spacetimedb_sdk::DbContext;

use crate::module_bindings::*;


pub fn get_transform(ctx:&DbConnection, sobj_id:u64) -> Result<StellarObjectTransform, String> {
    let hr= ctx.db.stellar_object_hi_res().sobj_id().find(&sobj_id);
    if hr.is_some() {
        Ok(hr.unwrap())
    } else {
        let lr = ctx.db.stellar_object_low_res().sobj_id().find(&sobj_id);
        if lr.is_some() {
            Ok(lr.unwrap())
        } else {
            Err("Could not find transform, even low-rez.".to_string())
        }
    }
}

pub fn get_player(ctx: &DbConnection) -> Option<Player> {
    let this = ctx.db.player().identity().find(&ctx.identity());
    match this {
        Some(p) => Some(p),
        None => None,
    }
}

pub fn get_player_sobj_id(ctx: &DbConnection) -> Option<u64> {
    let this = get_player(ctx);
    match this {
        Some(v) => v.controlled_entity_id,
        None => None,
    }
}

pub fn get_player_transform(ctx: &DbConnection) -> Option<StellarObjectTransform> {
    let this = get_player_sobj_id(ctx);
    if this.is_some() {
        get_transform(ctx, this.unwrap()).ok()
    } else {None}
}

pub fn get_player_transform_vec2(ctx: &DbConnection, default: glam::Vec2) -> glam::Vec2 {
    let this = get_player_sobj_id(ctx);
    if this.is_some() {
        match get_transform(ctx, this.unwrap()) {
            Ok(t) => t.to_vec2(),
            Err(_) => default,
        }
    } else {
        default
    }
}