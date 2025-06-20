use spacetimedb::*;
use spacetimedsl::*;

use crate::*;

// pub mod definitions; // Definitions for initial ingested data.
// pub mod impls; // Impls for this file's structs
// pub mod reducers; // SpacetimeDB Reducers for this file's structs.
// pub mod rls; // Row-level-security rules for this file's structs.
// pub mod timers; // Timers related to this file's structs.
//pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[dsl(plural_name = npc_behavior_schedules)]
#[table(name = npc_behavior_schedule, scheduled(process_npc_behavior_tick))]
pub struct NpcBehaviorSchedule {
    #[primary_key]
    pub npc_instance_id: u64, // FK to NpcInstance
    pub scheduled_at: ScheduleAt, // Periodic, frequency might depend on NPC complexity/activity
}

#[spacetimedb::reducer]
pub fn process_npc_behavior_tick(_ctx: &ReducerContext, _timer: NpcBehaviorSchedule) -> Result<(), String> {
    Err("Not implemented".to_string())
}