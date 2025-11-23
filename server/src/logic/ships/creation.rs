use log::info;
use spacetimedb::{Identity, ReducerContext};
use spacetimedsl::{dsl, Wrapper};

use crate::{
    definitions::item_types::*,
    logic::ships::{
        cargo::attempt_to_load_cargo_into_ship, player_controller::initialize_player_controller,
    },
    tables::{
        chats::send_global_chat,
        factions::FactionId,
        items::{utility::*, *},
        players::{GetPlayerRowOptionById, PlayerId},
        sectors::SectorId,
        server_messages::*,
        ships::{timers::*, *},
        stations::*,
        stellarobjects::{reducers::create_sobj_player_window_for, utility::*, *},
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
                ctx,
                &player_id,
                error_message.clone(),
                Some("Ship Creation"),
            )?;

            return Err("Client hasn't created a username yet!".to_string());
        }
    };

    if let Ok(sobj) = create_sobj_internal(
        ctx,
        StellarObjectKinds::Ship,
        &SectorId::new(0), // TODO: Make this the proper sector id!
        StellarObjectTransformInternal::default().from_xy(64.0, 64.0),
    ) {
        let _ = create_sobj_player_window_for(ctx, identity, sobj.get_id())?;
        initialize_player_controller(ctx, &player_id, &sobj)?;

        let ship_type = dsl.get_ship_type_definition_by_id(ShipTypeDefinitionId::new(1001))?;
        let faction_id = player.get_faction_id().clone();
        let (ship, mut status) =
            create_ship_from_sobj(ctx, &ship_type, &player_id, &faction_id, &sobj)?;

        let _ = attempt_to_load_cargo_into_ship(
            ctx,
            &mut status,
            &ship.get_id(),
            &get_item_definition(ctx, ITEM_FOOD_RATIONS)?,
            3,
            false,
        )?;
        let _ = attempt_to_load_cargo_into_ship(
            ctx,
            &mut status,
            &ship.get_id(),
            &get_item_definition(ctx, ITEM_ENERGY_CELL)?,
            5,
            false,
        )?;

        dsl.create_ship_equipment_slot(
            &ship.get_id(),
            EquipmentSlotType::MiningLaser,
            0,
            ItemDefinitionId::new(SMOD_BASIC_MINING_LASER),
        )?;

        dsl.create_ship_equipment_slot(
            &ship.get_id(),
            EquipmentSlotType::Weapon,
            0,
            ItemDefinitionId::new(SMOD_IONIC_BLASTER),
        )?;

        info!("Successfully created ship!");
        send_global_chat(ctx, format!("{} has created a ship!", username))?;
        Ok(())
    } else {
        let error_message =
            "Failed to create ship due to a system error. Please try again or contact support if the problem persists.".to_string();

        // Send server message for error feedback
        send_error_message(
            ctx,
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
        *ship_type.get_max_health() as f32,
        *ship_type.get_max_shields() as f32,
        *ship_type.get_max_energy() as f32,
        0, // weapon_cooldown_ms
        0, // missile_cooldown_ms
        0, // used_cargo_capacity
        *ship_type.get_cargo_capacity(),
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
        *ship_type.get_max_health() as f32,
        *ship_type.get_max_shields() as f32,
        *ship_type.get_max_energy() as f32,
        0, // weapon_cooldown_ms
        0, // missile_cooldown_ms
        0, // used_cargo_capacity
        *ship_type.get_cargo_capacity(),
        None,
    )?;

    return Ok((ship, ship_status));
}
