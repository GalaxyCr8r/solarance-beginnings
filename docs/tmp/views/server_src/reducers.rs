use log::info;
use solarance_shared::physics::predict_movement;
use spacetimedb::*;
use spacetimedsl::*;

use crate::{physics::*, sectors::observe_all_public_sectors, tables::*};

#[reducer]
pub fn spawn_ship(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = spacetimedsl::dsl(ctx);

    // Spawn a ship for the player if they don't have one
    if dsl.get_space_ship_by_player_id(&ctx.sender()).is_err() {
        // Get ship configuration to copy max_speed and max_turn_rate
        let config = dsl
            .get_ship_config_by_id(ShipConfigId::new(1))
            .expect("Default ship config not found");

        dsl.create_space_ship(CreateSpaceShip {
            player_id: ctx.sender(),
            ship_config_id: config.get_id().clone(),
            health: *config.get_max_health() as f32,
            sector_id: SectorId::new(1),
            movement: MovementState {
                pos: Vec2 { x: 0.0, y: 0.0 },
                velocity: 0.0,
                rotation: 0.0,
                angular_velocity: 0.0,
                last_update_time: ctx.timestamp.to_micros_since_unix_epoch(),
                acceleration: 0.0,
                angular_acceleration: 0.0,
                max_speed: *config.get_max_speed(),
                max_turn_rate: *config.get_max_turn_rate(),
                dampen_angular_rotation: true,
            },
            input_state: InputState {
                is_thrusting: false,
                is_breaking: false,
                turn_direction: 0,
            },
            last_fired: ctx.timestamp.clone(),
        })?;

        dsl.create_player_state(CreatePlayerState {
            id: ctx.sender(),
            current_system_id: 1,
            current_sector_id: 1,
        })?;

        dsl.create_visited_sector(CreateVisitedSector {
            player_id: ctx.sender(),
            sector_id: 1,
            visited_status: VisitedStatus::Visited,
        })?;

        observe_all_public_sectors(ctx)?;

        dsl.create_visited_system(CreateVisitedSystem {
            player_id: ctx.sender(),
            system_id: 1,
            visited_status: VisitedStatus::Visited,
        })?;
    }

    Ok(())
}

#[reducer]
pub fn fire_weapons(ctx: &ReducerContext, fired_at: i64) -> Result<(), String> {
    let dsl = spacetimedsl::dsl(ctx);

    let mut space_ship = dsl
        .get_space_ship_by_player_id(&ctx.sender())
        .map_err(|_| "Ship not found")?;

    // Calculate the ship's position at the time of firing
    let ship_movement = convert_to_movement_state(&space_ship.movement);
    let (ship_pos, ship_rot) = if fired_at < space_ship.movement.last_update_time {
        (ship_movement.pos, ship_movement.rotation)
    } else {
        let (p, r, _, _) = predict_movement(&ship_movement, fired_at);
        (p, r)
    };

    // Verify that this isn't firing too soon
    match space_ship
        .get_last_fired()
        .checked_add(TimeDuration::from_micros(500_000))
    {
        Some(last_fired) => {
            if last_fired > ctx.timestamp {
                return Err("Tried firing too soon".to_string());
            }
        }
        None => return Err("Couldn't convert last fired".to_string()),
    };

    // Create bullet
    dsl.create_bullet(CreateBullet {
        player_id: ctx.sender(),
        sector_id: space_ship.get_sector_id(),
        damage: 10.0,
        lifetime: fired_at + 1_000_000,
        movement: MovementState {
            pos: Vec2 {
                x: ship_pos.x,
                y: ship_pos.y,
            },
            velocity: 250.0,
            rotation: ship_rot,
            angular_velocity: 0.0,
            last_update_time: fired_at,
            acceleration: 0.0,
            angular_acceleration: 0.0,
            max_speed: 500.0,
            max_turn_rate: 0.0,
            dampen_angular_rotation: true,
        },
    })?;

    // Update ship's last fired
    space_ship.set_last_fired(Timestamp::from_micros_since_unix_epoch(fired_at));
    dsl.update_space_ship_by_id(space_ship)?;

    Ok(())
}

/// Called by clients in the same sector as the bullet when they detect a bullet hit any ship in that sector.
/// This is used to verify that the bullet actually hit the ship.
#[reducer]
pub fn submit_hit(
    ctx: &ReducerContext,
    hit_at: Timestamp,
    hit_ship_id: SpaceShipId,
    bullet_id: BulletId,
) -> Result<(), String> {
    let dsl = spacetimedsl::dsl(ctx);

    let reporter_ship = dsl
        .get_space_ship_by_player_id(&ctx.sender())
        .map_err(|_| "Ship not found")?;

    let mut hit_ship = dsl
        .get_space_ship_by_id(hit_ship_id)
        .map_err(|_| "Ship not found")?;

    if reporter_ship.get_sector_id() != hit_ship.get_sector_id() {
        return Err("Tried to submit a bullet hit a ship in a different sector!".to_string());
    }

    let bullet = dsl
        .get_bullet_by_id(&bullet_id)
        .map_err(|_| "Bullet not found")?;

    if hit_ship.get_player_id() == bullet.get_player_id() {
        return Err("Tried to submit a bullet hit on shooter ship!".to_string());
    }

    info!(
        "Bullet lifetime: {}",
        bullet.lifetime - ctx.timestamp.to_micros_since_unix_epoch()
    );

    if bullet.lifetime < ctx.timestamp.to_micros_since_unix_epoch() {
        info!("Bullet has expired!");
        dsl.delete_bullet_by_id(bullet_id)?;
        return Err("Expired".to_string());
    }

    // If hit_at is before hit_ship's last movement timestamp, then use it's current originator posiiton.
    let hit_ship_movement = convert_to_movement_state(&hit_ship.movement);
    let hit_ship_pos = if hit_at.to_micros_since_unix_epoch() < hit_ship.movement.last_update_time {
        info!("Using hit_ship's current originator position");
        hit_ship_movement.pos
    } else {
        info!("Predicting movement of the hit ship!");
        predict_movement(&hit_ship_movement, hit_at.to_micros_since_unix_epoch()).0
    };
    info!("Using position! ({}, {})", hit_ship_pos.x, hit_ship_pos.y);

    // Predict the hit_ship's position and bullet's position at timestamp
    let bullet_movement = convert_to_movement_state(&bullet.movement);
    let bullet_pos = if hit_at.to_micros_since_unix_epoch() < bullet.movement.last_update_time {
        info!("Using bullet's current originator position");
        bullet_movement.pos
    } else {
        info!("Predicting movement of the bullet!");
        predict_movement(&bullet_movement, hit_at.to_micros_since_unix_epoch()).0
    };

    // Check if they are near each other
    let distance = hit_ship_pos.distance_to_sq(&bullet_pos);
    if distance > 32.0 * 32.0 {
        return Err(format!("Bullet missed! Distance Squared: {}", distance));
    }
    info!("Bullet hit! Distance Squared: {}", distance);

    // TODO: Abstract out damage calculation to a function
    hit_ship.health -= bullet.damage;

    if hit_ship.health <= 0.0 {
        dsl.delete_space_ship_by_id(&hit_ship)?;
    } else {
        dsl.update_space_ship_by_id(hit_ship.clone())?;
    }

    // If we got this far and dealt damage, then this bullet should be deleted.
    dsl.delete_bullet_by_id(bullet_id)?;

    // Create damage event
    dsl.create_damage_event(CreateDamageEvent {
        sector_id: hit_ship.get_sector_id(),
        event_type: EventType::Bullet,
        pos: Vec2 {
            x: hit_ship_pos.x,
            y: hit_ship_pos.y,
        },
        timestamp: ctx.timestamp.to_micros_since_unix_epoch(),
    })?;

    Ok(())
}

#[reducer]
pub fn travel_to_sector(ctx: &ReducerContext, sector_id: u64) -> Result<(), String> {
    let dsl = spacetimedsl::dsl(ctx);

    let mut space_ship = dsl
        .get_space_ship_by_player_id(&ctx.sender())
        .map_err(|_| "Ship not found")?;

    let mut player_state = dsl
        .get_player_state_by_id(PlayerStateId::new(ctx.sender()))
        .map_err(|_| "player_state not found")?;

    let target_sector = dsl
        .get_sector_by_id(SectorId::new(sector_id))
        .map_err(|_| "Sector not found")?;

    let current_sector = dsl
        .get_sector_by_id(space_ship.get_sector_id().clone())
        .map_err(|_| "Current sector not found")?;

    if current_sector.get_system_id() != target_sector.get_system_id() {
        return Err("Cannot travel between systems".to_string());
    }

    if !dsl
        .get_visited_sectors_by_player_id(&ctx.sender())
        .any(|v| *v.get_sector_id() == sector_id)
    {
        let _ = dsl.create_visited_sector(CreateVisitedSector {
            player_id: ctx.sender(),
            sector_id: sector_id,
            visited_status: VisitedStatus::Visited,
        });
    }

    space_ship.sector_id = sector_id;
    player_state.current_sector_id = sector_id;
    dsl.update_space_ship_by_id(space_ship)?;
    dsl.update_player_state_by_id(player_state)?;
    Ok(())
}

#[reducer]
pub fn set_forward_thrust(ctx: &ReducerContext, meters_per_second: f32) -> Result<(), String> {
    let dsl = spacetimedsl::dsl(ctx);

    let mut space_ship = dsl
        .get_space_ship_by_player_id(&ctx.sender())
        .map_err(|_| "Ship not found")?;

    let stats = dsl
        .get_ship_config_by_id(ShipConfigId::new(space_ship.ship_config_id))
        .map_err(|_| "Ship stats not found")?;

    // 1. Enforce Server-Side Speed Limits
    let clamped_speed = meters_per_second.clamp(0.0, *stats.get_max_speed());
    if clamped_speed == space_ship.movement.velocity {
        return Ok(());
    }

    // 2. Synchronize current position BEFORE changing trajectory
    let (current_pos, current_rot, _, _) = predict_movement(
        &convert_to_movement_state(&space_ship.movement),
        ctx.timestamp.to_micros_since_unix_epoch(),
    );

    // 3. Update the movement state
    space_ship.movement = MovementState {
        pos: Vec2 {
            x: current_pos.x,
            y: current_pos.y,
        },
        velocity: clamped_speed,
        rotation: current_rot,
        angular_velocity: space_ship.movement.angular_velocity,
        last_update_time: ctx.timestamp.to_micros_since_unix_epoch(),
        acceleration: space_ship.movement.acceleration,
        angular_acceleration: space_ship.movement.angular_acceleration,
        max_speed: space_ship.movement.max_speed,
        max_turn_rate: space_ship.movement.max_turn_rate,
        dampen_angular_rotation: space_ship.movement.dampen_angular_rotation,
    };

    // 4. Update Database
    dsl.update_space_ship_by_id(space_ship);
    Ok(())
}

#[reducer]
pub fn set_turn_velocity(ctx: &ReducerContext, radians_per_second: f32) -> Result<(), String> {
    let dsl = spacetimedsl::dsl(ctx);

    let mut space_ship = dsl
        .get_space_ship_by_player_id(&ctx.sender())
        .map_err(|_| "Ship not found")?;

    let stats = dsl
        .get_ship_config_by_id(ShipConfigId::new(space_ship.ship_config_id))
        .map_err(|_| "Ship stats not found")?;

    // 1. Enforce Turn Limits
    let mut clamped_turn =
        radians_per_second.clamp(-*stats.get_max_turn_rate(), *stats.get_max_turn_rate());
    // ~0.25 deg/s expressed in radians — below this we treat the ship as not turning.
    if clamped_turn.abs() < 0.25_f32.to_radians() {
        clamped_turn = 0.0;
    }

    if clamped_turn == space_ship.movement.angular_velocity {
        return Ok(());
    }

    // 2. Synchronize current position/rotation
    let (current_pos, current_rot, _, _) = predict_movement(
        &convert_to_movement_state(&space_ship.movement),
        ctx.timestamp.to_micros_since_unix_epoch(),
    );

    // 3. Update trajectory
    space_ship.movement = MovementState {
        pos: Vec2 {
            x: current_pos.x,
            y: current_pos.y,
        },
        velocity: space_ship.movement.velocity,
        rotation: current_rot,
        angular_velocity: clamped_turn,
        last_update_time: ctx.timestamp.to_micros_since_unix_epoch(),
        acceleration: space_ship.movement.acceleration,
        angular_acceleration: space_ship.movement.angular_acceleration,
        max_speed: space_ship.movement.max_speed,
        max_turn_rate: space_ship.movement.max_turn_rate,
        dampen_angular_rotation: space_ship.movement.dampen_angular_rotation,
    };

    dsl.update_space_ship_by_id(space_ship);
    Ok(())
}

#[reducer]
pub fn set_thrust_input(
    ctx: &ReducerContext,
    is_thrusting: bool,
    is_breaking: bool,
) -> Result<(), String> {
    let dsl = spacetimedsl::dsl(ctx);

    let mut space_ship = dsl
        .get_space_ship_by_player_id(&ctx.sender())
        .map_err(|_| "Ship not found")?;

    // Early return if input hasn't changed (Req 3.8)
    if is_thrusting {
        if space_ship.input_state.is_thrusting == is_thrusting
            && space_ship.input_state.is_breaking == false
        {
            return Ok(());
        }
    } else if is_breaking {
        if space_ship.input_state.is_thrusting == false
            && space_ship.input_state.is_breaking == is_breaking
        {
            return Ok(());
        }
    } else if space_ship.input_state.is_thrusting == space_ship.input_state.is_breaking {
        return Ok(());
    }

    let config = dsl
        .get_ship_config_by_id(ShipConfigId::new(space_ship.ship_config_id))
        .map_err(|_| "Ship config not found")?;

    let now = ctx.timestamp.to_micros_since_unix_epoch();

    // 1. Predict current position, rotation, and velocities from the unified simulation.
    let (predicted_pos, predicted_rot, predicted_velocity, predicted_angular_velocity) =
        predict_movement(&convert_to_movement_state(&space_ship.movement), now);

    // 2. Calculate new acceleration based on thrust input
    let new_acceleration = if is_thrusting {
        *config.get_max_acceleration()
    } else if is_breaking {
        -*config.get_max_acceleration()
    } else {
        0.0 // Ship coasts at current velocity
    };

    // 4. Update input state and movement
    space_ship.input_state.is_thrusting = is_thrusting;
    space_ship.input_state.is_breaking = if !is_thrusting { is_breaking } else { false };
    space_ship.movement = MovementState {
        pos: Vec2 {
            x: predicted_pos.x,
            y: predicted_pos.y,
        },
        velocity: predicted_velocity,
        rotation: predicted_rot,
        angular_velocity: predicted_angular_velocity,
        acceleration: new_acceleration,
        angular_acceleration: space_ship.movement.angular_acceleration,
        last_update_time: now,
        max_speed: *config.get_max_speed(),
        max_turn_rate: *config.get_max_turn_rate(),
        dampen_angular_rotation: true,
    };

    dsl.update_space_ship_by_id(space_ship);
    Ok(())
}

#[reducer]
pub fn set_turn_input(ctx: &ReducerContext, turn_direction: i8) -> Result<(), String> {
    let dsl = spacetimedsl::dsl(ctx);

    // Validate turn_direction
    if turn_direction != -1 && turn_direction != 0 && turn_direction != 1 {
        return Err(format!(
            "Invalid turn_direction: {}. Must be -1, 0, or 1",
            turn_direction
        ));
    }

    let mut space_ship = dsl
        .get_space_ship_by_player_id(&ctx.sender())
        .map_err(|_| "Ship not found")?;

    // Early return if input hasn't changed (Req 3.8)
    if space_ship.input_state.turn_direction == turn_direction {
        return Ok(());
    }

    let config = dsl
        .get_ship_config_by_id(ShipConfigId::new(space_ship.ship_config_id))
        .map_err(|_| "Ship config not found")?;

    let now = ctx.timestamp.to_micros_since_unix_epoch();

    // 1. Predict current position, rotation, and velocities from the unified simulation.
    let (predicted_pos, predicted_rot, predicted_velocity, predicted_angular_velocity) =
        predict_movement(&convert_to_movement_state(&space_ship.movement), now);

    // 2. Calculate new angular acceleration based on turn direction
    let new_angular_acceleration = turn_direction as f32 * *config.get_max_angular_acceleration();

    // 4. Update input state and movement
    space_ship.input_state.turn_direction = turn_direction;
    space_ship.movement = MovementState {
        pos: Vec2 {
            x: predicted_pos.x,
            y: predicted_pos.y,
        },
        velocity: predicted_velocity,
        rotation: predicted_rot,
        angular_velocity: predicted_angular_velocity,
        acceleration: space_ship.movement.acceleration,
        angular_acceleration: new_angular_acceleration,
        last_update_time: now,
        max_speed: *config.get_max_speed(),
        max_turn_rate: *config.get_max_turn_rate(),
        dampen_angular_rotation: true,
    };

    dsl.update_space_ship_by_id(space_ship)?;
    Ok(())
}
