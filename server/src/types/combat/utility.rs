use spacetimedb::{ReducerContext, Timestamp};
use spacetimedsl::{dsl, Wrapper};

use crate::types::{
    items::{ItemDefinition, ItemMetadata},
    ships::*,
    stellarobjects::*,
};

use super::{
    CreateVisualEffectRow, CreateVisualEffectTimerRow, MissileType, VisualEffectType, WeaponType,
};

/// Represents the result of damage calculation
#[derive(Debug, Clone)]
pub struct DamageCalculation {
    pub base_damage: f32,
    pub shield_damage: f32,
    pub hull_damage: f32,
    pub energy_cost: f32,
}

impl DamageCalculation {
    /// Calculate damage values from weapon metadata
    pub fn calculate(weapon_metadata: &[ItemMetadata]) -> Self {
        let mut base_damage = 0.0;
        let mut shield_damage_mod = 1.0;
        let mut kinetic_damage_mod = 1.0;
        let mut base_damage_boost = 0.0;
        let mut energy_cost = 0.0;

        // Extract weapon stats from metadata
        for metadata in weapon_metadata {
            match metadata {
                ItemMetadata::BaseDamage(damage) => base_damage = *damage,
                ItemMetadata::ShieldDamageMod(modifier) => shield_damage_mod = *modifier,
                ItemMetadata::KineticDamageMod(modifier) => kinetic_damage_mod = *modifier,
                ItemMetadata::BaseDamageBoost(boost) => base_damage_boost = *boost,
                ItemMetadata::EnergyConsumption(cost) => energy_cost = *cost,
                _ => {} // Ignore other metadata types for now
            }
        }

        // Apply base damage boost
        let effective_base_damage = base_damage + base_damage_boost;

        // Calculate final damage values
        let shield_damage = effective_base_damage * shield_damage_mod;
        let hull_damage = effective_base_damage * kinetic_damage_mod;

        DamageCalculation {
            base_damage: effective_base_damage,
            shield_damage,
            hull_damage,
            energy_cost,
        }
    }

    /// Calculate the actual damage that will be applied to shields and hull
    /// Returns (shield_damage_applied, hull_damage_applied, target_destroyed)
    pub fn apply_to_target(&self, current_shields: f32, current_hull: f32) -> (f32, f32, bool) {
        let mut remaining_damage = self.shield_damage;
        let mut shield_damage_applied = 0.0;
        let mut hull_damage_applied = 0.0;

        // Apply damage to shields first
        if current_shields > 0.0 {
            shield_damage_applied = remaining_damage.min(current_shields);
            remaining_damage -= shield_damage_applied;
        }

        // Apply remaining damage to hull (converted to hull damage ratio)
        if remaining_damage > 0.0 && current_shields <= shield_damage_applied {
            // Convert remaining shield damage to hull damage using the damage ratio
            let hull_damage_ratio = self.hull_damage / self.shield_damage;
            hull_damage_applied = (remaining_damage * hull_damage_ratio).min(current_hull);
        }

        // Check if target is destroyed
        let target_destroyed = (current_hull - hull_damage_applied) <= 0.0;

        (shield_damage_applied, hull_damage_applied, target_destroyed)
    }
}

/// Process weapon fire with hitscan damage calculation
/// This function handles instant damage application for hitscan weapons
pub fn process_weapon_fire(
    ctx: &ReducerContext,
    source_sobj_id: u64,
    target_sobj_id: u64,
    actual_location: glam::Vec2, // Where exactly did the projectile explode, used for AoE weapons
    _weapon_type: WeaponType,
    weapon_item_def: ItemDefinition, // To get specific combat-related metadata
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Get source and target stellar objects
    let source_sobj = dsl.get_stellar_object_by_id(StellarObjectId::new(source_sobj_id))?;

    let target_sobj = dsl.get_stellar_object_by_id(StellarObjectId::new(target_sobj_id))?;

    // Validate target is a valid Ship or Station class
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

    // Get source ship to validate energy and get weapon configuration
    let source_ship = dsl
        .get_ships_by_sobj_id(StellarObjectId::new(source_sobj_id))
        .next()
        .ok_or_else(|| {
            format!(
                "Source ship not found for stellar object {}",
                source_sobj_id
            )
        })?;

    let mut source_ship_status = dsl.get_ship_status_by_id(&source_ship)?;

    // Get source and target positions and facing angles
    let source_transform = dsl.get_sobj_internal_transform_by_id(source_sobj.get_id())?;
    let target_transform = dsl.get_sobj_internal_transform_by_id(target_sobj.get_id())?;

    let source_pos = source_transform.to_vec2();
    let target_pos = target_transform.to_vec2();
    let ship_facing_angle = *source_transform.get_rotation_radians();

    // Extract lock-on angle bound from weapon metadata
    let mut lock_on_angle_bound = std::f32::consts::PI; // Default to 180 degrees if not specified
    for metadata in weapon_item_def.get_metadata() {
        if let ItemMetadata::LockOnAngleBoundRads(angle) = metadata {
            lock_on_angle_bound = *angle;
            break;
        }
    }

    // Check if target is within weapon range
    if !is_target_in_range(&source_pos, &target_pos, weapon_item_def.get_metadata()) {
        return Err("Target is out of weapon range".to_string());
    }

    // Get target ship dimensions for accurate collision detection
    let (target_width, target_height, target_orientation) =
        if target_sobj.get_kind() == &StellarObjectKinds::Ship {
            if let Some(target_ship) = dsl
                .get_ships_by_sobj_id(StellarObjectId::new(target_sobj_id))
                .next()
            {
                let target_ship_def =
                    dsl.get_ship_type_definition_by_id(target_ship.get_shiptype_id())?;
                (
                    *target_ship_def.get_sprite_width() as f32,
                    *target_ship_def.get_sprite_height() as f32,
                    *target_transform.get_rotation_radians(),
                )
            } else {
                // Fallback to default dimensions if ship not found
                (32.0, 32.0, 0.0)
            }
        } else {
            // For stations or other objects, use default dimensions for now
            // TODO: Add station dimensions when station combat is implemented
            (64.0, 64.0, 0.0)
        };

    // Check if target is within lock-on angle using actual target dimensions
    if !is_within_lock_on_angle(
        &source_pos,
        &target_pos,
        ship_facing_angle,
        lock_on_angle_bound,
        target_width,
        target_height,
        target_orientation,
    ) {
        return Err("Target is outside weapon lock-on angle".to_string());
    }

    // For hitscan weapons, check line of sight and intersection
    if let WeaponType::Hitscan = _weapon_type {
        if !has_line_of_sight(&source_pos, &target_pos, target_width.max(target_height)) {
            return Err("No line of sight to target".to_string());
        }

        if !hitscan_intersects_target_with_size(
            &source_pos,
            &target_pos,
            ship_facing_angle,
            target_width,
            target_height,
            target_orientation,
        ) {
            return Err("Hitscan shot misses target".to_string());
        }
    }

    // Calculate damage using weapon metadata
    let damage_calc = DamageCalculation::calculate(weapon_item_def.get_metadata());

    // Check if ship has sufficient energy
    if *source_ship_status.get_energy() < damage_calc.energy_cost {
        return Err("Insufficient energy to fire weapon".to_string());
    }

    // Consume energy
    source_ship_status.set_energy(*source_ship_status.get_energy() - damage_calc.energy_cost);

    // Apply damage to target if it's a ship
    if target_sobj.get_kind() == &StellarObjectKinds::Ship {
        if let Some(target_ship) = dsl
            .get_ships_by_sobj_id(StellarObjectId::new(target_sobj_id))
            .next()
        {
            let mut target_ship_status =
                dsl.get_ship_status_by_id(ShipId::new(target_ship.get_id().value()))?;

            // Apply damage using the enhanced damage calculation system
            let target_destroyed =
                apply_damage_to_ship(ctx, &mut target_ship_status, &damage_calc)?;

            if target_destroyed {
                spacetimedb::log::info!(
                    "Ship {} destroyed by weapon fire from ship {}",
                    target_ship.get_id().value(),
                    source_sobj_id
                );
            }

            // Update target ship status
            dsl.update_ship_status_by_id(target_ship_status)?;
        }
    }
    // TODO: Handle Station targets (will be implemented when station combat is added)

    // Update source ship status (energy consumption)
    dsl.update_ship_status_by_id(source_ship_status)?;

    // Create visual effect
    create_visual_effect(
        ctx,
        source_pos,
        actual_location.into(),
        VisualEffectType::WeaponFire,
    )?;

    spacetimedb::log::info!(
        "Weapon fired: {} -> {} (damage: {}, energy cost: {})",
        source_sobj_id,
        target_sobj_id,
        damage_calc.base_damage,
        damage_calc.energy_cost
    );

    Ok(())
}

/// Process missile fire as placeholder for future missile system
/// This function will be expanded when the missile system is implemented
pub fn process_missile_fire(
    ctx: &ReducerContext,
    source_sobj_id: u64,
    target_sobj_id: u64,
    actual_location: glam::Vec2, // Where exactly did the missile explode, used for AoE missiles
    missile_type: MissileType,
    missile_item_def: ItemDefinition, // To get specific combat-related metadata
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Get source stellar object for position
    let source_sobj = dsl.get_stellar_object_by_id(StellarObjectId::new(source_sobj_id))?;

    // Validate target exists (even though we're not implementing full missile logic yet)
    let _target_sobj = dsl.get_stellar_object_by_id(StellarObjectId::new(target_sobj_id))?;

    // Get source ship to validate energy
    let source_ship = dsl
        .get_ships_by_sobj_id(StellarObjectId::new(source_sobj_id))
        .next()
        .ok_or_else(|| {
            format!(
                "Source ship not found for stellar object {}",
                source_sobj_id
            )
        })?;

    let mut source_ship_status =
        dsl.get_ship_status_by_id(ShipId::new(source_ship.get_id().value()))?;

    // Calculate energy cost from missile metadata
    let mut energy_cost = 0.0;
    for metadata in missile_item_def.get_metadata() {
        if let ItemMetadata::EnergyConsumption(cost) = metadata {
            energy_cost = *cost;
            break;
        }
    }

    // Check if ship has sufficient energy
    if *source_ship_status.get_energy() < energy_cost {
        return Err("Insufficient energy to fire missile".to_string());
    }

    // Consume energy
    source_ship_status.set_energy(*source_ship_status.get_energy() - energy_cost);
    dsl.update_ship_status_by_id(source_ship_status)?;

    // Create visual effect for missile fire
    let source_transform = dsl.get_sobj_internal_transform_by_id(source_sobj.get_id())?;
    let source_pos = source_transform.to_vec2();
    create_visual_effect(
        ctx,
        source_pos,
        actual_location,
        VisualEffectType::MissileFire,
    )?;

    spacetimedb::log::info!(
        "Missile fired: {} -> {} (type: {:?}, energy cost: {})",
        source_sobj_id,
        target_sobj_id,
        missile_type,
        energy_cost
    );

    // TODO: Create actual missile stellar object and implement tracking logic
    // This will be implemented in future tasks when the full missile system is added

    Ok(())
}

/// Apply calculated damage to a ship and handle destruction
/// Returns true if the target was destroyed
pub fn apply_damage_to_ship(
    ctx: &ReducerContext,
    target_ship_status: &mut ShipStatus,
    damage_calc: &DamageCalculation,
) -> Result<bool, String> {
    let current_shields = *target_ship_status.get_shields();
    let current_hull = *target_ship_status.get_health();

    // Calculate actual damage application
    let (shield_damage_applied, hull_damage_applied, target_destroyed) =
        damage_calc.apply_to_target(current_shields, current_hull);

    // Apply shield damage
    let new_shields = (current_shields - shield_damage_applied).max(0.0);
    target_ship_status.set_shields(new_shields);

    // Apply hull damage
    let new_hull = (current_hull - hull_damage_applied).max(0.0);
    target_ship_status.set_health(new_hull);

    // Log damage application
    spacetimedb::log::info!(
        "Damage applied - Shield: {:.1} -> {:.1} (-{:.1}), Hull: {:.1} -> {:.1} (-{:.1})",
        current_shields,
        new_shields,
        shield_damage_applied,
        current_hull,
        new_hull,
        hull_damage_applied
    );

    // Handle target destruction
    if target_destroyed {
        handle_ship_destruction(ctx, target_ship_status)?;
    }

    Ok(target_destroyed)
}

/// Handle ship destruction when hull health reaches zero
fn handle_ship_destruction(_ctx: &ReducerContext, ship_status: &ShipStatus) -> Result<(), String> {
    spacetimedb::log::info!("Ship {} destroyed in combat", ship_status.get_id().value());

    // TODO: Implement full ship destruction logic in future tasks:
    // - Drop cargo as cargo crates
    // - Create explosion visual effect
    // - Award experience/credits to attacker
    // - Handle respawn mechanics for players
    // - Remove ship from active gameplay (but keep in database for history)

    // For now, just set health to 0 and log the destruction
    // The ship will be handled by other systems (respawn, cleanup, etc.)

    Ok(())
}

/// Calculate damage effectiveness against a specific target
/// Returns a multiplier based on target type and weapon characteristics
pub fn calculate_damage_effectiveness(
    weapon_metadata: &[ItemMetadata],
    target_type: &StellarObjectKinds,
) -> f32 {
    // Different weapon types have different effectiveness against different targets
    // This can be expanded in the future for more complex damage calculations
    let effectiveness = match target_type {
        StellarObjectKinds::Ship => {
            // Ships take normal damage from all weapons
            1.0
        }
        StellarObjectKinds::Station => {
            // Stations might have different armor characteristics
            // For now, treat them the same as ships
            1.0
        }
        _ => {
            // Other object types (asteroids, cargo crates, etc.) might take different damage
            0.5 // Reduced effectiveness against non-combat targets
        }
    };

    // Check for special weapon effects that might modify effectiveness
    for metadata in weapon_metadata {
        match metadata {
            ItemMetadata::SpecialEffect(effect) => {
                // Future: implement special weapon effects
                // For example: "anti_armor", "shield_piercing", etc.
                spacetimedb::log::info!("Special weapon effect: {}", effect);
            }
            _ => {}
        }
    }

    effectiveness
}

/// Check if a hitscan weapon has line-of-sight to the target
/// Uses actual ship dimensions for collision detection
pub fn has_line_of_sight(
    source_pos: &glam::Vec2,
    target_pos: &glam::Vec2,
    target_max_dimension: f32,
) -> bool {
    // For now, we assume clear line of sight since we don't have obstacles
    // In the future, this could check for asteroids, stations, or other ships blocking the shot

    // Calculate distance to ensure we're not shooting at ourselves
    let distance = (target_pos.x - source_pos.x).powi(2) + (target_pos.y - source_pos.y).powi(2);

    // Minimum distance to prevent self-targeting (target size + small buffer)
    let min_distance = target_max_dimension + 8.0; // target size + 8px buffer
    distance > min_distance.powi(2)
}

/// Check if target intersects with hitscan line using actual ship dimensions and orientation
pub fn hitscan_intersects_target_with_size(
    source_pos: &glam::Vec2,
    target_pos: &glam::Vec2,
    ship_facing_angle: f32,
    target_width: f32,
    target_height: f32,
    target_orientation: f32,
) -> bool {
    // Calculate the hitscan ray direction
    let ray_dx = ship_facing_angle.cos();
    let ray_dy = ship_facing_angle.sin();

    // Get the target's actual corners based on its dimensions and orientation
    let target_corners =
        get_target_corners(target_pos, target_width, target_height, target_orientation);

    // Check if the ray intersects with any edge of the target's oriented bounding box
    // We'll use a simplified approach: check if the ray intersects the axis-aligned bounding box
    // of the rotated target corners
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for corner in &target_corners {
        min_x = min_x.min(corner.x);
        max_x = max_x.max(corner.x);
        min_y = min_y.min(corner.y);
        max_y = max_y.max(corner.y);
    }

    // Check if ray intersects with the axis-aligned bounding box of the rotated target
    let t_min_x = if ray_dx != 0.0 {
        (min_x - source_pos.x) / ray_dx
    } else if source_pos.x >= min_x && source_pos.x <= max_x {
        0.0
    } else {
        f32::INFINITY
    };

    let t_max_x = if ray_dx != 0.0 {
        (max_x - source_pos.x) / ray_dx
    } else if source_pos.x >= min_x && source_pos.x <= max_x {
        f32::INFINITY
    } else {
        -f32::INFINITY
    };

    let t_min_y = if ray_dy != 0.0 {
        (min_y - source_pos.y) / ray_dy
    } else if source_pos.y >= min_y && source_pos.y <= max_y {
        0.0
    } else {
        f32::INFINITY
    };

    let t_max_y = if ray_dy != 0.0 {
        (max_y - source_pos.y) / ray_dy
    } else if source_pos.y >= min_y && source_pos.y <= max_y {
        f32::INFINITY
    } else {
        -f32::INFINITY
    };

    let t_enter_x = t_min_x.min(t_max_x);
    let t_exit_x = t_min_x.max(t_max_x);
    let t_enter_y = t_min_y.min(t_max_y);
    let t_exit_y = t_min_y.max(t_max_y);

    let t_enter = t_enter_x.max(t_enter_y);
    let t_exit = t_exit_x.min(t_exit_y);

    // Ray intersects if t_enter <= t_exit and t_exit >= 0
    t_enter <= t_exit && t_exit >= 0.0 && t_enter >= 0.0
}

/// Enhanced version that takes target dimensions and orientation into account
pub fn is_within_lock_on_angle(
    source_pos: &glam::Vec2,
    target_pos: &glam::Vec2,
    ship_facing_angle: f32,
    lock_on_angle_bound_rads: f32,
    target_width: f32,
    target_height: f32,
    target_orientation: f32,
) -> bool {
    // Edge case: If ships are at the same position, it's ambiguous, but usually considered pointing.
    if source_pos.x == target_pos.x && source_pos.y == target_pos.y {
        return true;
    }

    let your_forward_vec = glam::Vec2::new(ship_facing_angle.cos(), ship_facing_angle.sin());

    // Calculate target corners based on actual dimensions and orientation
    let target_corners =
        get_target_corners(target_pos, target_width, target_height, target_orientation);

    let mut min_angle_to_target_extremity = f32::MAX;
    let mut max_angle_to_target_extremity = f32::MIN;

    for corner in &target_corners {
        let vec_to_corner = corner - source_pos;
        if vec_to_corner.length() == 0.0 {
            // Your ship is exactly on a corner of the target, definitely pointing.
            return true;
        }

        // Calculate signed angle using cross product and dot product
        let cross = your_forward_vec.x * vec_to_corner.y - your_forward_vec.y * vec_to_corner.x;
        let dot = your_forward_vec.dot(vec_to_corner);
        let angle = cross.atan2(dot);

        min_angle_to_target_extremity = min_angle_to_target_extremity.min(angle);
        max_angle_to_target_extremity = max_angle_to_target_extremity.max(angle);
    }

    // Check for overlap between the target's angular range and the weapon's cone
    if min_angle_to_target_extremity <= max_angle_to_target_extremity {
        // Target does not wrap around +/- PI. Standard overlap check.
        lock_on_angle_bound_rads >= min_angle_to_target_extremity
            && -lock_on_angle_bound_rads <= max_angle_to_target_extremity
    } else {
        // Target wraps around +/- PI (e.g., from +170 to -170). This means it includes 0.
        // Your cone also includes 0. So there's an overlap unless your cone is entirely
        // within the 'gap' of the target's wrap.
        !(lock_on_angle_bound_rads < min_angle_to_target_extremity
            && -lock_on_angle_bound_rads > max_angle_to_target_extremity)
    }
}

/// Helper function to calculate target corners based on position, dimensions, and orientation
fn get_target_corners(
    position: &glam::Vec2,
    width: f32,
    height: f32,
    orientation: f32,
) -> [glam::Vec2; 4] {
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let cos_angle = orientation.cos();
    let sin_angle = orientation.sin();

    // Corners relative to target's center, assuming 0 orientation
    let corners_local = [
        (half_width, half_height),   // top_right
        (-half_width, half_height),  // top_left
        (-half_width, -half_height), // bottom_left
        (half_width, -half_height),  // bottom_right
    ];

    // Rotate and translate to world space
    corners_local.map(|(x, y)| {
        let rotated_x = x * cos_angle - y * sin_angle;
        let rotated_y = x * sin_angle + y * cos_angle;
        glam::Vec2::new(position.x + rotated_x, position.y + rotated_y)
    })
}

/// Backward compatibility function for hitscan_intersects_target
pub fn hitscan_intersects_target(
    source_pos: &glam::Vec2,
    target_pos: &glam::Vec2,
    ship_facing_angle: f32,
    target_half_size: f32,
) -> bool {
    let target_size = target_half_size * 2.0;
    hitscan_intersects_target_with_size(
        source_pos,
        target_pos,
        ship_facing_angle,
        target_size,
        target_size,
        0.0, // Default orientation
    )
}

/// Validate if a target is within weapon range
pub fn is_target_in_range(
    source_pos: &glam::Vec2,
    target_pos: &glam::Vec2,
    weapon_metadata: &[ItemMetadata],
) -> bool {
    let mut max_range = f32::INFINITY; // Default to infinite range if not specified

    // Extract maximum range from weapon metadata
    for metadata in weapon_metadata {
        if let ItemMetadata::MaximumRange(range) = metadata {
            max_range = *range;
            break;
        }
    }

    // Calculate distance_squared to target. Avoid using sqrt() for long distance reasons
    let distance_squared =
        (target_pos.x - source_pos.x).powi(2) + (target_pos.y - source_pos.y).powi(2);

    distance_squared <= max_range.powi(2)
}

/// Create a visual effect and schedule its cleanup
fn create_visual_effect(
    ctx: &ReducerContext,
    source_pos: glam::Vec2,
    target_pos: glam::Vec2,
    effect_type: VisualEffectType,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Create visual effect
    let visual_effect =
        dsl.create_visual_effect(source_pos.into(), target_pos.into(), effect_type)?;

    // Schedule cleanup after 10 milliseconds
    let cleanup_time = spacetimedb::ScheduleAt::Time(Timestamp::from_micros_since_unix_epoch(
        ctx.timestamp.to_micros_since_unix_epoch() + 10_000,
    ));

    dsl.create_visual_effect_timer(visual_effect.get_id(), cleanup_time)?;

    Ok(())
}
