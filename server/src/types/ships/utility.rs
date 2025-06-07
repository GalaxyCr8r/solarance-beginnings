
use log::info;

use crate::types::{items::ItemDefinition, stellarobjects::{GetStellarObjectRowOptionById, StellarObject}};

use super::{*};

pub fn same_sector_from_ids(ctx: &ReducerContext, id1: &StellarObjectId, id2: &StellarObjectId) -> bool {
    let dsl = dsl(ctx);

    if let Some(sobj1) = dsl.get_stellar_object_by_id(id1) {
        if let Some(sobj2) = dsl.get_stellar_object_by_id(id2) {
            return sobj1.get_sector_id() == sobj2.get_sector_id();
        }
    }
    false
}

pub fn create_ship_instance(ctx: &ReducerContext, ship_type: ShipTypeDefinition, identity: Identity, sobj: StellarObject) -> Result<ShipInstance, String> {
    let dsl = dsl(ctx);

    match dsl.create_ship_instance(
            ship_type.get_id(),
            Some(identity), None, 
            Some(sobj.get_id()), 
            sobj.get_sector_id(), 
            ship_type.max_health.into(),
            ship_type.max_shield.into(),
            ship_type.max_energy.into(),
            ship_type.cargo_capacity, 
            None, None,
            ctx.timestamp) {
                Ok(ship) => Ok(ship),
                Err(e) => Err(e.to_string())
            }
}

pub fn load_cargo_into_ship(ctx: &ReducerContext, ship: &mut ShipInstance, item: &ItemDefinition, amount: u16) -> Result<(), String> {
    let dsl = dsl(ctx);

    info!("Attempting to load {}x {} into ship #{} with remaining cargo space of {}u", amount, item.name, ship.id, ship.get_cargo_capacity());
    if *ship.get_cargo_capacity() >= *item.get_volume_per_unit() * amount {
        let _ = dsl.create_ship_cargo_item(ship.get_id(), item, amount.into())?;
        ship.cargo_capacity -= item.get_volume_per_unit() * amount;
        let _ = dsl.update_ship_instance_by_id(ship.clone())?;

        Ok(())
    } else {
        Err(format!("Not enough cargo space: Remaining {} / Required {}", ship.get_cargo_capacity(), item.get_volume_per_unit() * amount))
    }
}