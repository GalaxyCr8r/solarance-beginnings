//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

use spacetimedb::ReducerContext;
use spacetimedsl::*;

use crate::types::{
    common::Vec2,
    jumpgates::CreateJumpGateRow,
    sectors::SectorId,
    stellarobjects::{utility::create_sobj_vec2, StellarObjectKinds},
};

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

    let current_sector_id = SectorId::new(sector_id);

    let sobj = create_sobj_vec2(
        ctx,
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
    dsl.create_jump_gate(
        &sobj,
        current_sector_id,
        &SectorId::new(target_sector_id),
        Vec2 { x: t_x, y: t_y },
        Some(gfx_key),
        true,
    )?;

    Ok(())
}
