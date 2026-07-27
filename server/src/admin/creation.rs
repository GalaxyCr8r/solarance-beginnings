use std::f32::consts::PI;

use solarance_shared::Vec2;
use spacetimedb::ReducerContext;
use crate::spacetimedsl::prelude::*;

use crate::logic::stellarobjects::stellar_object_creation::create_sobj;
use crate::tables::{
    factions::FactionId, jumpgates::*, sectors::*, star_system::StarSystemId,
    stellarobjects::StellarObjectKinds,
};
use crate::utility::try_server_only;

pub fn create_jumpgate_internal<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    sector_id: u64,
    x: f32,
    y: f32,
    target_sector_id: u64,
    t_x: f32,
    t_y: f32,
) -> Result<(), String> {
    try_server_only(dsl)?;

    let current_sector_id = SectorId::new(sector_id);

    let sobj = create_sobj(dsl, StellarObjectKinds::JumpGate, &current_sector_id)?;
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
    // Local helper: classify a gate position (relative to sector origin)
    // into the heading that points "into the sector" — i.e. away from
    // whatever edge the gate sits on. Reused for both the gate's own
    // visual rotation and the arrival rotation on the destination side
    // (whose coords are `t_x, t_y`).
    let inward_facing_rotation = |px: f32, py: f32| -> f32 {
        if py.abs() < px.abs() {
            // Horizontal gate
            if px < 0.0 {
                0.0 // west gate → face east (+x)
            } else {
                PI // east gate → face west (-x)
            }
        } else {
            // Vertical gate
            if py < 0.0 {
               PI / 2.0 // north gate → face north (-y), screen up
            } else {
               2.0 * PI - (PI * 0.5) // south gate → face south (+y), screen down
            }
        }
    };

    // The arriving ship should face into the destination sector (same
    // convention as the destination gate's own rotation), so feed the
    // arrival coords through the same classifier.
    let arrival_rotation = inward_facing_rotation(t_x, t_y);

    dsl.create_jump_gate(CreateJumpGate {
        id: sobj.get_id(),
        current_sector_id,
        target_sector_id: SectorId::new(target_sector_id),
        target_gate_arrival_pos: Vec2 { x: t_x, y: t_y },
        target_gate_arrival_rotation: arrival_rotation,
        gfx_key: Some(gfx_key),
        is_active: true,
        position: Vec2 { x, y },
        rotation: 0.0,
    })?;

    Ok(())
}

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
    create_jumpgate_internal(&dsl, sector_id, x, y, target_sector_id, t_x, t_y)
}

/// Galaxy Creator (#34): drop a brand-new sector into an existing star system
/// at runtime, mirroring the init-time seed in `definitions/galaxy.rs` so the
/// designer can lay out the MVP sectors without republishing the module.
///
/// Sector IDs are NOT auto-increment (so a sector can be reloaded as-is), so we
/// assign the next free ID ourselves: `max(existing) + 1`, or 0 for the first.
#[spacetimedb::reducer]
pub fn admin_create_sector(
    ctx: &ReducerContext,
    system_id: u32,
    name: String,
    controlling_faction_id: u32,
    security_level: u8,
    sunlight: f32,
    anomalous: f32,
    nebula: f32,
    rare_ore: f32,
    x: f32,
    y: f32,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    if name.trim().is_empty() {
        return Err("admin_create_sector: name must not be empty".to_string());
    }

    let next_id = dsl
        .get_all_sectors()
        .map(|s| s.get_id().value())
        .max()
        .map(|max| max + 1)
        .unwrap_or(0);

    let sector = dsl.create_sector(CreateSector {
        id: next_id,
        system_id: StarSystemId::new(system_id),
        name: name.clone(),
        description: None,
        controlling_faction_id: FactionId::new(controlling_faction_id),
        security_level,
        sunlight,
        anomalous,
        nebula,
        rare_ore,
        x,
        y,
        background_gfx_key: None,
    })?;

    log::info!(
        "admin_create_sector: caller={} sector_id={} name={:?} system_id={} faction={} pos=({:.1},{:.1})",
        ctx.sender().to_abbreviated_hex(),
        sector.get_id().value(),
        name,
        system_id,
        controlling_faction_id,
        x,
        y,
    );
    Ok(())
}

/// Galaxy Creator (#34): link two existing sectors with a bidirectional pair of
/// jump gates. Wraps `connect_sectors_with_warpgates`, which places one gate on
/// each side, orienting each toward the other sector based on their star-system
/// positions.
#[spacetimedb::reducer]
pub fn admin_connect_sectors(
    ctx: &ReducerContext,
    sector_a_id: u64,
    sector_b_id: u64,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    if sector_a_id == sector_b_id {
        return Err(format!(
            "admin_connect_sectors: cannot connect sector {} to itself",
            sector_a_id
        ));
    }

    let a = dsl.get_sector_by_id(&SectorId::new(sector_a_id))?;
    let b = dsl.get_sector_by_id(&SectorId::new(sector_b_id))?;

    connect_sectors_with_warpgates(&dsl, &a, &b)?;

    log::info!(
        "admin_connect_sectors: caller={} connected sector {} <-> {}",
        ctx.sender().to_abbreviated_hex(),
        sector_a_id,
        sector_b_id,
    );
    Ok(())
}
