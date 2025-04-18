use std::time::Duration;
use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table};

#[spacetimedb::table(name = update_sobj_transform_timer, scheduled(update_sobj_transforms))]
pub struct UpdatePositionTimer {
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

#[spacetimedb::reducer]
pub fn update_sobj_transforms(ctx: &ReducerContext, _arg: UpdatePositionTimer) {
    // TODO
}