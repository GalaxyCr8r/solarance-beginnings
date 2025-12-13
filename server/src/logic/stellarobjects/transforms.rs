use glam::Vec2;
use log::info;
use spacetimedb::*;
use spacetimedsl::*;
use std::f32::consts::PI;

use crate::tables::items::*;
use crate::tables::stellarobjects::*;
use crate::utility::*;

#[dsl(plural_name = all_transforms_timers)]
#[spacetimedb::table(name = all_transforms_timer, scheduled(recalculate_sobj_transforms))]
pub struct AllTransformsTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    scheduled_at: spacetimedb::ScheduleAt,
    pub current_update: u8,
}

/// Scheduled reducer that recalculates and updates stellar object transforms.
/// Runs every 50ms (20 FPS) to move objects and update their positions.
/// Updates both high-resolution (every tick) and low-resolution (every 5th tick) transform tables.
#[spacetimedb::reducer]
pub fn recalculate_sobj_transforms(
    ctx: &ReducerContext,
    timer: AllTransformsTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    try_server_only(&dsl)?;

    // We're using this value to determine whether or not to update the lower resolution table.
    let mut update = timer;
    let low_resolution = update.current_update == 0;

    move_stellar_objects(&dsl)?;

    // Update the value in the config table
    update.current_update = (update.current_update + 1) % 5; // TODO: Make this configurable
    dsl.update_all_transforms_timer_by_id(update)?;

    // Clear all high res positions
    for row in dsl.get_all_sobj_hi_res_transforms() {
        dsl.delete_sobj_hi_res_transform_by_id(&row.get_id())?;
    }

    // Clear all low res positions
    if low_resolution {
        for row in dsl.get_all_sobj_low_res_transforms() {
            dsl.delete_sobj_low_res_transform_by_id(&row.get_id())?;
        }
    }

    // Update all high res positions
    for row in dsl.get_all_sobj_internal_transforms() {
        dsl.create_sobj_hi_res_transform(
            &row.get_id(),
            *row.get_x(),
            *row.get_y(),
            *row.get_rotation_radians(),
        )?; // TODO, Only create hi-res transforms if a player is in-sector.

        // Update all low res positions
        if low_resolution {
            dsl.create_sobj_low_res_transform(
                row.get_id(),
                *row.get_x(),
                *row.get_y(),
                *row.get_rotation_radians(),
            )?;
        }
    }

    Ok(())
}

pub fn __move_stellar_object(dsl: &DSL, sobj: StellarObject) -> Result<(), String> {
    let mut transform = dsl.get_sobj_internal_transform_by_id(&sobj)?;
    let mut velocity = dsl.get_sobj_velocity_by_id(&sobj)?;

    // TODO: Remove this code, this is ONLY for early milestones!
    if let Ok(_) = dsl.get_sobj_turn_left_controller_by_id(&sobj) {
        velocity = velocity.from_vec2(Vec2::from_angle(*transform.get_rotation_radians()) * 25.0);
        transform.set_rotation_radians(transform.get_rotation_radians() + PI * 0.01337);
    }
    // TODO:RM

    if let Some(dampen) = velocity.get_auto_dampen() {
        velocity = velocity.from_vec2(velocity.to_vec2() * dampen);
    }

    // Apply velocity to transform
    transform = transform.from_vec2(transform.to_vec2() + velocity.to_vec2());
    transform
        .set_rotation_radians(transform.get_rotation_radians() + velocity.get_rotation_radians());
    if transform.get_rotation_radians().abs() > PI * 2.0 {
        transform.set_rotation_radians((transform.get_rotation_radians().abs() % PI) * 2.0);
    }

    dsl.update_sobj_velocity_by_id(velocity)?;
    dsl.update_sobj_internal_transform_by_id(transform)?;

    if *sobj.get_kind() == StellarObjectKinds::CargoCrate {
        let cargo_crate = dsl.get_cargo_crate_by_sobj_id(sobj.get_id())?;
        if let Some(despawn_ts) = cargo_crate.get_despawn_ts() {
            if *despawn_ts < dsl.ctx().timestamp {
                info!(
                    "Cargo Crate outlived its despawn timestamp. Deleting #{}!",
                    sobj.get_id()
                );
                dsl.delete_stellar_object_by_id(&sobj)?;
            }
        }
    }

    Ok(())
}

/// Server-only ~reducer~ that applies physics updates to all stellar objects.
/// Updates position, rotation, and velocity for each object based on their current state.
//#[spacetimedb::reducer]
pub fn move_stellar_objects(dsl: &DSL) -> Result<(), String> {
    // let dsl = dsl(ctx);
    // try_server_only(dsl)?;

    // TODO Cache which sectors have players in them and only do fine updates in those.

    for object in dsl.get_all_stellar_objects() {
        __move_stellar_object(&dsl, object)?;
    }
    Ok(())
}
