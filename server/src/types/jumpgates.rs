use spacetimedb::{ table, ReducerContext };
use spacetimedsl::{ dsl, Wrapper };

use super::{common::Vec2, sectors::SectorId, stellarobjects::{utility::create_sobj_vec2, StellarObjectKinds}};

// pub mod definitions; // Definitions for initial ingested data.
// pub mod impls; // Impls for this file's structs
pub mod reducers; // SpacetimeDB Reducers for this file's structs.
// pub mod rls; // Row-level-security rules for this file's structs.
// pub mod timers; // Timers related to this file's structs.
// pub mod utility; // Utility functions (NOT reducers) for this file's structs.

#[dsl(plural_name = jump_gates)]
#[table(name = jump_gate, public)]
pub struct JumpGate {
    #[primary_key]
    #[wrapped(path = crate::types::stellarobjects::StellarObjectId)]
    /// FK to StellarObject
    pub sobj_id: u64,

    #[wrapped(path = crate::types::sectors::SectorId)]
    #[index(btree)] // To find gates in a specific sector
    /// FK to SectorDefinition where this gate physically is
    pub current_sector_id: u64,

    #[wrapped(path = crate::types::sectors::SectorId)]
    /// FK to SectorDefinition for the destination sector
    pub target_sector_id: u64,
    pub target_gate_arrival_pos: Vec2, // Position where ships appear in the target_sector_id

    pub gfx_key: Option<String>, // For clients

    pub is_active: bool, // Can the gate be used?

    //pub required_item_id: Option<u32>, // Optional: item needed to use the gate (e.g., jump drive, key)
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {
    Ok(())
}

//////////////////////////////////////////////////////////////
// Reducers
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn create_jumpgate_in_sector(ctx: &ReducerContext, sector_id: u64, x: f32, y: f32, target_sector_id: u64, t_x: f32, t_y: f32) -> Result<(), String> {
    let dsl = dsl(ctx);

    let current_sector_id = SectorId::new(sector_id);

    let sobj = create_sobj_vec2(ctx, StellarObjectKinds::JumpGate, &current_sector_id, glam::Vec2::new(x, y))?;
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
    dsl.create_jump_gate(&sobj,
        current_sector_id,
        &SectorId::new(target_sector_id),
        Vec2 { x: t_x, y: t_y },
        Some(gfx_key),
        true)?;

    Ok(())
}
