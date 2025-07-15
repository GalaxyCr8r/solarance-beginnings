use super::*;

#[dsl(plural_name = npc_behavior_schedules)]
#[table(name = npc_behavior_schedule, scheduled(process_npc_behavior_tick))]
pub struct NpcBehaviorSchedule {
    #[primary_key]
    #[create_wrapper]
    /// FK to NpcInstance
    id: u64,
    pub scheduled_at: ScheduleAt, // Periodic, frequency might depend on NPC complexity/activity
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// Reducers

#[spacetimedb::reducer]
pub fn process_npc_behavior_tick(
    _ctx: &ReducerContext,
    _timer: NpcBehaviorSchedule
) -> Result<(), String> {
    Err("Not implemented".to_string())
}
