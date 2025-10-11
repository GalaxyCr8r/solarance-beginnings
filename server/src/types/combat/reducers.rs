use spacetimedb::ReducerContext;
use spacetimedsl::{dsl, Wrapper};

use crate::types::{
    items::utility::get_item_definition, npcs::*, players::*, ships::*, stellarobjects::*,
};

use super::utility::{process_missile_fire, process_weapon_fire};
use super::{MissileType, ShipController, WeaponType};

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
                        spacetimedb::log::info!(
                            "Weapon fire failed for ship {}: {}",
                            source_sobj_id,
                            e
                        );
                    }
                }
            } else {
                spacetimedb::log::info!(
                    "Ship {} attempted to fire weapons without a target",
                    source_sobj_id
                );
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
                        spacetimedb::log::info!(
                            "Missile fire failed for ship {}: {}",
                            source_sobj_id,
                            e
                        );
                    }
                }
            } else {
                spacetimedb::log::info!(
                    "Ship {} attempted to fire missiles without a target",
                    source_sobj_id
                );
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
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Validate target is valid Ship or Station class
    let target_sobj = dsl.get_stellar_object_by_id(StellarObjectId::new(target_sobj_id))?;
    match target_sobj.get_kind() {
        StellarObjectKinds::Ship | StellarObjectKinds::Station => {
            // Valid target
        }
        _ => {
            return Err(format!(
                "Invalid target class: {:?}. Only Ship and Station can be targeted.",
                target_sobj.get_kind()
            ));
        }
    }

    // Get source ship to find equipped weapons
    let source_ship = dsl
        .get_ships_by_sobj_id(StellarObjectId::new(source_sobj_id))
        .next()
        .ok_or_else(|| {
            format!(
                "Source ship not found for stellar object {}",
                source_sobj_id
            )
        })?;

    // Find equipped weapons in weapon slots
    let weapon_slots: Vec<ShipEquipmentSlot> = dsl
        .get_ship_equipment_slots_by_ship_id(source_ship.get_id())
        .filter(|slot| slot.get_slot_type() == &EquipmentSlotType::Weapon)
        .collect();

    if weapon_slots.is_empty() {
        return Err("No weapons equipped".to_string());
    }

    // Get target position for actual_location parameter
    let target_transform = dsl.get_sobj_internal_transform_by_id(target_sobj.get_id())?;
    let target_pos = target_transform.to_vec2();

    // Fire each equipped weapon
    for weapon_slot in weapon_slots {
        let weapon_def = get_item_definition(ctx, weapon_slot.get_item_id().value())?;

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
                    e
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
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Validate target is valid Ship or Station class
    let target_sobj = dsl.get_stellar_object_by_id(StellarObjectId::new(target_sobj_id))?;
    match target_sobj.get_kind() {
        StellarObjectKinds::Ship | StellarObjectKinds::Station => {
            // Valid target
        }
        _ => {
            return Err(format!(
                "Invalid target class: {:?}. Only Ship and Station can be targeted.",
                target_sobj.get_kind()
            ));
        }
    }

    // Get source ship to find equipped missiles
    let source_ship = dsl
        .get_ships_by_sobj_id(StellarObjectId::new(source_sobj_id))
        .next()
        .ok_or_else(|| {
            format!(
                "Source ship not found for stellar object {}",
                source_sobj_id
            )
        })?;

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
    let target_transform = dsl.get_sobj_internal_transform_by_id(target_sobj.get_id())?;
    let target_pos = target_transform.to_vec2();

    // Fire each equipped missile
    for missile_slot in missile_slots {
        let missile_def = get_item_definition(ctx, missile_slot.get_item_id().value())?;

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
                    e
                );
                // Continue with other missiles even if one fails
            }
        }
    }

    Ok(())
}
