use std::hash::Hasher;

use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Clone, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

// pub struct TradeCommand {
//     item_to_sell: ItemDefinitionId,
//     station: StationId,
// }

// Enum for AI states or player commands, can be expanded
#[derive(SpacetimeType, Clone, Debug, PartialEq, Hash)]
pub enum CurrentAction {
    Idle,
    Patrolling(Vec<Vec2>),
    MiningAsteroid(u64),  // target asteroid_id
    AttackingTarget(u64), // target sobj_id
    MovingToPosition(Vec2),
    JumpingWithGate(u64),       // target gate_id
    JumpingWithHyperdrive(u64), // target gate_id
    Docking(u64),               // target station_id
    Undocking(u64),             // target station_id
    Fleeing(u64),               // target sobj_id
    Trading(u64),               // target station_id
}

///////////////////////////////////////////////////////////
// Impl
///////////////////////////////////////////////////////////

impl PartialEq for Vec2 {
    fn eq(&self, other: &Self) -> bool {
        // Compare the bit patterns of the floats.
        // This means 0.0 and -0.0 are different, and NaN == NaN.
        self.x.to_bits() == other.x.to_bits() && self.y.to_bits() == other.y.to_bits()
    }
}

impl Eq for Vec2 {
    // The PartialEq impl fulfills Eq's requirements.
}

impl std::hash::Hash for Vec2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the bit patterns of the floats.
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl Vec2 {
    pub fn to_glam(&self) -> glam::Vec2 {
        glam::Vec2 {
            x: self.x,
            y: self.y,
        }
    }
    pub fn from_glam(vec: glam::Vec2) -> Vec2 {
        Vec2 { x: vec.x, y: vec.y }
    }

    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    pub fn sub(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn signed_angle_to(&self, other: &Vec2) -> f32 {
        // Calculate the angle from self to other
        let cross = self.x * other.y - self.y * other.x;
        let dot = self.x * other.x + self.y * other.y;
        cross.atan2(dot)
    }
}

impl From<glam::Vec2> for Vec2 {
    fn from(glam_vec: glam::Vec2) -> Self {
        Vec2 {
            x: glam_vec.x,
            y: glam_vec.y,
        }
    }
}

impl From<Vec2> for glam::Vec2 {
    fn from(vec: Vec2) -> Self {
        glam::Vec2 { x: vec.x, y: vec.y }
    }
}
