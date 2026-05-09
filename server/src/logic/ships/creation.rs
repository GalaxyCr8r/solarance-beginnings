use log::info;
use spacetimedb::{Identity, ReducerContext};
use spacetimedsl::*;

use crate::{
    definitions::item_types::*,
    logic::{
        chat_messages::send_global_chat,
        ships::{cargo::*, movement_controllers::initialize_controller_for_player, status::*},
        stellarobjects::{player_windows::*, stellar_object_creation::*},
    },
    tables::{
        factions::FactionId,
        items::*,
        players::*,
        sectors::SectorId,
        server_messages::*,
        ships::{CreateShipEquipmentSlot, *},
        stations::*,
        stellarobjects::*,
    },
};

///////////////////////////////////
///  Reducers

/// Creates a new ship for a registered player with starting equipment and cargo.
/// Sets up the ship's stellar object, player window, controller, and initial inventory.
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

            // Send server message for error feedback
            send_error_message(
                &dsl,
                &player_id,
                error_message.clone(),
                Some("Ship Creation"),
            )?;

            return Err("Client hasn't created a username yet!".to_string());
        }
    };

    if let Ok(sobj) = create_sobj_internal(
        &dsl,
        StellarObjectKinds::Ship,
        &SectorId::new(0), // TODO: Make this the proper sector id! Needs to be picked based on the joinable faction's home station.
        StellarObjectTransformInternal::default().from_xy(64.0, 64.0),
    ) {
        let _ = create_sobj_player_window_for(dsl.ctx(), identity, sobj.get_id())?;
        initialize_controller_for_player(&dsl, &player_id, &sobj)?;

        let ship_type = dsl.get_ship_type_definition_by_id(ShipTypeDefinitionId::new(1001))?;
        let faction_id = player.get_faction_id().clone();
        let (ship, mut status) =
            create_ship_from_sobj(&dsl, &ship_type, &player_id, &faction_id, &sobj)?;

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
        send_global_chat(dsl.ctx(), format!("{} has created a ship!", username))?;
        Ok(())
    } else {
        let error_message =
            "Failed to create ship due to a system error. Please try again or contact support if the problem persists.".to_string();

        // Send server message for error feedback
        send_error_message(
            &dsl,
            &player_id,
            error_message.clone(),
            Some("Ship Creation"),
        )?;

        Err("Failed to create ship!".to_string())
    }
}

////////////////////////////////////////////
/// Utility

/// Creates a brand new ship instance in a sector with a specific stellar object.
pub fn create_ship_from_sobj<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship_type: &ShipTypeDefinition,
    player_id: &PlayerId,
    faction_id: &FactionId,
    sobj: &StellarObject,
) -> Result<(Ship, ShipStatus), String> {
    let ship = dsl.create_ship(CreateShip {
        shiptype_id: ship_type.get_id(),
        location: ShipLocation::Sector,
        sobj_id: sobj.get_id(),
        station_id: StationId::new(0), // Sentinel for None
        sector_id: sobj.get_sector_id(),
        player_id: player_id.clone(),
        faction_id: faction_id.clone(),
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

    return Ok((ship, ship_status));
}

/// Creates a brand new ship instance docked at a station.
pub fn create_ship_docked_at_station<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship_type: ShipTypeDefinition,
    player_id: &PlayerId,
    faction_id: &FactionId,
    station: Station,
) -> Result<(Ship, ShipStatus), String> {
    let ship = dsl.create_ship(CreateShip {
        shiptype_id: ship_type.get_id(),
        location: ShipLocation::Station,
        sobj_id: station.get_sobj_id(),
        station_id: station.get_id(),
        sector_id: station.get_sector_id(),
        player_id: player_id.clone(),
        faction_id: faction_id.clone(),
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

    return Ok((ship, ship_status));
}
