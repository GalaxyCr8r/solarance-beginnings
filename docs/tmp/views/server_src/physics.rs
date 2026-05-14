use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq)]
pub struct InputState {
    pub is_thrusting: bool,
    pub is_breaking: bool,
    pub turn_direction: i8,
}

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq)]
pub struct MovementState {
    pub pos: Vec2,
    /// Pixels per second
    pub velocity: f32,
    /// Radians
    pub rotation: f32,
    /// Radians per second
    pub angular_velocity: f32,
    /// Microseconds
    pub last_update_time: i64,
    /// Pixels per second squared
    pub acceleration: f32,
    /// Radians per second squared
    pub angular_acceleration: f32,
    /// Pixels per second (velocity cap)
    pub max_speed: f32,
    /// Radians per second (angular velocity cap)
    pub max_turn_rate: f32,
    /// When true and angular_acceleration is zero, bleeds angular_velocity toward zero
    /// at max_turn_rate / 2 radians per second squared.
    pub dampen_angular_rotation: bool,
}

pub fn convert_to_movement_state(
    state: &MovementState,
) -> solarance_shared::physics::MovementState {
    solarance_shared::physics::MovementState {
        pos: solarance_shared::physics::Vec2 {
            x: state.pos.x,
            y: state.pos.y,
        },
        velocity: state.velocity,
        rotation: state.rotation,
        angular_velocity: state.angular_velocity,
        last_update_time: state.last_update_time,
        acceleration: state.acceleration,
        angular_acceleration: state.angular_acceleration,
        max_speed: state.max_speed,
        max_turn_rate: state.max_turn_rate,
        dampen_angular_rotation: state.dampen_angular_rotation,
    }
}

pub fn convert_from_movement_state(
    state: &solarance_shared::physics::MovementState,
) -> MovementState {
    MovementState {
        pos: Vec2 {
            x: state.pos.x,
            y: state.pos.y,
        },
        velocity: state.velocity,
        rotation: state.rotation,
        angular_velocity: state.angular_velocity,
        last_update_time: state.last_update_time,
        acceleration: state.acceleration,
        angular_acceleration: state.angular_acceleration,
        max_speed: state.max_speed,
        max_turn_rate: state.max_turn_rate,
        dampen_angular_rotation: state.dampen_angular_rotation,
    }
}
