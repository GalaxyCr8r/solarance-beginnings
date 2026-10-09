use macroquad::prelude::*;
use spacetimedb_sdk::{DbContext, Table};

use crate::server::bindings::*;

use crate::stdb::utils::*;

use super::state::GameState;

pub fn control_player_ship(ctx: &DbConnection, game_state: &mut GameState) -> Result<(), String> {
    if game_state.chat_window.has_focus || ctx.try_identity().is_none() {
        return Ok(());
    }

    let forward  = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
    let backward = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);
    let left     = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
    let right    = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);

    let new_flags = (forward, backward, left, right);
    if game_state.movement_flags != new_flags {
        game_state.movement_flags = new_flags;
        let _ = ctx.reducers.update_ship_movement_controller(forward, backward, left, right);
    }

    Ok(())
}

pub fn target_closest_stellar_object(
    ctx: &DbConnection,
    game_state: &mut GameState,
) -> Result<StellarObject, String> {
    if game_state.chat_window.has_focus {
        return Err("Chat window has focus. Cannot target objects.".to_string());
    }

    //let player_id = ctx.identity();
    let player_ship_id = get_player_ship(ctx)
        .ok_or("Player doesn't control a stellar object yet!")?
        .sobj_id;
    let player_sobj = ctx
        .db
        .stellar_object()
        .id()
        .find(&player_ship_id)
        .ok_or("Player doesn't control a stellar object yet!")?;
    let player_transform = get_transform(ctx, player_ship_id)?.to_vec2();

    let mut closest_distance = f32::MAX;
    let mut closest_sobj = Option::None;

    for sobj in ctx.db().stellar_object().iter() {
        if sobj.id == player_ship_id || sobj.sector_id != player_sobj.sector_id {
            continue; // Skip the player's ship and non-sector objects
        }
        if let Ok(transform) = get_transform(ctx, sobj.id) {
            let distance = transform.to_vec2().distance_squared(player_transform);
            if distance < closest_distance {
                closest_distance = distance;
                closest_sobj = Some(sobj);
            }
        }
    }

    if let Some(sobj) = closest_sobj {
        match sobj.kind {
            // None => {
            //     info!("Could not find type for stellar object: {}", sobj.id);
            //     Err("Could not find type for targeted stellar object.".to_string())
            // },
            _ => {
                info!("Targeted closest {:?}: {}", sobj.kind, sobj.id);
                Ok(sobj)
            }
        }
    } else {
        info!("No stellar objects found to target.");
        Err("Could not find a stellar object to target.".to_string())
    }
}

/// Start mining the current target, or stop an in-progress beam.
///
/// The `[X]` button and the `[X]` hotkey both land here (#218) so the two can't
/// disagree about when mining is possible. Mining state is read back from the
/// server's beam row, never a local flag (#141).
pub fn toggle_mining_beam(ctx: &DbConnection, game_state: &mut GameState) -> Result<(), String> {
    if is_player_mining(ctx) {
        ctx.reducers
            .stop_mining_asteroid()
            .map_err(|e| format!("Failed to stop mining: {e}"))
    } else {
        let target = get_current_target(ctx, &mut game_state.current_target_sobj_id)
            .ok_or("No target selected to mine.")?;
        if target.kind != StellarObjectKinds::Asteroid {
            return Err(format!(
                "Target is a {:?}, not an asteroid — nothing to mine.",
                target.kind
            ));
        }
        ctx.reducers
            .try_mining_asteroid(StellarObjectId { value: target.id })
            .map_err(|e| format!("Failed to start mining asteroid {}: {e}", target.id))
    }
}

/// What the `[C]` key does right now, given where the ship is and what it has
/// targeted. `None` means the key is inert — nothing to dock with or jump to.
///
/// Both the button label and the keypress read this, so the label can never
/// promise an action the key won't take (#218).
pub fn docking_action(ctx: &DbConnection, game_state: &mut GameState) -> Option<DockingAction> {
    let identity = ctx.try_identity()?;
    let ship = ctx.db().ship().iter().find(|s| s.player_id == identity)?;

    match ship.location {
        ShipLocation::Station => Some(DockingAction::Undock(ship)),
        ShipLocation::Sector => {
            let target = get_current_target(ctx, &mut game_state.current_target_sobj_id)?;
            match target.kind {
                StellarObjectKinds::Station => Some(DockingAction::Dock(target.id)),
                StellarObjectKinds::JumpGate => Some(DockingAction::Jump(target.id)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Where the `[C]` key sends the player. Server-side distance / energy gating
/// still applies — this only routes the intent.
pub enum DockingAction {
    Dock(u64),
    Jump(u64),
    Undock(Ship),
}

impl DockingAction {
    /// Verb for the button face, so `[C]`'s label follows the binding.
    pub fn verb(&self) -> &'static str {
        match self {
            DockingAction::Dock(_) => "Dock",
            DockingAction::Jump(_) => "Jump",
            DockingAction::Undock(_) => "Undock",
        }
    }

    pub fn perform(self, ctx: &DbConnection) -> Result<(), String> {
        match self {
            DockingAction::Dock(sobj_id) => ctx
                .reducers
                .dock_ship(sobj_id)
                .map_err(|e| format!("Failed to dock at station sobj {sobj_id}: {e}")),
            DockingAction::Jump(sobj_id) => ctx
                .reducers
                .use_jumpgate(sobj_id)
                .map_err(|e| format!("Failed to use jumpgate sobj {sobj_id}: {e}")),
            DockingAction::Undock(ship) => {
                let ship_id = ship.id;
                ctx.reducers
                    .undock_ship(ship)
                    .map_err(|e| format!("Failed to undock ship {ship_id}: {e}"))
            }
        }
    }
}
