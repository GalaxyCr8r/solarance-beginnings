use super::*;
use crate::tables::ships::*;
use spacetimedb::TimeDuration;
use spacetimedsl::*;

#[dsl(plural_name = visual_effect_timers)]
#[spacetimedb::table(name = visual_effect_timer, scheduled(cleanup_visual_effect))]
pub struct VisualEffectTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(path = VisualEffectId)]
    #[foreign_key(
        path = crate::tables::combat,
        table = visual_effect,
        column = id,
        on_delete = Delete
    )]
    pub effect_id: u64,

    pub scheduled_at: ScheduleAt,
}

#[spacetimedb::reducer]
pub fn cleanup_visual_effect(ctx: &ReducerContext, timer: VisualEffectTimer) -> Result<(), String> {
    let dsl = dsl(ctx);

    // Delete the visual effect from the database
    let effect_id = VisualEffectId::new(timer.get_effect_id().value());
    if let Ok(_) = dsl.delete_visual_effect_by_id(effect_id) {
        spacetimedb::log::info!("Cleaned up visual effect {}", timer.get_effect_id().value());
    } else {
        // Visual effect might have already been deleted, which is fine
        spacetimedb::log::info!(
            "Visual effect {} already cleaned up",
            timer.get_effect_id().value()
        );
    }

    // The timer itself is automatically deleted by SpacetimeDB after this reducer runs
    Ok(())
}

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

pub fn init(dsl: &DSL) -> Result<(), String> {
    // Schedule the cooldown update timer to run every 100ms
    let cooldown_timer = dsl.create_combat_cooldown_timer(spacetimedb::ScheduleAt::Interval(
        TimeDuration::from_micros(100_000), // 100ms = 100,000 microseconds
    ))?;

    spacetimedb::log::info!(
        "Combat cooldown timer initialized with ID {}",
        cooldown_timer.get_id().value()
    );

    Ok(())
}
