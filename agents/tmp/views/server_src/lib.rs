use spacetimedb::*;
use spacetimedsl::*;

mod physics;

mod tables;
use tables::*;

mod views;

mod reducers;

mod sectors;

#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    let dsl = spacetimedsl::dsl(ctx);

    // Seed initial ship types
    dsl.create_ship_config(CreateShipConfig {
        id: 1,
        max_health: 100,
        max_speed: 150.0,
        // 80 deg/s in radians
        max_turn_rate: 80.0_f32.to_radians(),
        max_acceleration: 100.0,
        // 180 deg/s² in radians (= π)
        max_angular_acceleration: std::f32::consts::PI,
    });

    dsl.create_sector(CreateSector {
        id: 1,
        system_id: SystemId::new(1),
        is_public: true,
        x: 0,
        y: 0,
    });

    dsl.create_sector(CreateSector {
        id: 2,
        system_id: SystemId::new(1),
        is_public: true,
        x: 5,
        y: 0,
    });

    dsl.create_sector(CreateSector {
        id: 3,
        system_id: SystemId::new(1),
        is_public: false,
        x: 15,
        y: 0,
    });

    dsl.create_system(CreateSystem {
        id: 1,
        name: "Sol".to_string(),
    });

    dsl.create_system(CreateSystem {
        id: 2,
        name: "Alpha Centauri".to_string(),
    });

    dsl.create_system(CreateSystem {
        id: 3,
        name: "Tau Ceti".to_string(),
    });

    dsl.create_sector(CreateSector {
        id: 20,
        system_id: SystemId::new(2),
        is_public: true,
        x: 0,
        y: 0,
    });

    dsl.create_sector(CreateSector {
        id: 30,
        system_id: SystemId::new(3),
        is_public: true,
        x: 0,
        y: 0,
    });
}

#[reducer(client_connected)]
pub fn on_connect(_ctx: &ReducerContext) {
    //
}
