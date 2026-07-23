//! Welcome-back message composition (#92).
//!
//! A returning player should learn, in one glance, what changed in the world
//! while they were away. `client_connected` calls [`send_welcome_back_message`]
//! once per connect; everything below — the table sweeps, the cross-table
//! joins, the prose assembly — is hidden behind that single call.
//!
//! **Text-only. No LLM.** LLM-narrated welcome-backs are Future Vision v1.1,
//! gated by cost analysis.
//!
//! ## What it reports (per the MVP design doc, #92)
//! - Each construction site still under construction: name + progress %, with
//!   the player's own-faction sites flagged (M3 soft-default — anyone may
//!   contribute to any site; the flag is purely informational).
//! - The number of contribution events logged since the player's `last_login`.
//! - The player's current cargo, aggregated across all their ships.
//!
//! ## Deliberately omitted
//! The design doc also names "trades since last login, credits earned, ships
//! visited" as welcome-back content. Markets, credit-generating activity, and
//! ship traffic are all out of MVP scope, so these would only ever read zero.
//! We omit them rather than print honest-but-useless zeros — see #92 for the
//! decision. Re-add them here when the systems that move those numbers ship.
//!
//! ## How the client identifies *the* welcome-back DM
//! Post-#101, `DirectServerMessage` carries no discriminator. The client picks
//! the most recent DM whose `created_at` is at-or-after the player's
//! `last_login` and treats it as the welcome-back. That's the message this
//! function emits exactly once per connect.

use crate::spacetimedsl::prelude::*;
use spacetimedb::Timestamp;

use crate::tables::{items::ItemDefinitionId, messages::send_direct_server_info, players::Player};

/// Compose and deliver the welcome-back `DirectServerMessage` for `player`.
///
/// `player` must be read with the *pre-connect* `last_login` still intact —
/// that timestamp is the "while you were away" boundary. The caller is
/// responsible for re-stamping `last_login` afterwards (see
/// `lifecycle/client_connected.rs`), so this function never mutates the player.
///
/// Errors propagate (per the debugging contract) only if the underlying
/// message insert fails; missing rows during the read sweeps are skipped, not
/// fatal, because a welcome-back that omits a ghost row is better than a
/// connect that bounces.
pub fn send_welcome_back_message<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player: &Player,
) -> Result<(), String> {
    let since = player.last_login;

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("Welcome back, {}.", player.username));

    lines.push(compose_construction_summary(dsl, player));

    if let Some(since) = since {
        lines.push(compose_contribution_summary(dsl, since));
    }

    lines.push(compose_cargo_summary(dsl, player));

    send_direct_server_info(dsl, &player.get_id(), lines.join("\n"))
}

/// One line per construction site still under way — own-faction sites flagged
/// and listed first (#104).
fn compose_construction_summary<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    player: &Player,
) -> String {
    let own_faction = player.faction_id.value();

    let mut own_lines: Vec<String> = Vec::new();
    let mut other_lines: Vec<String> = Vec::new();
    for uc in dsl.get_all_stations_under_construction() {
        if *uc.get_is_operational() {
            continue; // Already finished — not "under construction" any more.
        }
        let station = match dsl.get_station_by_id(&uc.get_id()) {
            Ok(s) => s,
            Err(_) => continue, // Station row gone; skip rather than bail.
        };
        let sector = match dsl.get_sector_by_id(station.get_sector_id()){
            Ok(s) => s,
            Err(_) => continue, // Sector row gone, somehow; skip rather than bail.
        };
        let progress = *uc.get_construction_progress_percentage();
        let is_own = station.get_owner_faction_id().value() == own_faction;
        let flag = if is_own { " (your faction)" } else { "" };
        let line = format!(
            "  • {} in {} — {:.0}% complete{}",
            station.get_name(),
            sector.get_name(),
            progress,
            flag
        );
        if is_own {
            own_lines.push(line);
        } else {
            other_lines.push(line);
        }
    }

    let mut site_lines = own_lines;
    site_lines.extend(other_lines);

    if site_lines.is_empty() {
        "No construction sites are active right now.".to_string()
    } else {
        format!("Construction sites:\n{}", site_lines.join("\n"))
    }
}

/// Count contribution events logged strictly after `since`.
fn compose_contribution_summary<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    since: Timestamp,
) -> String {
    let events = dsl
        .get_all_construction_contribution_logs()
        .filter(|log| *log.get_contributed_at() > since)
        .count();

    match events {
        0 => "No new contributions since your last visit.".to_string(),
        1 => "1 contribution was logged since your last visit.".to_string(),
        n => format!("{} contributions were logged since your last visit.", n),
    }
}

/// Aggregate the player's cargo across every ship they own and name the items.
fn compose_cargo_summary<T: spacetimedsl::WriteContext>(dsl: &DSL<T>, player: &Player) -> String {
    // (item_id, total_quantity), summed across all of the player's ships.
    let mut totals: Vec<(u32, u32)> = Vec::new();
    for ship in dsl.get_ships_by_player_id(&player.get_id()) {
        for item in dsl.get_ship_cargo_items_by_ship_id(&ship.get_id()) {
            let id = item.get_item_id().value();
            let qty = *item.get_quantity() as u32;
            if let Some(entry) = totals.iter_mut().find(|(i, _)| *i == id) {
                entry.1 += qty;
            } else {
                totals.push((id, qty));
            }
        }
    }

    if totals.is_empty() {
        return "Your cargo holds are empty.".to_string();
    }

    let parts: Vec<String> = totals
        .iter()
        .map(|(id, qty)| {
            let name = dsl
                .get_item_definition_by_id(&ItemDefinitionId::new(*id))
                .map(|def| def.get_name().clone())
                .unwrap_or_else(|_| format!("item #{}", id));
            format!("{} {}", qty, name)
        })
        .collect();

    format!("Your cargo: {}.", parts.join(", "))
}
