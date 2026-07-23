use spacetimedb::*;
use crate::spacetimedsl::prelude::*;

use crate::tables::*;

// --- Views ---

/// View: Returns all ships in the player's current sector.
#[view(accessor = my_player_state, public)]
pub fn my_player_state(ctx: &ViewContext) -> Vec<PlayerState> {
    match ctx.db.player_state().id().find(ctx.sender()) {
        Some(p) => vec![p],
        None => Vec::new(), // Player hasn't joined/initialized
    }
}

/// View: Returns all ships in the player's current sector.
#[view(accessor = current_sector_ships, public)]
pub fn current_sector_ships(ctx: &ViewContext) -> impl Query<SpaceShip> {
    ctx.from
        .player_state()
        .filter(|player| player.id.eq(ctx.sender()))
        .right_semijoin(ctx.from.space_ship(), |player, ship| {
            player.current_sector_id.eq(ship.sector_id)
        })
}

/// View: Returns all bullets in the player's current sector.
#[view(accessor = current_sector_bullets, public)]
pub fn current_sector_bullets(ctx: &ViewContext) -> impl Query<Bullet> {
    ctx.from
        .player_state()
        .filter(|player| player.id.eq(ctx.sender()))
        .right_semijoin(ctx.from.bullet(), |player, bullet| {
            player.current_sector_id.eq(bullet.sector_id)
        })
}

/// View: Returns all sectors in the player's current system that they are
/// authorized to see (either because they visited them or they are public).
#[view(accessor = current_system_visible_sectors, public)]
pub fn current_system_visible_sectors(ctx: &ViewContext) -> impl Query<Sector> {
    let dsl = spacetimedsl::read_only_dsl(ctx);
    // 1. Get the player's current system ID
    let player = match dsl.get_player_state_by_id(PlayerStateId::new(ctx.sender())) {
        Ok(p) => p,
        Err(_) => return ctx.from.sector().r#where(|sector| sector.id.eq(0)).build(), // Player hasn't joined/initialized
    };

    ctx.from
        .sector()
        .r#where(|sector| sector.system_id.eq(player.current_system_id))
        .left_semijoin(ctx.from.visited_sector(), |sector, visited| {
            sector.id.eq(visited.sector_id)
        })
}

/// View: Returns the full System details for every system the player has ever visited.
#[view(accessor = my_visited_systems, public)]
pub fn my_visited_systems(ctx: &ViewContext) -> Vec<System> {
    let dsl = spacetimedsl::read_only_dsl(ctx);
    dsl.get_visited_systems_by_player_id(&ctx.sender())
        .filter(|sys| sys.get_visited_status().is_visible())
        .flat_map(|v| dsl.get_system_by_id(SystemId::new(*v.get_system_id())))
        .collect()
}
