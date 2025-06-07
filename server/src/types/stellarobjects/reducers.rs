
use super::{utility::*, *};

///////////////////////////////////////////////////////////
// Reducers ///
///////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn create_sobj_player_window_for(ctx: &ReducerContext, identity: Identity, sobj_id: StellarObjectId) -> Result<(), String> {
    let dsl = dsl(ctx);

    dsl.create_sobj_player_window(
        identity, 
        &sobj_id, 
        4000.0,
        2000.0,
        -2000.0,
        -2000.0,
        2000.0,
        2000.0)?;
    info!("Created player window for {} and object #{}!", identity.to_abbreviated_hex().to_string(), sobj_id.value);
    Ok(())
}

#[spacetimedb::reducer]
pub fn create_turn_left_controller_for(ctx: &ReducerContext, sobj_id: StellarObjectId) -> Result<(), String> {
    let dsl = dsl(ctx);

    if let Some(controller) = dsl.get_sobj_turn_left_controller_by_sobj_id(&sobj_id) {
        dsl.delete_sobj_turn_left_controller_by_sobj_id(controller.get_sobj_id());
        spacetimedb::log::info!("Deleted controller #{:?}", sobj_id.value);
    } else {
        let controller = dsl.create_sobj_turn_left_controller(sobj_id)?;
        spacetimedb::log::info!("Created controller #{}", controller.sobj_id);
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn create_stellar_object(
    ctx: &ReducerContext,
    kind: StellarObjectKinds,
    sector_id: SectorId,
    transform: StellarObjectTransformInternal
) -> Result<(), String> {
    //server_only(ctx);

    match create_sobj_internal(ctx, kind, &sector_id, transform) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[spacetimedb::reducer]
pub fn create_sobj_random(ctx: &ReducerContext, sector_id: u64) -> Result<(), String> {
    //server_only(ctx);

    create_stellar_object(
        ctx,
        StellarObjectKinds::Ship,
        SectorId::new(sector_id),
        StellarObjectTransformInternal {
            sobj_id: 0,
            x: ctx.rng().gen_range(0.0..1024.0),
            y: ctx.rng().gen_range(0.0..512.0),
            rotation_radians: ctx.rng().gen_range(-std::f32::consts::PI..std::f32::consts::PI),
        }
    )
}

//// Called by clients to update their ships. Will limit the acceleration and etc.
// #[spacetimedb::reducer]
// pub fn update_sobj_velocity(
//     ctx: &ReducerContext,
//     velocity: StellarObjectVelocity
// ) -> Result<(), String> {
//     let dsl = dsl(ctx);

//     is_server_or_owner(ctx, velocity.get_sobj_id())?;

//     let mut update_velocity = velocity.clone();
//     //let ship_def = dsl.get_ship_type_definition_by_id(dsl.get_ship_instance_by_id())
    
//     match dsl.get_sobj_velocity_by_sobj_id(velocity.get_sobj_id()) {
//         Some(prev_velocity) => {
//             // Check if the acceleration required for the velocity change is too high
//             let acceleration = (velocity.to_vec2() - prev_velocity.to_vec2()).length();
//             if acceleration > 1.0 {
//                 //// TODO: Make this variable per ship type
//                 //log::info!("Acceleration too high! {:?}", acceleration);

//                 // Reduce the acceleration down                
//                 update_velocity = update_velocity.from_vec2(
//                     prev_velocity.to_vec2() +
//                     (update_velocity.to_vec2() - prev_velocity.to_vec2()).normalize() * 1.0);
//             }

//             // Check if the absolute velocity is too fast for the ship.
//             if update_velocity.to_vec2().length() > 50.0 {
//                 //// TODO: Make this variable per ship type
//                 //log::info!("Velocity too high! {:?}", update_velocity.to_vec2().length());

//                 // Reduce the velocity down
//                 let new_velocity = update_velocity.to_vec2().normalize() * 50.0;
//                 update_velocity = update_velocity.from_vec2(new_velocity);
//             }
//         }
//         None => {
//             return Err("Stellar object's velocity table entry was not found!".to_string());
//         }
//     }

//     if let Err(e) = dsl.update_sobj_velocity_by_sobj_id(update_velocity) {
//         return Err(e.to_string())
//     }
//     Ok(())
// }