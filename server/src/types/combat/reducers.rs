use spacetimedb::ReducerContext;
use spacetimedsl::{dsl, Wrapper};

use crate::types::{
    items::utility::get_item_definition, npcs::*, players::*, sectors::SectorId, ships::*,
    stellarobjects::*,
};

use super::utility::{
    get_equipped_weapons, log_combat_error, process_missile_fire, process_weapon_fire,
    test_combat_validation, validate_combat_action,
};
use super::{
    CombatError, CreateVisualEffectRow, CreateVisualEffectTimerRow, MissileType, ShipController,
    VisualEffectType, WeaponType,
};

/// Process combat actions from both PlayerShipController and NpcShipController
/// This reducer handles weapon and missile firing when the respective flags are set
#[spacetimedb::reducer]
pub fn process_combat_actions(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Collect all controllers (both player and NPC) that have combat actions pending
    let mut controllers: Vec<ShipController> = Vec::new();

    // Add player controllers
    let player_controllers: Vec<PlayerShipController> = dsl
        .get_all_player_ship_controllers()
        .filter(|controller| *controller.get_fire_weapons() || *controller.get_fire_missles())
        .collect();

    for controller in player_controllers {
        controllers.push(ShipController::Player(controller));
    }

    // Add NPC controllers
    let npc_controllers: Vec<NpcShipController> = dsl
        .get_all_npc_ship_controllers()
        .filter(|controller| *controller.get_fire_weapons() || *controller.get_fire_missiles())
        .collect();

    for controller in npc_controllers {
        controllers.push(ShipController::Npc(controller));
    }

    // Process combat actions for all controllers
    for mut controller in controllers {
        let source_sobj_id = controller.get_stellar_object_id();

        // Process weapon firing
        if controller.should_fire_weapons() {
            if let Some(target_sobj_id) = controller.get_targeted_sobj_id() {
                match process_weapon_combat_action(ctx, source_sobj_id, target_sobj_id) {
                    Ok(_) => {
                        spacetimedb::log::info!(
                            "Weapon fired successfully: {} -> {}",
                            source_sobj_id,
                            target_sobj_id
                        );
                    }
                    Err(e) => {
                        log_combat_error(source_sobj_id, Some(target_sobj_id), &e, "weapon fire");
                        // Create visual effect even when weapon fire fails
                        let _ = create_visual_effect_for_failed_shot(
                            ctx,
                            source_sobj_id,
                            Some(target_sobj_id),
                        );
                    }
                }
            } else {
                log_combat_error(
                    source_sobj_id,
                    None,
                    &CombatError::InvalidTarget,
                    "weapon fire",
                );
                // Create visual effect even when there's no target
                let _ = create_visual_effect_for_failed_shot(ctx, source_sobj_id, None);
            }

            // Reset fire_weapons flag
            controller.reset_fire_weapons();
        }

        // Process missile firing
        if controller.should_fire_missiles() {
            if let Some(target_sobj_id) = controller.get_targeted_sobj_id() {
                match process_missile_combat_action(ctx, source_sobj_id, target_sobj_id) {
                    Ok(_) => {
                        spacetimedb::log::info!(
                            "Missile fired successfully: {} -> {}",
                            source_sobj_id,
                            target_sobj_id
                        );
                    }
                    Err(e) => {
                        log_combat_error(source_sobj_id, Some(target_sobj_id), &e, "missile fire");
                        // Create visual effect even when missile fire fails
                        let _ = create_visual_effect_for_failed_missile(
                            ctx,
                            source_sobj_id,
                            Some(target_sobj_id),
                        );
                    }
                }
            } else {
                log_combat_error(
                    source_sobj_id,
                    None,
                    &CombatError::InvalidTarget,
                    "missile fire",
                );
                // Create visual effect even when there's no target
                let _ = create_visual_effect_for_failed_missile(ctx, source_sobj_id, None);
            }

            // Reset fire_missiles flag
            controller.reset_fire_missiles();
        }

        // Update the controller with reset flags
        match controller {
            ShipController::Player(player_controller) => {
                dsl.update_player_ship_controller_by_id(player_controller)?;
            }
            ShipController::Npc(npc_controller) => {
                dsl.update_npc_ship_controller_by_id(npc_controller)?;
            }
        }
    }

    Ok(())
}

/// Process weapon firing for a specific ship and target
fn process_weapon_combat_action(
    ctx: &ReducerContext,
    source_sobj_id: u64,
    target_sobj_id: u64,
) -> Result<(), CombatError> {
    // Perform comprehensive server-side validation
    validate_combat_action(ctx, source_sobj_id, target_sobj_id, false)?;

    let dsl = dsl(ctx);

    // Get validated target (we know it exists from validation)
    let target_sobj = dsl
        .get_stellar_object_by_id(StellarObjectId::new(target_sobj_id))
        .map_err(|_| CombatError::InvalidTarget)?;

    // Get source ship to find equipped weapons
    let source_ship = dsl
        .get_ships_by_sobj_id(StellarObjectId::new(source_sobj_id))
        .next()
        .ok_or(CombatError::InvalidTarget)?;

    // Find equipped weapons using helper function
    let weapon_slots = get_equipped_weapons(ctx, source_ship.get_id())?;

    // Get target position for actual_location parameter
    let target_transform = dsl
        .get_sobj_internal_transform_by_id(target_sobj.get_id())
        .map_err(|_| CombatError::InvalidTarget)?;
    let target_pos = target_transform.to_vec2();

    // Fire each equipped weapon
    for weapon_slot in weapon_slots {
        let weapon_def = get_item_definition(ctx, weapon_slot.get_item_id().value())
            .map_err(|_| CombatError::WeaponNotEquipped)?;

        // For now, assume all weapons are hitscan type
        // TODO: Determine weapon type from item metadata in future tasks
        let weapon_type = WeaponType::Hitscan;

        match process_weapon_fire(
            ctx,
            source_sobj_id,
            target_sobj_id,
            target_pos,
            weapon_type,
            weapon_def,
        ) {
            Ok(_) => {
                spacetimedb::log::info!(
                    "Weapon {} fired successfully from ship {}",
                    weapon_slot.get_item_id().value(),
                    source_sobj_id
                );
            }
            Err(e) => {
                spacetimedb::log::info!(
                    "Weapon {} failed to fire from ship {}: {}",
                    weapon_slot.get_item_id().value(),
                    source_sobj_id,
                    e.to_message()
                );
                // Continue with other weapons even if one fails
            }
        }
    }

    Ok(())
}

/// Process missile firing for a specific ship and target
fn process_missile_combat_action(
    ctx: &ReducerContext,
    source_sobj_id: u64,
    target_sobj_id: u64,
) -> Result<(), CombatError> {
    // Perform comprehensive server-side validation
    validate_combat_action(ctx, source_sobj_id, target_sobj_id, true)?;

    let dsl = dsl(ctx);

    // Get validated target (we know it exists from validation)
    let target_sobj = dsl
        .get_stellar_object_by_id(StellarObjectId::new(target_sobj_id))
        .map_err(|_| CombatError::InvalidTarget)?;

    // Get source ship to find equipped missiles
    let source_ship = dsl
        .get_ships_by_sobj_id(StellarObjectId::new(source_sobj_id))
        .next()
        .ok_or(CombatError::InvalidTarget)?;

    // TODO: Implement missile slot type in future tasks
    // For now, missiles will be handled as special equipment or weapons
    // This is a placeholder implementation as per the design document
    let missile_slots: Vec<ShipEquipmentSlot> = dsl
        .get_ship_equipment_slots_by_ship_id(source_ship.get_id())
        .filter(|slot| slot.get_slot_type() == &EquipmentSlotType::Special)
        .collect();

    if missile_slots.is_empty() {
        // Don't error for missing missiles since this is placeholder functionality
        spacetimedb::log::info!(
            "No missile equipment found for ship {} (placeholder implementation)",
            source_sobj_id
        );
        return Ok(());
    }

    // Get target position for actual_location parameter
    let target_transform = dsl
        .get_sobj_internal_transform_by_id(target_sobj.get_id())
        .map_err(|_| CombatError::InvalidTarget)?;
    let target_pos = target_transform.to_vec2();

    // Fire each equipped missile
    for missile_slot in missile_slots {
        let missile_def = get_item_definition(ctx, missile_slot.get_item_id().value())
            .map_err(|_| CombatError::WeaponNotEquipped)?;

        // For now, assume all missiles are dumbfire type
        // TODO: Determine missile type from item metadata in future tasks
        let missile_type = MissileType::Dumbfire;

        match process_missile_fire(
            ctx,
            source_sobj_id,
            target_sobj_id,
            target_pos,
            missile_type,
            missile_def,
        ) {
            Ok(_) => {
                spacetimedb::log::info!(
                    "Missile {} fired successfully from ship {}",
                    missile_slot.get_item_id().value(),
                    source_sobj_id
                );
            }
            Err(e) => {
                spacetimedb::log::info!(
                    "Missile {} failed to fire from ship {}: {}",
                    missile_slot.get_item_id().value(),
                    source_sobj_id,
                    e.to_message()
                );
                // Continue with other missiles even if one fails
            }
        }
    }

    Ok(())
}

/// Test reducer for combat error handling validation
/// This reducer can be used to test different combat error conditions
#[spacetimedb::reducer]
pub fn test_combat_error_handling(
    ctx: &ReducerContext,
    source_sobj_id: u64,
    target_sobj_id: u64,
    is_missile: bool,
) -> Result<(), String> {
    match test_combat_validation(ctx, source_sobj_id, target_sobj_id, is_missile) {
        Ok(message) => {
            spacetimedb::log::info!("Combat validation test passed: {}", message);
            Ok(())
        }
        Err(error) => {
            spacetimedb::log::info!("Combat validation test failed: {}", error.to_message());
            Err(error.to_message())
        }
    }
}

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
