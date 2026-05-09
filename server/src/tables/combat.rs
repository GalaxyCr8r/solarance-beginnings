use spacetimedb::{table, SpacetimeType, Timestamp};
use spacetimedsl::*;

use crate::tables::common_types::Vec2;

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum VisualEffectType {
    WeaponFire,
    MissileFire,
    Explosion,
}

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum WeaponType {
    /// Most autocannons/blasters
    Hitscan,
    /// Slow/dodgable fire
    Projectile,
    /// e.g. Flak
    AreaOfEffect,
}

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum MissileType {
    /// Set angle
    Dumbfire,
    /// Follows a stellar object
    Heatseeking,
}

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum CombatError {
    InsufficientEnergy,
    InvalidTarget,
    WeaponNotEquipped,
    OutOfRange,
}

impl CombatError {
    pub fn to_message(&self) -> String {
        match self {
            CombatError::InsufficientEnergy => "Insufficient energy to fire weapon".to_string(),
            CombatError::InvalidTarget => {
                "Invalid target - only ships and stations can be targeted".to_string()
            }
            CombatError::WeaponNotEquipped => "No weapons equipped".to_string(),
            CombatError::OutOfRange => "Target is out of weapon range".to_string(),
        }
    }
}

impl std::fmt::Display for CombatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_message())
    }
}

impl From<spacetimedsl::SpacetimeDSLError> for CombatError {
    fn from(_: spacetimedsl::SpacetimeDSLError) -> Self {
        CombatError::InvalidTarget
    }
}

#[dsl(plural_name = visual_effects, method(update = false))]
#[table(accessor = visual_effect, public)]
pub struct VisualEffect {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::logic::combat::visual_effects, table = visual_effect_timer)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::tables::sectors::SectorId)]
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Delete)]
    sector_id: u64,

    source: Vec2,
    target: Vec2,

    effect_type: VisualEffectType,
    created_at: Timestamp,
}
