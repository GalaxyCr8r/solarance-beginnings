
use log::info;
use spacetimedb::{table, Identity, ReducerContext, Timestamp};
use spacetimedsl::{dsl, Wrapper};

use crate::types::{items::{GetCargoCrateRowOptionBySobjId, GetItemDefinitionRowOptionById}, ships::{timers::{create_timer_to_add_cargo_to_ship, DeleteShipMiningTimerRowByScheduledId, GetShipMiningTimerRowsByShipSobjId}, GetShipInstanceRowOptionById, GetShipObjectRowOptionBySobjId}, stellarobjects::{GetStellarObjectPlayerWindowRowOptionByIdentity, GetStellarObjectRowOptionById, StellarObject, StellarObjectId}};

use super::{common::CurrentAction, ships::ship_object};

pub mod timers;

#[dsl(plural_name = players)]
#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,

    #[unique]
    pub username: String,
    pub credits: u64,

    created_at: Timestamp,
    modified_at: Timestamp,
}

#[dsl(plural_name = player_controllers)]
#[table(name = player_controller, public)]
pub struct PlayerController {
    #[primary_key]
    pub identity: Identity,

    pub stellar_object_id: Option<u64>,

    // Movement
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    /// Currently selected Autopilot Action
    pub current_action: CurrentAction,

    // Equipment
    pub activate_jump_drive: bool,
    pub tractor_beam_on: bool,
    pub mining_laser_on: bool,
    pub cargo_bay_open: bool,

    // Actions
    pub dock: bool,
    pub undock: bool,
    pub shield_boost: bool,
    pub fire_weapons: bool,
    pub fire_missle: bool,

    // Misc
    pub targetted_sobj_id: Option<u64>, // FK to StellarObject
}


//////////////////////////////////////////////////////////////
// Impls ///
//////////////////////////////////////////////////////////////

impl Player {
    pub fn get_ship_id(&self, ctx: &ReducerContext) -> Option<u64> {
        if let Some(player_controlled_stellar_object) = ctx.db.ship_object().player_id().filter(self.identity).last() {
            Some(player_controlled_stellar_object.sobj_id)
        } else {
            None
        }
    }
}

pub fn get_username(ctx: &ReducerContext, id:Identity) -> String {
    if let Some(player) = ctx.db.player().identity().find(id) {
        player.username
    } else {
        if ctx.sender == ctx.identity() {
            "SERVER".to_string()
        } else {
            id.to_abbreviated_hex().to_string()
        }
    }
}

//////////////////////////////////////////////////////////////
// Reducers ///
//////////////////////////////////////////////////////////////

#[spacetimedb::reducer]
pub fn update_player_controller(ctx: &ReducerContext, mut controller: PlayerController) -> Result<(), String> {
    let dsl = dsl(ctx);

    if ctx.sender != controller.identity {
        info!("SECURITY ERROR: ID {} is trying to change player controller for ID {}!!! Username: {}", 
            ctx.sender, controller.identity, get_username(ctx, controller.identity));
        return Err("ID Mismatch. This was reported to the system admin.".to_string());
    }

    // Check target-specific things.
    if controller.targetted_sobj_id.is_none() || controller.stellar_object_id.is_none() {
        // info!("ERROR: Player {} does not have a stellar object and/or a target!", 
        //     get_username(ctx, controller.identity));
    } else {
        let player_sobj = dsl.get_stellar_object_by_id(StellarObjectId::new(controller.stellar_object_id.unwrap())).ok_or(
            format!("ERROR: Player {} has stellar object #{} that does not exist!", 
                get_username(ctx, controller.identity), controller.stellar_object_id.unwrap())
        )?;

        match verify_target(ctx, &controller, &player_sobj) { // These "Do things if nearby target" should be in their own timer. As-is things will ONLY happen if you are updating your controller when nearby!!!
            Ok(target_sobj) => {
                use super::stellarobjects::StellarObjectKinds;
                match target_sobj.kind {
                    StellarObjectKinds::Ship => {
                        // Nothing to do.. yet

                        // Maybe implement ship scanning?
                    },
                    StellarObjectKinds::Asteroid => {
                        // Nothing to do.. yet

                        // Maybe implement asteroid scanning?
                    },
                    StellarObjectKinds::CargoCrate => {
                        if controller.cargo_bay_open && 
                           player_sobj.distance_squared(ctx, &target_sobj).is_some_and(|d| d < 100.0) &&
                           attempt_to_pickup_cargo_crate(ctx, &player_sobj, &target_sobj) {
                            // Picking up the crate!
                            controller.targetted_sobj_id = None;
                            target_sobj.delete(ctx);
                        }
                    },
                    StellarObjectKinds::Station => {
                        // Nothing to do.. yet
                    },
                    StellarObjectKinds::JumpGate => {
                        if controller.dock && 
                           player_sobj.distance_squared(ctx, &target_sobj).is_some_and(|d| d < 50.0) {
                            info!("Trying to jump to sector but its' not implemented yet :'(");
                        }
                    },
                    _ => {
                        // Do nothing
                    }
                }
            },
            Err(error) => {
                info!("WARNING: {}", error);
                controller.targetted_sobj_id = None;
            },
        }
    }

    // Clean up player's mining timers.
    if let Some(previous_controller) = dsl.get_player_controller_by_identity(&controller.identity) {
        // Check if the player had been trying to mine, if so, remove the mining timers.
        if previous_controller.mining_laser_on && !controller.mining_laser_on {
            info!("Player {} no longer mining, removing mining timers.", 
                get_username(ctx, controller.identity));
            for mining_timer in dsl.get_ship_mining_timers_by_ship_sobj_id(StellarObjectId::new(previous_controller.stellar_object_id.unwrap())) {
                dsl.delete_ship_mining_timer_by_scheduled_id(&mining_timer);
            }
        }
    }

    // Even if there's no sobj or target, still update
    ctx.db.player_controller().identity().update(controller.clone());
    //info!("Player controller changed! {:?}", controller);

    Ok(())
}

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init(_ctx: &ReducerContext) -> Result<(), String> {

    Ok(())
}

//////////////////////////////////////////////////////////////
// Utils
//////////////////////////////////////////////////////////////
 
fn verify_target(ctx: &ReducerContext, controller: &PlayerController, player_sobj: &StellarObject) -> Result<StellarObject, String> {
    let dsl = dsl(ctx);

    if let Some(target_sobj) = dsl.get_stellar_object_by_id(StellarObjectId::new(controller.targetted_sobj_id.unwrap())) {
        if player_sobj.sector_id != target_sobj.sector_id {
            Err(format!("Player {} tried to target a stellar object in a different sector! Player SOBJ ID: {}, Target SOBJ ID: {}", 
                get_username(ctx, controller.identity), player_sobj.id, target_sobj.id))
        } else {
            Ok(target_sobj)
        }
    } else {
        Err(format!("Player {} tried targetting a non-existant stellar object #{}!",
            get_username(ctx, controller.identity), controller.targetted_sobj_id.unwrap()))
    }    
}

fn attempt_to_pickup_cargo_crate(ctx: &ReducerContext, player_sobj: &StellarObject, crate_sobj: &StellarObject) -> bool {
    let dsl = dsl(ctx);

    if let Some(cargo_crate) = dsl.get_cargo_crate_by_sobj_id(crate_sobj) {
        if let Some(item_def) = dsl.get_item_definition_by_id(cargo_crate.get_item_id()) {
            if let Some(ship_obj) = dsl.get_ship_object_by_sobj_id(player_sobj) {
                if let Some(ship) = dsl.get_ship_instance_by_id(ship_obj.get_ship_id()) {
                    if item_def.can_any_of_this_fit_inside_this_ship(&ship) {
                        return create_timer_to_add_cargo_to_ship(
                                ctx, ship.get_id(),
                                item_def.get_id(),
                                cargo_crate.quantity
                            ).inspect_err(|e| info!("WARNING: Couldn't add cargo crate timer: {}", e)).is_ok()
                    }
                }
            }
        }
    }

    //can_any_of_this_fit_inside_this_ship

    false
}