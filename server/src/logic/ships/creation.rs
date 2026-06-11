use log::info;
use solarance_shared::Vec2;
use spacetimedb::{Identity, ReducerContext};
use spacetimedsl::*;

use crate::{
    definitions::item_types::*,
    logic::{
        chat_messages::send_galaxy_chat,
        ships::{cargo::*, movement_controllers::initialize_controller_for_player, status::*},
        stellarobjects::stellar_object_creation::create_sobj,
    },
    tables::{
        factions::*,
        items::*,
        players::*,
        sectors::SectorId,
        messages::*,
        ships::{CreateShipEquipmentSlot, *},
        stations::*,
        stellarobjects::*,
    },
};

///////////////////////////////////
///  Reducers

/// Fallback spawn pose for players whose faction has no Capital station
/// (e.g. Factionless). Factions *with* a Capital spawn beside it instead —
/// see [`capital_spawn_for_faction`].
const STARTING_SPAWN_POS: Vec2 = Vec2 { x: 64.0, y: 64.0 };
const STARTING_SPAWN_ROTATION: f32 = 0.0;
const FALLBACK_SPAWN_SECTOR: u64 = 0;

/// Offset from the Capital station's position so new ships don't spawn inside
/// the station sprite.
const CAPITAL_SPAWN_OFFSET: Vec2 = Vec2 { x: 400.0, y: 250.0 };

/// Resolve where a new player's ship should spawn: their faction's Capital
/// station's sector, right beside the station (#105). Falls back to the
/// default sector/pose when the faction has no Capital (Factionless) or the
/// Capital row is missing — a wrong-but-playable spawn beats a failed one.
fn capital_spawn_for_faction(
    dsl: &DSL<'_, ReducerContext>,
    faction_id: &FactionId,
) -> (SectorId, Vec2) {
    let fallback = (SectorId::new(FALLBACK_SPAWN_SECTOR), STARTING_SPAWN_POS);

    let faction = match dsl.get_faction_by_id(faction_id) {
        Ok(f) => f,
        Err(_) => return fallback,
    };
    let station_id = match faction.get_capital_station_id() {
        Some(id) => StationId::new(*id),
        None => return fallback,
    };
    match dsl.get_station_by_id(&station_id) {
        Ok(station) => (
            SectorId::new(station.get_sector_id().value()),
            Vec2 {
                x: station.get_position().x + CAPITAL_SPAWN_OFFSET.x,
                y: station.get_position().y + CAPITAL_SPAWN_OFFSET.y,
            },
        ),
        Err(_) => {
            log::warn!(
                "Faction {} names capital station {} but the station row is missing; spawning at fallback sector {}",
                faction_id.value(),
                station_id.value(),
                FALLBACK_SPAWN_SECTOR
            );
            fallback
        }
    }
}

/// Creates a new ship for a registered player with starting equipment and cargo.
/// Sets up the ship's stellar object, controller, and initial inventory.
#[spacetimedb::reducer]
pub fn create_player_controlled_ship(
    ctx: &ReducerContext,
    identity: Identity,
    username: String, // TODO ReMOVE
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let player_id = PlayerId::new(identity);
    let player = match dsl.get_player_by_id(&player_id) {
        Ok(p) => p,
        Err(_) => {
            let error_message =
                "You must register a username before creating a ship. Please use the registration system first.".to_string();

            send_direct_server_warning(
                &dsl,
                &player_id,
                error_message.clone(),
            )?;

            return Err("Client hasn't created a username yet!".to_string());
        }
    };

    // One ship per player (#103). The client only offers ship creation when
    // no ship exists, but the reducer must hold the line on its own.
    if dsl.get_ships_by_player_id(&player_id).next().is_some() {
        return Err(format!(
            "Player {} already owns a ship — one Column per player in MVP (#103); rejecting create_player_controlled_ship",
            username
        ));
    }

    let faction_id = player.get_faction_id().clone();
    let (spawn_sector, spawn_pos) = capital_spawn_for_faction(&dsl, &faction_id);

    if let Ok(sobj) = create_sobj(&dsl, StellarObjectKinds::Ship, &spawn_sector) {
        initialize_controller_for_player(&dsl, &player_id, &sobj)?;

        let ship_type = dsl.get_ship_type_definition_by_id(ShipTypeDefinitionId::new(1001))?;
        let (ship, mut status) = create_ship_from_sobj(
            &dsl,
            &ship_type,
            &player_id,
            &faction_id,
            &sobj,
            spawn_pos,
            STARTING_SPAWN_ROTATION,
        )?;

        attempt_to_load_cargo_into_ship(
            dsl.ctx(),
            &dsl,
            &mut status,
            &ship.get_id(),
            &dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_FOOD_RATIONS))?,
            3,
            false,
        )?;
        attempt_to_load_cargo_into_ship(
            dsl.ctx(),
            &dsl,
            &mut status,
            &ship.get_id(),
            &dsl.get_item_definition_by_id(ItemDefinitionId::new(ITEM_ENERGY_CELL))?,
            5,
            false,
        )?;

        dsl.create_ship_equipment_slot(CreateShipEquipmentSlot {
            ship_id: ship.get_id(),
            slot_type: EquipmentSlotType::MiningLaser,
            slot_index: 0,
            item_id: ItemDefinitionId::new(SMOD_BASIC_MINING_LASER),
        })?;

        dsl.create_ship_equipment_slot(CreateShipEquipmentSlot {
            ship_id: ship.get_id(),
            slot_type: EquipmentSlotType::Weapon,
            slot_index: 0,
            item_id: ItemDefinitionId::new(SMOD_IONIC_BLASTER),
        })?;

        info!("Successfully created ship!");
        send_galaxy_chat(dsl.ctx(), format!("{} has created a ship!", username))?;
        Ok(())
    } else {
        let error_message =
            "Failed to create ship due to a system error. Please try again or contact support if the problem persists.".to_string();

        send_direct_server_warning(
            &dsl,
            &player_id,
            error_message.clone(),
        )?;

        Err("Failed to create ship!".to_string())
    }
}

////////////////////////////////////////////
/// Utility

/// Creates a brand new ship instance in a sector at the given spawn pose.
/// `spawn_pos` / `spawn_rotation` populate `Ship.movement` directly — the
/// legacy `sobj_internal_transform` table no longer exists.
pub fn create_ship_from_sobj<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship_type: &ShipTypeDefinition,
    player_id: &PlayerId,
    faction_id: &FactionId,
    sobj: &StellarObject,
    spawn_pos: Vec2,
    spawn_rotation: f32,
) -> Result<(Ship, ShipStatus), String> {
    let movement = solarance_shared::MovementState {
        pos: spawn_pos,
        rotation: spawn_rotation,
        max_speed: *ship_type.get_base_speed(),
        max_turn_rate: *ship_type.get_base_max_turn_rate(),
        last_update_time: dsl.ctx().timestamp()?.to_micros_since_unix_epoch(),
        ..Default::default()
    };

    let ship = dsl.create_ship(CreateShip {
        shiptype_id: ship_type.get_id(),
        location: ShipLocation::Sector,
        sobj_id: sobj.get_id(),
        station_id: StationId::new(0), // Sentinel for None
        sector_id: sobj.get_sector_id(),
        player_id: player_id.clone(),
        faction_id: faction_id.clone(),
        movement,
    })?;

    create_status_timer_for_ship(dsl, &ship.get_id(), &ship_type.get_id())?;

    let ship_status = dsl.create_ship_status(CreateShipStatus {
        id: ship.get_id(),
        sector_id: sobj.get_sector_id(),
        player_id: player_id.clone(),
        health: *ship_type.get_max_health() as f32,
        shields: *ship_type.get_max_shields() as f32,
        energy: *ship_type.get_max_energy() as f32,
        weapon_cooldown_ms: 0,
        missile_cooldown_ms: 0,
        used_cargo_capacity: 0,
        max_cargo_capacity: *ship_type.get_cargo_capacity(),
    })?;

    Ok((ship, ship_status))
}

/// Creates a brand new ship instance docked at a station.
pub fn create_ship_docked_at_station<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship_type: ShipTypeDefinition,
    player_id: &PlayerId,
    faction_id: &FactionId,
    station: Station,
) -> Result<(Ship, ShipStatus), String> {
    // Docked ships are immovable — movement defaults to all zeros, and
    // last_update_time == 0 makes `predict_movement` a no-op.
    let ship = dsl.create_ship(CreateShip {
        shiptype_id: ship_type.get_id(),
        location: ShipLocation::Station,
        sobj_id: station.get_sobj_id(),
        station_id: station.get_id(),
        sector_id: station.get_sector_id(),
        player_id: player_id.clone(),
        faction_id: faction_id.clone(),
        movement: solarance_shared::MovementState::default(),
    })?;

    create_status_timer_for_ship(dsl, &ship.get_id(), &ship_type.get_id())?;

    let ship_status = dsl.create_ship_status(CreateShipStatus {
        id: ship.get_id(),
        sector_id: station.get_sector_id(),
        player_id: player_id.clone(),
        health: *ship_type.get_max_health() as f32,
        shields: *ship_type.get_max_shields() as f32,
        energy: *ship_type.get_max_energy() as f32,
        weapon_cooldown_ms: 0,
        missile_cooldown_ms: 0,
        used_cargo_capacity: 0,
        max_cargo_capacity: *ship_type.get_cargo_capacity(),
    })?;

    Ok((ship, ship_status))
}
