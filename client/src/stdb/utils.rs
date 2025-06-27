use macroquad::prelude::glam;
use spacetimedb_sdk::{ DbContext, Identity, Table };

use crate::module_bindings::*;

pub fn get_transform(
    ctx: &DbConnection,
    sobj_id: u64
) -> Result<StellarObjectTransformHiRes, String> {
    if let Some(hr) = ctx.db().sobj_hi_res_transform().sobj_id().find(&sobj_id) {
        Ok(hr)
    } else {
        if let Some(lr) = ctx.db().sobj_low_res_transform().sobj_id().find(&sobj_id) {
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

pub fn get_username(ctx: &DbConnection, id: &Identity) -> String {
    if let Some(player) = ctx.db().player().identifier().find(id) {
        player.username
    } else {
        id.to_abbreviated_hex().to_string()
    }
}

pub fn get_current_player(ctx: &DbConnection) -> Option<Player> {
    get_player(&ctx.db, &ctx.identity())
}

pub fn get_player(db: &RemoteTables, id: &Identity) -> Option<Player> {
    let this = db.player().identifier().find(id);
    match this {
        Some(p) => Some(p),
        None => None,
    }
}

pub fn get_player_sobj_id(ctx: &DbConnection) -> Option<u64> {
    if let Some(this) = get_current_player(ctx) {
        this.get_controlled_stellar_object_id(ctx)
    } else {
        None
    }
}

pub fn get_player_ship(ctx: &DbConnection) -> Option<Ship> {
    ctx.db().ship().sobj_id().find(&get_player_sobj_id(ctx)?)
}

pub fn get_all_equipped_of_type(
    ctx: &DbConnection,
    ship_id: u64,
    slot_type: EquipmentSlotType
) -> Vec<ShipEquipmentSlot> {
    let mut equipment = Vec::new();
    for slot in ctx.db().ship_equipment_slot().iter() {
        if slot.ship_id == ship_id {
            if slot.slot_type == slot_type {
                equipment.push(slot);
            }
        }
    }
    equipment
}

// pub fn get_player_docked_ship(ctx: &DbConnection) -> Option<Ship> {
//     ctx.db()
//         .docked_ship()
//         .iter()
//         .filter(|ds| ds.player_id == ctx.identity())
//         .next()
// }

pub fn get_player_ship_status(ctx: &DbConnection) -> Option<ShipStatus> {
    if let Some(this) = get_player_ship(ctx) { this.status(ctx) } else { None }
}

pub fn get_player_transform(ctx: &DbConnection) -> Option<StellarObjectTransformHiRes> {
    if let Some(this) = get_player_sobj_id(ctx) { get_transform(ctx, this).ok() } else { None }
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
