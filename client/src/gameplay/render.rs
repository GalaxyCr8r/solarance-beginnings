use macroquad::prelude::*;

use crate::{gameplay::render::star_system::render_star_system, server::bindings::*};
use spacetimedb_sdk::{DbContext, Table};

use crate::stdb::utils::*;

use super::{resources::Resources, state::GameState};

pub mod in_sector;
pub mod star_system;

use in_sector::*;

pub fn sector(game_state: &mut GameState) {
    let mut player_pose: Option<RenderPose> = None;
    let mut player_ship_type = None;
    let player_ship = get_player_ship(game_state.ctx);

    let mut local_targets: Vec<(u64, glam::Vec2, StellarObjectKinds)> = Vec::new();

    set_camera(&game_state.bg_camera);

    render_star_system(game_state);

    set_camera(&game_state.camera);

    let db = &game_state.ctx.db;
    let now_micros = now_unix_micros();

    // (#89) The player's current sector is wherever their in-sector ship is.
    // Used below to drop any stellar object whose `sector_id` doesn't match —
    // see the wrong-sector filter in the first pass. `None` while docked /
    // out-of-play, in which case we don't filter (nothing to anchor to).
    let player_sector = player_ship.as_ref().map(|s| s.sector_id);

    // Collect ships to draw after stations
    let mut ships_to_draw: Vec<(Ship, RenderPose, ShipTypeDefinition)> = Vec::new();

    // First pass: Draw everything except ships
    for object in db.stellar_object().iter() {
        // (#89) Wrong-sector render filter (approach (b)). `try_to_use_jumpgate`
        // flips Ship/ShipStatus/StellarObject.sector_id + the movement snapshot
        // in one server transaction, but the rows can briefly land out of order
        // on the client — and pre-#84 the cache holds other sectors' objects
        // outright. Skipping any object not in the player's sector kills the
        // cross-sector ghost/flicker for every kind, and during the transient
        // it hides the player's own ship for at most a frame rather than drawing
        // it at a stale position. Cosmetic only: server state is consistent.
        // ponytail: StellarObject.sector_id is atomic with the row's pose, so
        // this subsumes approach (a) — there is no client-side prediction
        // buffer to separately invalidate.
        if let Some(ps) = player_sector {
            if object.sector_id != ps {
                continue;
            }
        }
        // Build a render pose for this object — predicts forward for ships
        // and cargo crates, reads the static position column for asteroids /
        // stations / jumpgates.
        let pose = match pose_for_object(game_state.ctx, &object, now_micros) {
            Some(p) => p,
            None => continue,
        };

        match object.kind {
            StellarObjectKinds::Ship => {
                if let Some((ship_object, ship_type)) =
                    get_ship_with_type(game_state.ctx, pose.sobj_id)
                {
                    if player_ship
                        .as_ref()
                        .is_some_and(|ship_obj| ship_obj.sobj_id == object.id)
                    {
                        player_pose = Some(pose);
                        player_ship_type = Some(ship_type);
                    } else {
                        ships_to_draw.push((ship_object, pose, ship_type));
                    }
                }
            }
            StellarObjectKinds::Station => {
                if let Some(station) = db.station().sobj_id().find(&object.id) {
                    draw_station(&pose, station, game_state);
                }
            }
            StellarObjectKinds::JumpGate => {
                if let Some(jumpgate) = db.jump_gate().id().find(&object.id) {
                    draw_jumpgate(&pose, jumpgate, game_state);
                }
            }
            StellarObjectKinds::Asteroid => {
                if let Some(asteroid) = db.asteroid().id().find(&object.id) {
                    draw_asteroid(&pose, asteroid, game_state);
                }
            }
            StellarObjectKinds::CargoCrate => {
                if let Some(cargo_crate) = db.cargo_crate().sobj_id().find(&object.id) {
                    draw_crate(&pose, cargo_crate, game_state);
                }
            }
        }
        local_targets.push((object.id, pose.pos, object.kind));
    }

    // Second pass: Draw all non-player ships AFTER stations
    for (ship_object, pose, ship_type) in ships_to_draw {
        draw_ship(&ship_object, &pose, &ship_type, game_state);
    }

    if let (Some(actual_player_pose), Some(player_ship_type), Some(player_ship)) =
        (player_pose, player_ship_type, player_ship)
    {
        draw_mining_laser(game_state, &actual_player_pose);

        // Draw the controlled ship so its always on top.
        draw_ship(&player_ship, &actual_player_pose, &player_ship_type, game_state);

        // Draw 'radar'
        draw_radar(
            game_state,
            local_targets,
            &actual_player_pose,
            actual_player_pose.pos,
        );
    }
}
