use std::f32::consts::PI;

use log::info;
use spacetimedsl::*;

use crate::tables::ships::*;

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    fighters(dsl)?;

    info!(
        "Ship Defs Loaded: {}",
        dsl.count_of_all_ship_type_definitions()
    );
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn fighters<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    dsl.create_ship_type_definition(CreateShipTypeDefinition {
        id: 1000,
        name: "Phalanx".to_string(),
        description: Some("The frontline fightercraft for the Lrak Combine.".into()),
        class: ShipClass::Fighter,
        max_health: 100,
        max_shields: 100,
        max_energy: 100,
        base_speed: 50.0,
        base_acceleration: 0.167,
        base_turn_rate: PI / 224.0,
        cargo_capacity: 8,
        num_weapon_slots: 3,
        num_large_weapon_slots: 0,
        num_turret_slots: 0,
        num_large_turret_slots: 0,
        num_shield_slots: 1,
        num_engine_slots: 1,
        num_mining_laser_slots: 0,
        num_special_slots: 1,
        sprite_width: 41,
        sprite_height: // sprite_width
        51,
        gfx_key: // sprite_height
        Some("lc.phalanx".into()),
    })?;
    dsl.create_ship_type_definition(CreateShipTypeDefinition {
        id: 1001,
        name: "Column".to_string(),
        description: Some(
            "A workhorse corvette. This chunky design has been in use for hundreds of years by all factions.".into()
        ),
        class: ShipClass::Shuttle,
        max_health: 500,
        max_shields: 300,
        max_energy: 200,
        base_speed: 45.0,
        base_acceleration: 0.117,
        base_turn_rate: PI / 365.0,
        cargo_capacity: 64,
        num_weapon_slots: 2,
        num_large_weapon_slots: 0,
        num_turret_slots: 0,
        num_large_turret_slots: 0,
        num_shield_slots: 2,
        num_engine_slots: 2,
        num_mining_laser_slots: 1,
        num_special_slots: 3,
        sprite_width: 64,
        sprite_height: // sprite_width
        64,
        gfx_key: // sprite_height
        Some("lc.column".into()),
    })?;
    dsl.create_ship_type_definition(CreateShipTypeDefinition {
        id: 1011,
        name: "Javelin".to_string(),
        description: Some("The frontline fightercraft for the Rediar Federation.".into()),
        class: ShipClass::Fighter,
        max_health: 150,
        max_shields: 50,
        max_energy: 125,
        base_speed: 35.0,
        base_acceleration: 0.167,
        base_turn_rate: PI / 256.0,
        cargo_capacity: 8,
        num_weapon_slots: 2,
        num_large_weapon_slots: 0,
        num_turret_slots: 0,
        num_large_turret_slots: 0,
        num_shield_slots: 1,
        num_engine_slots: 1,
        num_mining_laser_slots: 0,
        num_special_slots: 0,
        sprite_width: 46,
        sprite_height: // sprite_width
        29,
        gfx_key: // sprite_height
        Some("rf.javelin".into()),
    })?;

    Ok(())
}
