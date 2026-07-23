use spacetimedb::*;
use crate::spacetimedsl::prelude::*;

use crate::tables::*;

pub fn observe_all_public_sectors(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = spacetimedsl::dsl(ctx);
    let player_id = ctx.sender();
    let current_system_id = match dsl.get_player_state_by_id(PlayerStateId::new(player_id)) {
        Ok(player_state) => *player_state.get_current_system_id(),
        Err(_) => return Ok(()),
    };
    let sectors = dsl.get_sectors_by_system_id(SystemId::new(current_system_id));
    for sector in sectors {
        if *sector.get_is_public() {
            let _ = dsl.create_visited_sector(CreateVisitedSector {
                player_id: player_id,
                sector_id: sector.get_id().value(),
                visited_status: VisitedStatus::Observed,
            });
        }
    }
    Ok(())
}
