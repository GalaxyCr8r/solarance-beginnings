use spacetimedb::ReducerContext;
use spacetimedsl::*;

use crate::logic::stellarobjects::stellar_object_creation::create_sobj_vec2;
use crate::tables::{
    common_types::Vec2, jumpgates::CreateJumpGate, sectors::SectorId,
    stellarobjects::StellarObjectKinds,
};
use crate::utility::try_server_only;

/// Creates a jump gate in a sector that connects to another sector.
/// Automatically determines gate orientation (north/south/east/west) based on position.
#[spacetimedb::reducer]
pub fn create_jumpgate_in_sector(
    ctx: &ReducerContext,
    sector_id: u64,
    x: f32,
    y: f32,
    target_sector_id: u64,
    t_x: f32,
    t_y: f32,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    try_server_only(&dsl)?;

    let current_sector_id = SectorId::new(sector_id);

    let sobj = create_sobj_vec2(
        &dsl,
        StellarObjectKinds::JumpGate,
        &current_sector_id,
        glam::Vec2::new(x, y),
    )?;
    let gfx_key = {
        if y.abs() < x.abs() {
            // Horizontal gate
            if x < 0.0 {
                "warpgate_west".to_string()
            } else {
                "warpgate_east".to_string()
            }
        } else {
            // Vertical gate
            if y < 0.0 {
                "warpgate_north".to_string()
            } else {
                "warpgate_south".to_string()
            }
        }
    };
    dsl.create_jump_gate(CreateJumpGate {
        id: sobj.get_id().value(),
        current_sector_id: current_sector_id.value(),
        target_sector_id,
        target_gate_arrival_pos: Vec2 { x: t_x, y: t_y },
        gfx_key: Some(gfx_key),
        is_active: true,
    })?;

    Ok(())
}
