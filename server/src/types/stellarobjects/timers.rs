use std::{f32::consts::PI, time::Duration};
use glam::Vec2;
use spacetimedb::{ReducerContext};
use spacetimedsl::{dsl};

use crate::types::{common::*, ships::GetShipObjectRowsByPlayerId, stellarobjects::*, utility::*};

#[dsl(plural_name = sobj_transform_timers)]
#[spacetimedb::table(name = sobj_transform_timer, scheduled(recalculate_sobj_transforms))]
pub struct TransformsTimer {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
    current_update: u8,
}

#[dsl(plural_name = player_windows_timers)]
#[spacetimedb::table(name = player_windows_timer, scheduled(recalculate_player_windows))]
pub struct PlayerWindowsTimer {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx); // Waiting for DSL implementation of timers

    dsl.create_sobj_transform_timer(spacetimedb::ScheduleAt::Interval(Duration::from_millis(1000 / 20).into()), 0)?;
    dsl.create_player_windows_timer(spacetimedb::ScheduleAt::Interval(Duration::from_millis(750).into()))?;

    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn recalculate_sobj_transforms(ctx: &ReducerContext, timer: TransformsTimer) -> Result<(), String> {
    let dsl = dsl(ctx);

    try_server_only(ctx)?;

    // Bail out ASAP if there's no players connected.
    if !are_there_active_players(ctx) {
        return Ok(());
    }

    // We're using this value to determine whether or not to update the lower resolution table.
    let mut update = timer;
    let low_resolution = update.current_update == 0;

    move_ships(ctx)?;

    // Update the value in the config table
    update.current_update = (update.current_update + 1) % 5; // TODO: Make this configurable
    dsl.update_sobj_transform_timer_by_scheduled_id(update)?;

    // Clear all high res positions
    for row in dsl.get_all_sobj_hi_res_transforms() {
        dsl.delete_sobj_hi_res_transform_by_sobj_id(row.get_sobj_id());
    }

    // Clear all low res positions
    if low_resolution {
        for row in dsl.get_all_sobj_low_res_transforms() {
            dsl.delete_sobj_low_res_transform_by_sobj_id(row.get_sobj_id());
        }
    }

    // Update all high res positions
    for row in dsl.get_all_sobj_internal_transforms() {
        dsl.create_sobj_hi_res_transform(row.get_sobj_id(), row.x, row.y, row.rotation_radians)?;

        // Update all low res positions
        if low_resolution {
            dsl.create_sobj_low_res_transform(row.get_sobj_id(), row.x, row.y, row.rotation_radians)?;
        }
    }
    Ok(())
}

pub fn __move_ship(ctx: &ReducerContext, sobj: StellarObject) -> Result<(), String> {
    let dsl = dsl(ctx);
    
    if let Some(mut transform) = dsl.get_sobj_internal_transform_by_sobj_id(&sobj) {
        if let Some(mut velocity) = dsl.get_sobj_velocity_by_sobj_id(&sobj) {
            // TODO: Remove this code, this is ONLY for early milestones!
            if let Some(_) = dsl.get_sobj_turn_left_controller_by_sobj_id(&sobj) {
                velocity = velocity.from_vec2(Vec2::from_angle(transform.rotation_radians) * 25.0);
                transform.rotation_radians += PI * 0.01337;
            }
            // TODO:RM
            
            // Apply velocity to transform
            transform=transform.from_vec2(transform.to_vec2() + velocity.to_vec2());
            transform.rotation_radians += velocity.rotation_radians;
            if transform.rotation_radians.abs() > PI * 2.0 {
                transform.rotation_radians = transform.rotation_radians.abs() % PI * 2.0;
            }

            // Add inertia to the velocity
            velocity=velocity.from_vec2(velocity.to_vec2() * 0.975); // TODO:: Milestone 10+ make these ship-type specific values.
            if velocity.to_vec2().length() < 0.01337 {
                velocity = velocity.from_vec2(Vec2::ZERO);
            }
            velocity.rotation_radians *= 0.75; // TODO:: Milestone 10+ make these ship-type specific values.
            
            dsl.update_sobj_velocity_by_sobj_id(velocity)?;
        }

        dsl.update_sobj_internal_transform_by_sobj_id(transform)?;
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn move_ships(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    try_server_only(ctx)?;

    // TODO Cache which sectors have players in them and only do fine updates in those.

    for object in dsl.get_all_stellar_objects() {
        __move_ship(ctx, object)?;
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn recalculate_player_windows(ctx: &ReducerContext, _timer: PlayerWindowsTimer) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Bail out ASAP if there's no players connected.
    if !are_there_active_players(ctx) {
        return Ok(());
    }
    
    for window in dsl.get_all_sobj_player_windows() {
        if let Some(ship_obj) = dsl.get_ship_objects_by_player_id(&window.identity).last() {
            if let Some(transform) = dsl.get_sobj_internal_transform_by_sobj_id(ship_obj.get_sobj_id()) {
                // Check to see if the player has moved too close to window's margin and recalculate the window if needed.
                if transform.x < window.tl_x + window.margin || 
                   transform.x > window.br_x - window.margin ||
                   transform.y < window.tl_y + window.margin || 
                   transform.y > window.br_y - window.margin 
                {                    
                    dsl.update_sobj_player_window_by_identity(StellarObjectPlayerWindow { 
                        tl_x: transform.x - window.window, 
                        tl_y: transform.y - window.window, 
                        br_x: transform.x + window.window, 
                        br_y: transform.y + window.window,
                        ..window
                    })?;
                    //info!("Recalcuating window for player stellar obj #{}: [({}, {}) ({}, {})]", player.sobj_id, result.tl_x, result.tl_y, result.br_x, result.br_y);
                }
            }
        }
    }
    Ok(())
}
