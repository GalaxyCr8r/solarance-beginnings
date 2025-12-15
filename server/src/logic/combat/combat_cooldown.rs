use crate::tables::ships::*;
use spacetimedb::*;
use spacetimedsl::*;

#[dsl(plural_name = combat_cooldown_timers)]
#[spacetimedb::table(name = combat_cooldown_timer, scheduled(update_combat_cooldowns))]
pub struct CombatCooldownTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub scheduled_at: ScheduleAt,
}

/// Scheduled reducer to update weapon and missile cooldowns for all ships
/// This runs every 100ms to provide smooth cooldown updates
#[spacetimedb::reducer]
pub fn update_combat_cooldowns(
    ctx: &ReducerContext,
    _timer: CombatCooldownTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Update cooldowns for all ships that have active cooldowns
    let ship_statuses: Vec<ShipStatus> = dsl
        .get_all_ship_statuses()
        .filter(|status| {
            *status.get_weapon_cooldown_ms() > 0 || *status.get_missile_cooldown_ms() > 0
        })
        .collect();

    for mut ship_status in ship_statuses {
        let mut updated = false;

        // Update weapon cooldown
        if *ship_status.get_weapon_cooldown_ms() > 0 {
            let new_weapon_cooldown = ship_status.get_weapon_cooldown_ms().saturating_sub(100);
            ship_status.set_weapon_cooldown_ms(new_weapon_cooldown);
            updated = true;
        }

        // Update missile cooldown
        if *ship_status.get_missile_cooldown_ms() > 0 {
            let new_missile_cooldown = ship_status.get_missile_cooldown_ms().saturating_sub(100);
            ship_status.set_missile_cooldown_ms(new_missile_cooldown);
            updated = true;
        }

        // Only update the database if we actually changed something
        if updated {
            dsl.update_ship_status_by_id(ship_status)?;
        }
    }

    Ok(())
}
