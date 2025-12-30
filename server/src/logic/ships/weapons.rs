
use log::info;
use spacetimedb::{reducer, ReducerContext};
use spacetimedsl::*;

use crate::{
    logic::combat::actions::*,
    tables::{
        players::*, ships::*,
        stellarobjects::*,
    },
};

#[reducer]
pub fn fire_weapons(ctx: &ReducerContext, target_sobj_id: u64) -> Result<(), String> {
    let dsl = dsl(ctx);

    let ship = dsl
        .get_ships_by_player_id(PlayerId::new(ctx.sender))
        .next()
        .ok_or(format!(
            "Ship couldn't be found for playerId:: {}",
            ctx.sender
        ))?;

    let target_sobj = dsl.get_stellar_object_by_id(StellarObjectId::new(target_sobj_id))?;

    // Currently every path prints the username, so just grab it here
    let username = get_username(&dsl, ctx.sender);

    if ship.get_sector_id() != target_sobj.get_sector_id() {
        return Err(format!(
            "Weapon cannot fire at object in another sector! {} -> {} by player {}",
            ship.get_sobj_id().value(),
            target_sobj_id,
            username
        ));
    }

    match process_weapon_combat_action(&dsl, &ship.get_sobj_id(), &target_sobj.get_id()) {
        Ok(_) => {
            info!(
                "Weapon fired successfully: {} -> {} by player {}",
                ship.get_sobj_id().value(),
                target_sobj_id,
                username
            );
        }
        Err(e) => {
            info!(
                "Weapon fire failed for ship {} (player {}): {}",
                ship.get_sobj_id().value(),
                username,
                e
            );
        }
    }

    Ok(())
}
