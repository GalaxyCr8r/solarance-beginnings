use macroquad::prelude::*;

use crate::{gameplay::render::star_system::render_star_system, module_bindings::*};
use spacetimedb_sdk::{DbContext, Table};

use crate::stdb::utils::*;

use super::{resources::Resources, state::GameState};

pub mod in_sector;
pub mod star_system;

use in_sector::*;

pub fn sector(game_state: &mut GameState) {
    let mut player_transform = None;
    let mut player_ship_type = None;
    let player_ship = get_player_ship(game_state.ctx);

    let mut local_targets: Vec<(u64, glam::Vec2, StellarObjectKinds)> = Vec::new();

    set_camera(&game_state.bg_camera);

    render_star_system(game_state);

    set_camera(&game_state.camera);

    let db = &game_state.ctx.db;

    // Collect ships to draw after stations
    let mut ships_to_draw: Vec<(Ship, StellarObjectTransformHiRes, ShipTypeDefinition)> =
        Vec::new();

    // First pass: Draw everything except ships
    for object in db.stellar_object().iter() {
        // Don't continue if isn't in the same sector.
        // if player_ship.as_ref().is_some_and(|ship_obj| ship_obj.sector_id != object.sector_id) {
        //     continue;
        // }

        // ONLY draw if they have hi-resolution positions.
        if let Some(transform) = db.sobj_hi_res_transform().id().find(&object.id) {
            if let Some(ship_object) = db.ship().sobj_id().find(&transform.id) {
                if let Some(ship_type) = db
                    .ship_type_definition()
                    .id()
                    .find(&ship_object.shiptype_id)
                {
                    if player_ship
                        .as_ref()
                        .is_some_and(|ship_obj| ship_obj.sobj_id == object.id)
                    {
                        // Store player transform for later use
                        player_transform = Some(transform.clone());
                        player_ship_type = Some(ship_type);
                    } else {
                        // Collect non-player ships to draw later
                        ships_to_draw.push((
                            ship_object.clone(),
                            transform.clone(),
                            ship_type.clone(),
                        ));
                    }
                }
            } else if let Some(station) = db.station().sobj_id().find(&object.id) {
                draw_station(&transform, station, game_state);
            } else if let Some(jumpgate) = db.jump_gate().id().find(&object.id) {
                draw_jumpgate(&transform, jumpgate, game_state);
            } else if let Some(asteroid) = db.asteroid().id().find(&object.id) {
                draw_asteroid(&transform, asteroid, game_state);
            } else if let Some(cargo_crate) = db.cargo_crate().sobj_id().find(&object.id) {
                draw_crate(&transform, cargo_crate, game_state);
            } else {
                continue; // Skip unknown object types instead of stopping all rendering
            }
            local_targets.push((object.id, transform.clone().to_vec2(), object.kind));
        } else if let Some(transform) = db.sobj_low_res_transform().id().find(&object.id) {
            // Draw icon even if it has a low-res transform.
            local_targets.push((object.id, transform.clone().to_vec2(), object.kind));
        }
    }

    // Second pass: Draw all non-player ships AFTER stations
    for (ship_object, transform, ship_type) in ships_to_draw {
        draw_ship(&ship_object, &transform, &ship_type, game_state);
    }

    if let Some(controller) = db
        .player_ship_controller()
        .id()
        .find(&game_state.ctx.identity())
    {
        if player_transform.is_none() || player_ship_type.is_none() || player_ship.is_none() {
            return;
        }
        let actual_player_transform = player_transform.unwrap();
        let player_vec = actual_player_transform.to_vec2();

        draw_mining_laser(game_state, &actual_player_transform, &controller);

        // Draw the controlled ship so its always on top.
        draw_ship(
            &player_ship.unwrap(),
            &actual_player_transform,
            &player_ship_type.unwrap(),
            game_state,
        );

        // Draw 'radar'
        draw_radar(
            game_state,
            local_targets,
            &actual_player_transform,
            player_vec,
        );
    }
}
