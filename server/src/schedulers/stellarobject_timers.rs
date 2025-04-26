use std::time::Duration;
use glam::Vec2;
use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table};

use crate::types::stellarobjects::{*};

// // Intentionally private
// #[spacetimedb::table(name = update_config)]
// pub struct UpdateConfig {
//     #[unique]
//     id: u32,
//     value: i32,
// }

#[spacetimedb::table(name = update_sobj_transform_timer, scheduled(update_sobj_transforms))]
pub struct UpdateTransformsTimer {
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
    current_update: u8,
}

#[spacetimedb::table(name = move_ships_timer, scheduled(move_ships))]
pub struct MoveShipsTimer {
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
    current_update: u8,
}

pub fn init(ctx: &ReducerContext) {
    ctx.db.update_sobj_transform_timer().insert(UpdateTransformsTimer {
        scheduled_id: 0,
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_millis(1000 / 20).into()),
        current_update: 0
    });
}

#[spacetimedb::reducer]
pub fn update_sobj_transforms(ctx: &ReducerContext, timer: UpdateTransformsTimer) {
    // We're using this value to determine whether or not to update the lower resolution table.
    let mut update = timer;
    let low_resolution = update.current_update == 0;

    // Only let SpacetimeDB call this function
    if ctx.sender != ctx.identity() {
        panic!("This reducer can only be called by SpacetimeDB!");
    }

    __move_ships(ctx);

    // Update the value in the config table
    update.current_update = (update.current_update + 1) % 5; // TODO: Make this configurable
    ctx.db.update_sobj_transform_timer().scheduled_id().update(update);


    // Clear all high res positions
    for row in ctx.db.stellar_object_hi_res().iter() {
        ctx.db.stellar_object_hi_res().sobj_id().delete(row.sobj_id);
    }

    // Clear all low res positions
    if low_resolution {
        for row in ctx.db.stellar_object_low_res().iter() {
            ctx.db.stellar_object_low_res().sobj_id().delete(row.sobj_id);
        }
    }

    // Update all high res positions
    for row in ctx.db.stellar_object_internal().iter() {
        ctx.db.stellar_object_hi_res().insert(StellarObjectTransform {
            sobj_id: row.sobj_id,
            x: row.x,
            y: row.y,
            rotation_radians: row.rotation_radians
        });

        // Update all low res positions
        if low_resolution {
            ctx.db.stellar_object_low_res().insert(StellarObjectTransform {
                sobj_id: row.sobj_id,
                x: row.x,
                y: row.y,
                rotation_radians: row.rotation_radians
            });
        }
    }
}

#[spacetimedb::reducer]
pub fn move_ships(ctx: &ReducerContext, _timer: MoveShipsTimer) {
    __move_ships(ctx)
}

pub fn __move_ships(ctx: &ReducerContext) {
    for object in ctx.db.stellar_object().iter() {
        let wrapped_transform = ctx.db.stellar_object_internal().sobj_id().find(object.id);
        if wrapped_transform.is_none() { continue; }
        let tranform = wrapped_transform.unwrap();
        let wrapped_velocity = ctx.db.stellar_object_velocity().sobj_id().find(object.id);
        if wrapped_velocity.is_none() { continue; }
        let velocity = wrapped_velocity.unwrap();

        let current_pos = tranform.to_vec2();
        let mut x = current_pos.x;
        let mut y = current_pos.y;
        //let velocity = Vec2::from_angle(tranform.rotation_radians) * 1.337;

        if x > 500.0 {
            x -= 500.0;
        } else if x < 0. {
            x += 500.0;
        }

        if y > 500.0 {
            y -= 500.0;
        } else if y < 0. {
            y += 500.0;
        }

        ctx.db.stellar_object_internal().sobj_id().update(
            StellarObjectTransform {
                sobj_id: tranform.sobj_id,
                x: x + velocity.x,
                y: y + velocity.y,
                rotation_radians: tranform.rotation_radians + velocity.rotation_radians
            }
        );
    }
}