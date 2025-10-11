use spacetimedb::{table, ReducerContext, ScheduleAt, SpacetimeType, Timestamp};
use spacetimedsl::dsl;

use crate::types::common::Vec2;

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

#[dsl(plural_name = visual_effects)]
#[table(name = visual_effect, public)]
pub struct VisualEffect {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::types::combat, table = visual_effect_timer)]
    id: u64,

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
