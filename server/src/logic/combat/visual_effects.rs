use spacetimedb::ReducerContext;
use spacetimedsl::{dsl, Wrapper};

use crate::types::{
    items::utility::get_item_definition, npcs::*, players::*, sectors::SectorId, ships::*,
    stellarobjects::*,
};

use crate::types::combat::utility::{
    get_equipped_weapons, log_combat_error, process_missile_fire, process_weapon_fire,
    test_combat_validation, validate_combat_action,
};
use crate::types::combat::{
    CombatError, CreateVisualEffectRow, CreateVisualEffectTimerRow, MissileType, ShipController,
    VisualEffectType, WeaponType,
};

/// Create visual effect for failed weapon shots (when target is invalid or out of range)
fn create_visual_effect_for_failed_shot(
    ctx: &ReducerContext,
    source_sobj_id: u64,
    target_sobj_id: Option<u64>,
) -> Result<(), CombatError> {
    let dsl = dsl(ctx);

    // Get source stellar object and ship
    let source_sobj = dsl
        .get_stellar_object_by_id(StellarObjectId::new(source_sobj_id))
        .map_err(|_| CombatError::InvalidTarget)?;

    let source_ship = dsl
        .get_ships_by_sobj_id(StellarObjectId::new(source_sobj_id))
        .next()
        .ok_or(CombatError::InvalidTarget)?;

    // Get source position and facing
    let source_transform = dsl
        .get_sobj_internal_transform_by_id(source_sobj.get_id())
        .map_err(|_| CombatError::InvalidTarget)?;
    let source_pos = source_transform.to_vec2();
    let ship_facing_angle = *source_transform.get_rotation_radians();

    // Calculate target position
    let target_pos = if let Some(target_id) = target_sobj_id {
        // If we have a target, try to get its position
        if let Ok(target_sobj) = dsl.get_stellar_object_by_id(StellarObjectId::new(target_id)) {
            let target_transform = dsl
                .get_sobj_internal_transform_by_id(target_sobj.get_id())
                .map_err(|_| CombatError::InvalidTarget)?;
            target_transform.to_vec2()
        } else {
            // Target doesn't exist, fire in facing direction
            let range = 100.0; // Default range for visual effect
            glam::Vec2::new(
                source_pos.x + ship_facing_angle.cos() * range,
                source_pos.y + ship_facing_angle.sin() * range,
            )
        }
    } else {
        // No target, fire in facing direction
        let range = 100.0; // Default range for visual effect
        glam::Vec2::new(
            source_pos.x + ship_facing_angle.cos() * range,
            source_pos.y + ship_facing_angle.sin() * range,
        )
    };

    // Get sector_id from source ship
    let source_sector_id = source_ship.get_sector_id().value();

    // Create visual effect using the internal function
    create_visual_effect_internal(
        ctx,
        source_pos,
        target_pos,
        VisualEffectType::WeaponFire,
        source_sector_id,
    )
}

/// Create visual effect for failed missile shots (when target is invalid or out of range)
fn create_visual_effect_for_failed_missile(
    ctx: &ReducerContext,
    source_sobj_id: u64,
    target_sobj_id: Option<u64>,
) -> Result<(), CombatError> {
    let dsl = dsl(ctx);

    // Get source stellar object and ship
    let source_sobj = dsl
        .get_stellar_object_by_id(StellarObjectId::new(source_sobj_id))
        .map_err(|_| CombatError::InvalidTarget)?;

    let source_ship = dsl
        .get_ships_by_sobj_id(StellarObjectId::new(source_sobj_id))
        .next()
        .ok_or(CombatError::InvalidTarget)?;

    // Get source position and facing
    let source_transform = dsl
        .get_sobj_internal_transform_by_id(source_sobj.get_id())
        .map_err(|_| CombatError::InvalidTarget)?;
    let source_pos = source_transform.to_vec2();
    let ship_facing_angle = *source_transform.get_rotation_radians();

    // Calculate target position
    let target_pos = if let Some(target_id) = target_sobj_id {
        // If we have a target, try to get its position
        if let Ok(target_sobj) = dsl.get_stellar_object_by_id(StellarObjectId::new(target_id)) {
            let target_transform = dsl
                .get_sobj_internal_transform_by_id(target_sobj.get_id())
                .map_err(|_| CombatError::InvalidTarget)?;
            target_transform.to_vec2()
        } else {
            // Target doesn't exist, fire in facing direction
            let range = 150.0; // Slightly longer range for missiles
            glam::Vec2::new(
                source_pos.x + ship_facing_angle.cos() * range,
                source_pos.y + ship_facing_angle.sin() * range,
            )
        }
    } else {
        // No target, fire in facing direction
        let range = 150.0; // Slightly longer range for missiles
        glam::Vec2::new(
            source_pos.x + ship_facing_angle.cos() * range,
            source_pos.y + ship_facing_angle.sin() * range,
        )
    };

    // Get sector_id from source ship
    let source_sector_id = source_ship.get_sector_id().value();

    // Create visual effect using the internal function
    create_visual_effect_internal(
        ctx,
        source_pos,
        target_pos,
        VisualEffectType::MissileFire,
        source_sector_id,
    )
}

/// Internal function to create visual effects (duplicated from utility to avoid circular dependencies)
fn create_visual_effect_internal(
    ctx: &ReducerContext,
    source_pos: glam::Vec2,
    target_pos: glam::Vec2,
    effect_type: VisualEffectType,
    sector_id: u64,
) -> Result<(), CombatError> {
    let dsl = dsl(ctx);

    // Create visual effect
    let visual_effect = dsl.create_visual_effect(
        SectorId::new(sector_id),
        source_pos.into(),
        target_pos.into(),
        effect_type,
    )?;

    // Schedule cleanup after 10 milliseconds
    let cleanup_time =
        spacetimedb::ScheduleAt::Time(spacetimedb::Timestamp::from_micros_since_unix_epoch(
            ctx.timestamp.to_micros_since_unix_epoch() + 10_000,
        ));

    dsl.create_visual_effect_timer(visual_effect.get_id(), cleanup_time)?;

    Ok(())
}
