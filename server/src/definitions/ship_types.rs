use std::f32::consts::PI;

use log::info;
use crate::spacetimedsl::prelude::*;

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
        base_speed: 70.0,
        base_acceleration: 19.0,
        // Hand-tuned: nimble interceptor, ramps angular speed quickly and tops out fast.
        // Cap and accel scaled to 2/3 of the original first cut after live-feel test.
        base_angular_acceleration: 8.0 * PI / 3.0,
        base_max_turn_rate: 4.0 * PI / 3.0,
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
        sprite_height: 51,
        gfx_key: Some("lc.phalanx".into()),
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
        base_speed: 150.0,
        base_acceleration: 70.0,
        // Hand-tuned: chunky shuttle, slower ramp-up, lower top angular speed.
        // Cap and accel scaled to 2/3 of the original first cut after live-feel test.
        base_angular_acceleration: 4.0 * PI / 3.0,
        base_max_turn_rate: 2.0 * PI / 3.0,
        cargo_capacity: 256,
        num_weapon_slots: 2,
        num_large_weapon_slots: 0,
        num_turret_slots: 0,
        num_large_turret_slots: 0,
        num_shield_slots: 2,
        num_engine_slots: 2,
        num_mining_laser_slots: 1,
        num_special_slots: 3,
        sprite_width: 64,
        sprite_height: 64,
        gfx_key: Some("lc.column".into()),
    })?;
    dsl.create_ship_type_definition(CreateShipTypeDefinition {
        id: 1011,
        name: "Javelin".to_string(),
        description: Some("The frontline fightercraft for the Rediar Federation.".into()),
        class: ShipClass::Fighter,
        max_health: 150,
        max_shields: 50,
        max_energy: 125,
        base_speed: 75.0,
        base_acceleration: 18.0,
        // Hand-tuned: agile fighter, similar to Phalanx but slightly stiffer.
        // Cap and accel scaled to 2/3 of the original first cut after live-feel test.
        base_angular_acceleration: 7.0 * PI / 3.0,
        base_max_turn_rate: 7.0 * PI / 6.0,
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
        sprite_height: 29,
        gfx_key: Some("rf.javelin".into()),
    })?;

    Ok(())
}
