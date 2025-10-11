use spacetimedb::{table, ReducerContext, ScheduleAt, SpacetimeType, Timestamp};
use spacetimedsl::{dsl, Wrapper};

use crate::types::common::Vec2;
use crate::types::npcs::NpcShipController;
use crate::types::players::PlayerShipController;

pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
pub mod timers; // Timers related to this file's structs.
pub mod utility; // Utility functions (NOT reducers) for this file's structs.

use timers::{cleanup_visual_effect, update_combat_cooldowns};

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

/// Represents either a player or NPC ship controller for combat operations
#[derive(Debug, Clone)]
pub enum ShipController {
    Player(PlayerShipController),
    Npc(NpcShipController),
}

impl ShipController {
    /// Get the stellar object ID for this controller
    pub fn get_stellar_object_id(&self) -> u64 {
        match self {
            ShipController::Player(controller) => controller.get_stellar_object_id().value(),
            ShipController::Npc(controller) => controller.get_stellar_object_id().value(),
        }
    }

    /// Check if weapons should be fired
    pub fn should_fire_weapons(&self) -> bool {
        match self {
            ShipController::Player(controller) => *controller.get_fire_weapons(),
            ShipController::Npc(controller) => *controller.get_fire_weapons(),
        }
    }

    /// Check if missiles should be fired
    pub fn should_fire_missiles(&self) -> bool {
        match self {
            ShipController::Player(controller) => *controller.get_fire_missles(),
            ShipController::Npc(controller) => *controller.get_fire_missiles(),
        }
    }

    /// Get the targeted stellar object ID
    pub fn get_targeted_sobj_id(&self) -> Option<u64> {
        match self {
            ShipController::Player(controller) => *controller.get_targetted_sobj_id(),
            ShipController::Npc(controller) => *controller.get_targetted_sobj_id(),
        }
    }

    /// Reset the fire weapons flag
    pub fn reset_fire_weapons(&mut self) {
        match self {
            ShipController::Player(controller) => {
                controller.set_fire_weapons(false);
            }
            ShipController::Npc(controller) => {
                controller.set_fire_weapons(false);
            }
        }
    }

    /// Reset the fire missiles flag
    pub fn reset_fire_missiles(&mut self) {
        match self {
            ShipController::Player(controller) => {
                controller.set_fire_missles(false);
            }
            ShipController::Npc(controller) => {
                controller.set_fire_missiles(false);
            }
        }
    }
}

#[dsl(plural_name = visual_effects)]
#[table(name = visual_effect, public)]
pub struct VisualEffect {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::types::combat, table = visual_effect_timer)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(path = crate::types::sectors::SectorId)]
    #[foreign_key(path = crate::types::sectors, table = sector, column = id, on_delete = Delete)]
    pub sector_id: u64,

    pub source: Vec2,
    pub target: Vec2,

    pub effect_type: VisualEffectType,
    created_at: Timestamp,
}

#[dsl(plural_name = visual_effect_timers)]
#[spacetimedb::table(name = visual_effect_timer, scheduled(cleanup_visual_effect))]
pub struct VisualEffectTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(path = VisualEffectId)]
    #[foreign_key(path = crate::types::combat, table = visual_effect, column = id, on_delete = Delete)]
    pub effect_id: u64,

    pub scheduled_at: ScheduleAt,
}

#[dsl(plural_name = combat_cooldown_timers)]
#[spacetimedb::table(name = combat_cooldown_timer, scheduled(update_combat_cooldowns))]
pub struct CombatCooldownTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub scheduled_at: ScheduleAt,
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    timers::init(ctx)?;

    Ok(())
}
