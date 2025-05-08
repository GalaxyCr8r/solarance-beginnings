use macroquad::{math::Vec2, prelude::glam};
use spacetimedb_sdk::{DbContext, Identity};

use crate::module_bindings::*;


pub fn get_transform(ctx:&DbConnection, sobj_id:u64) -> Result<StellarObjectTransform, String> {
    if let Some(hr)= ctx.db.stellar_object_hi_res().sobj_id().find(&sobj_id) {
        Ok(hr)
    } else {
        if let Some(lr) = ctx.db.stellar_object_low_res().sobj_id().find(&sobj_id) {
            Ok(lr)
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

pub fn get_player_transform(ctx: &DbConnection) -> Option<StellarObjectTransform> {
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