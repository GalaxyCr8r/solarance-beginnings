use std::time::Duration;

use glam::Vec2;
use log::info;
use spacetimedb::{rand::Rng, TimeDuration};

use crate::{
    logic::ships::player_controller::initialize_player_controller,
    tables::{
        factions::FactionId,
        players::PlayerId,
        server_messages::utility::send_info_message,
        ships::timers::*,
        stellarobjects::{reducers::create_sobj_player_window_for, utility::*, *},
    },
};

/// Creates a brand new ship instance in a sector with a specific stellar object.
pub fn create_ship_from_sobj(
    ctx: &ReducerContext,
    ship_type: &ShipTypeDefinition,
    player_id: &PlayerId,
    faction_id: &FactionId,
    sobj: &StellarObject,
) -> Result<(Ship, ShipStatus), String> {
    let dsl = dsl(ctx);

    let ship = (match dsl.create_ship(
        ship_type.get_id(),
        ShipLocation::Sector,
        sobj,
        StationId::new(0),
        sobj.get_sector_id(),
        player_id,
        faction_id,
    ) {
        Ok(ship) => {
            create_status_timer_for_ship(ctx, &ship.get_id(), &ship_type.get_id())?;
            Ok(ship)
        }
        Err(e) => Err(e.to_string()),
    })?;

    let ship_status = dsl.create_ship_status(
        &ship,
        sobj.get_sector_id(),
        player_id,
        ship_type.max_health as f32,
        ship_type.max_shields as f32,
        ship_type.max_energy as f32,
        0, // weapon_cooldown_ms
        0, // missile_cooldown_ms
        0, // used_cargo_capacity
        ship_type.cargo_capacity,
        None,
    )?;

    return Ok((ship, ship_status));
}

/// Creates a brand new ship instance docked at a station.
pub fn create_ship_docked_at_station(
    ctx: &ReducerContext,
    ship_type: ShipTypeDefinition,
    player_id: &PlayerId,
    faction_id: &FactionId,
    station: Station,
) -> Result<(Ship, ShipStatus), String> {
    let dsl = dsl(ctx);

    let ship = (match dsl.create_ship(
        ship_type.get_id(),
        ShipLocation::Station,
        station.get_sobj_id(),
        &station,
        station.get_sector_id(),
        player_id,
        faction_id,
    ) {
        Ok(ship) => {
            create_status_timer_for_ship(ctx, &ship.get_id(), &ship_type.get_id())?;
            Ok(ship)
        }
        Err(e) => Err(e.to_string()),
    })?;

    let ship_status = dsl.create_ship_status(
        &ship,
        station.get_sector_id(),
        player_id,
        ship_type.max_health as f32,
        ship_type.max_shields as f32,
        ship_type.max_energy as f32,
        0, // weapon_cooldown_ms
        0, // missile_cooldown_ms
        0, // used_cargo_capacity
        ship_type.cargo_capacity,
        None,
    )?;

    return Ok((ship, ship_status));
}
