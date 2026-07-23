use spacetimedb::ReducerContext;
use crate::spacetimedsl::prelude::*;

use crate::{
    logic::ships::cargo::remove_cargo_from_ship,
    logic::stations::create_station_with_modules,
    logic::stellarobjects::movement::get_ship_movement_snapshot,
    tables::{
        economy::ResourceAmount,
        factions::FactionId,
        items::*,
        messages::{
            post_galaxy_channel, send_direct_server_info, send_direct_server_warning, MessageSender,
        },
        players::*,
        sectors::Sector,
        ships::*,
        stations::*,
        stellarobjects::StellarObject,
    },
};

/// Maximum world-space distance (px) between a contributing ship and a
/// construction site. Mirrored client-side in `construction_window.rs` so the
/// UI can warn before the player tries.
pub const CONTRIBUTE_RANGE_PX: f32 = 300.0;

///////////////////////////////////////////////////////////
// Pure progress engine
///////////////////////////////////////////////////////////

/// Compute construction progress as the average completion ratio across the
/// required resource types, expressed as a percentage in `[0.0, 100.0]`.
///
/// `requirements` is the spec: `(item_id, quantity_required)` per resource
/// type. `contributions` is the aggregated total contributed per item id.
/// Items present in `contributions` but absent from `requirements` are
/// ignored — only the spec defines the completion baseline.
///
/// Behavior:
/// - Empty requirements → 0.0 (no spec, no progress).
/// - Each requirement contributes `min(contributed/required, 1.0)` to the
///   average; over-contribution to one resource cannot mask a deficit in
///   another.
/// - A requirement with `quantity_required == 0` counts as fully complete
///   (1.0). This is a defensive choice for malformed specs — the reducer
///   should never insert one.
pub fn compute_construction_progress(
    requirements: &[(u32, u32)],
    contributions: &[(u32, u32)],
) -> f32 {
    if requirements.is_empty() {
        return 0.0;
    }

    let mut total_ratio: f32 = 0.0;

    for (req_item_id, req_qty) in requirements {
        if *req_qty == 0 {
            total_ratio += 1.0;
            continue;
        }

        let contributed: u32 = contributions
            .iter()
            .filter(|(item_id, _)| item_id == req_item_id)
            .map(|(_, qty)| *qty)
            .sum();

        let ratio = (contributed as f32 / *req_qty as f32).min(1.0);
        total_ratio += ratio;
    }

    (total_ratio / requirements.len() as f32) * 100.0
}

///////////////////////////////////////////////////////////
// DSL-bound helpers
///////////////////////////////////////////////////////////

/// Sum every contribution row for the given station, grouped by item id.
fn aggregate_contributions<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station_id: &StationId,
) -> Vec<(u32, u32)> {
    let mut totals: Vec<(u32, u32)> = Vec::new();
    for log in dsl.get_construction_contribution_logs_by_station_id(station_id) {
        let item_id = log.get_item_id().value();
        let qty = *log.get_quantity();
        if let Some(entry) = totals.iter_mut().find(|(id, _)| *id == item_id) {
            entry.1 += qty;
        } else {
            totals.push((item_id, qty));
        }
    }
    totals
}

/// Collect the requirement spec for a station as `(item_id, required)` pairs.
fn collect_requirements<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station_id: &StationId,
) -> Vec<(u32, u32)> {
    dsl.get_construction_requirements_by_station_id(station_id)
        .map(|req| (req.get_resource_item_id().value(), *req.get_quantity_required()))
        .collect()
}

/// Recompute progress for a single station from current table state and
/// persist the new percentage. If progress hits 100% and the site was not
/// already flagged operational, flip the bit and broadcast a system
/// completion message to every logged-in player.
fn refresh_station_progress<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station_id: &StationId,
) -> Result<f32, String> {
    let requirements = collect_requirements(dsl, station_id);
    let contributions = aggregate_contributions(dsl, station_id);
    let progress = compute_construction_progress(&requirements, &contributions);

    let mut under_construction = dsl.get_station_under_construction_by_id(station_id)?;
    let was_operational = *under_construction.get_is_operational();
    under_construction.set_construction_progress_percentage(progress);

    let now_complete = progress >= 100.0 && !was_operational;
    if now_complete {
        under_construction.set_is_operational(true);
    }

    dsl.update_station_under_construction_by_id(under_construction)?;

    if now_complete {
        let station = dsl.get_station_by_id(station_id)?;
        // Construction completion is a genuinely async, everyone-relevant event:
        // post it to the Galaxy channel as System. Replaces the old per-player
        // fan-out via `send_server_message_to_group`.
        post_galaxy_channel(
            dsl,
            MessageSender::System,
            format!(
                "Construction complete: '{}' (station #{}) is now operational.",
                station.get_name(),
                station_id.value()
            ),
        )?;
    }

    Ok(progress)
}

///////////////////////////////////////////////////////////
// Construction site lifecycle helpers
///////////////////////////////////////////////////////////

/// Create a station that starts life under construction: no modules, zero
/// progress, with the given resource requirement spec. Used by both the
/// init seeder in `definitions/galaxy.rs` and the admin reducer in
/// `admin/construction.rs` so the two paths can't drift.
pub fn create_construction_site<T: spacetimedsl::WriteContext + 'static>(
    dsl: &DSL<T>,
    size: StationSize,
    sector: &Sector,
    sobj: &StellarObject,
    owner_faction_id: FactionId,
    name: &str,
    position: solarance_shared::Vec2,
    rotation: f32,
    requirements: Vec<ResourceAmount>,
) -> Result<Station, String> {
    if requirements.is_empty() {
        return Err(format!(
            "create_construction_site refused: '{}' has no requirements — would never complete",
            name
        ));
    }

    let station = create_station_with_modules(
        dsl,
        size,
        sector,
        sobj,
        owner_faction_id,
        name,
        None,
        position,
        rotation,
        Vec::new(),
    )?;

    dsl.create_station_under_construction(CreateStationUnderConstruction {
        id: station.get_id(),
        is_operational: false,
        construction_progress_percentage: 0.0,
    })?;

    for req in requirements {
        if req.quantity == 0 {
            return Err(format!(
                "create_construction_site rejected zero-quantity requirement for item {} on station {}",
                req.resource_item_id,
                station.get_id().value()
            ));
        }
        dsl.create_construction_requirement(CreateConstructionRequirement {
            station_id: station.get_id(),
            resource_item_id: ItemDefinitionId::new(req.resource_item_id),
            quantity_required: req.quantity,
        })?;
    }

    Ok(station)
}

/// Wipe every contribution row for the station and zero the progress bar.
/// Used by `admin_reset_construction_site` so the designer can replay the
/// completion moment without re-publishing the module.
pub fn reset_construction_site<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station_id: &StationId,
) -> Result<(), String> {
    let log_ids: Vec<_> = dsl
        .get_construction_contribution_logs_by_station_id(station_id)
        .map(|log| log.get_id().clone())
        .collect();
    let cleared = log_ids.len();
    for id in log_ids {
        dsl.delete_construction_contribution_log_by_id(&id)?;
    }

    let mut under_construction = dsl.get_station_under_construction_by_id(station_id)?;
    under_construction.set_is_operational(false);
    under_construction.set_construction_progress_percentage(0.0);
    dsl.update_station_under_construction_by_id(under_construction)?;

    log::info!(
        "reset_construction_site: station_id={} cleared {} contribution log rows",
        station_id.value(),
        cleared
    );
    Ok(())
}

///////////////////////////////////////////////////////////
// Reducers
///////////////////////////////////////////////////////////

/// Deposit cargo from the caller's ship into a station-under-construction.
///
/// One conceptual operation per the debugging contract: validate → cap →
/// move items → log → recompute → maybe-complete. Every reject path carries
/// a contextual error string keyed off the station / ship / item ids so
/// `spacetime logs` is enough to diagnose a failed contribution.
#[spacetimedb::reducer]
pub fn contribute_to_station(
    ctx: &ReducerContext,
    station_id: StationId,
    item_id: ItemDefinitionId,
    quantity: u32,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    let player_id = PlayerId::new(ctx.sender());

    if quantity == 0 {
        return Err(format!(
            "contribute_to_station rejected: quantity must be > 0 (player {}, station {}, item {})",
            player_id.value().to_abbreviated_hex(),
            station_id.value(),
            item_id.value()
        ));
    }

    let (ship, _sobj) = get_player_ship_and_sobj(&dsl, &player_id)?;
    let station = dsl.get_station_by_id(&station_id)?;
    let under_construction = dsl
        .get_station_under_construction_by_id(&station_id)
        .map_err(|e| {
            format!(
                "contribute_to_station rejected: station {} is not under construction ({})",
                station_id.value(),
                e
            )
        })?;

    if *under_construction.get_is_operational() {
        let msg = format!(
            "Station {} is already operational — no further contributions accepted.",
            station_id.value()
        );
        let _ = send_direct_server_warning(&dsl, &player_id, msg.clone());
        return Err(msg);
    }

    if ship.get_sector_id().value() != station.get_sector_id().value() {
        let msg = format!(
            "Cannot contribute: your ship is in sector {} but station {} is in sector {}.",
            ship.get_sector_id().value(),
            station_id.value(),
            station.get_sector_id().value()
        );
        let _ = send_direct_server_warning(&dsl, &player_id, msg.clone());
        return Err(msg);
    }

    let ship_snapshot = get_ship_movement_snapshot(&dsl, &ship.get_id())?;
    let station_pos = station.get_position();
    let dx = ship_snapshot.pos.x - station_pos.x;
    let dy = ship_snapshot.pos.y - station_pos.y;
    let dist_sq = dx * dx + dy * dy;
    let max_dist_sq = CONTRIBUTE_RANGE_PX * CONTRIBUTE_RANGE_PX;
    if dist_sq > max_dist_sq {
        let msg = format!(
            "Too far to contribute: ship #{} is {:.0}px from '{}' (max {:.0}px).",
            ship.get_id().value(),
            dist_sq.sqrt(),
            station.get_name(),
            CONTRIBUTE_RANGE_PX,
        );
        let _ = send_direct_server_warning(&dsl, &player_id, msg.clone());
        return Err(msg);
    }

    let requirement = dsl
        .get_construction_requirements_by_station_id(&station_id)
        .find(|r| r.get_resource_item_id() == item_id);
    let requirement = match requirement {
        Some(r) => r,
        None => {
            let item_def = dsl.get_item_definition_by_id(item_id)?;
            let msg = format!(
                "Station {} does not require '{}' for construction.",
                station_id.value(),
                item_def.get_name()
            );
            let _ = send_direct_server_warning(&dsl, &player_id, msg.clone());
            return Err(msg);
        }
    };

    let contributions = aggregate_contributions(&dsl, &station_id);
    let already_contributed: u32 = contributions
        .iter()
        .filter(|(id, _)| *id == item_id.value())
        .map(|(_, q)| *q)
        .sum();
    let required = *requirement.get_quantity_required();
    let remainder = required.saturating_sub(already_contributed);

    if remainder == 0 {
        let item_def = dsl.get_item_definition_by_id(item_id)?;
        let msg = format!(
            "Station {} already has all '{}' it needs ({} / {}).",
            station_id.value(),
            item_def.get_name(),
            already_contributed,
            required
        );
        let _ = send_direct_server_warning(&dsl, &player_id, msg.clone());
        return Err(msg);
    }

    let effective_qty = quantity.min(remainder);
    let effective_qty_u16: u16 = effective_qty.try_into().map_err(|_| {
        format!(
            "contribute_to_station: capped quantity {} exceeds u16 cargo limit for station {} item {}",
            effective_qty,
            station_id.value(),
            item_id.value()
        )
    })?;

    let item_def = dsl.get_item_definition_by_id(&item_id)?;
    let mut ship_status = dsl.get_ship_status_by_id(&ship.get_id())?;

    let cargo_available: u32 = dsl
        .get_ship_cargo_items_by_ship_id(&ship.get_id())
        .filter(|c| c.get_item_id() == item_id)
        .map(|c| *c.get_quantity() as u32)
        .sum();

    if cargo_available < effective_qty {
        let msg = format!(
            "Cannot contribute {}x {} to station {}: ship #{} only carries {}.",
            effective_qty,
            item_def.get_name(),
            station_id.value(),
            ship.get_id().value(),
            cargo_available
        );
        let _ = send_direct_server_warning(&dsl, &player_id, msg.clone());
        return Err(msg);
    }

    remove_cargo_from_ship(&dsl, &mut ship_status, &item_def, effective_qty_u16)?;

    dsl.create_construction_contribution_log(CreateConstructionContributionLog {
        station_id: station_id.clone(),
        player_id: player_id.clone(),
        item_id,
        quantity: effective_qty,
        contributed_at: ctx.timestamp,
    })?;

    let new_progress = refresh_station_progress(&dsl, &station_id)?;

    send_direct_server_info(
        &dsl,
        &player_id,
        format!(
            "Contributed {}x {} to station '{}'. Construction now {:.1}%.",
            effective_qty,
            item_def.get_name(),
            station.get_name(),
            new_progress
        ),
    )?;

    Ok(())
}

///////////////////////////////////////////////////////////
// Unit tests — pure progress engine only
///////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::compute_construction_progress;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    #[test]
    fn empty_requirements_returns_zero() {
        assert!(approx(compute_construction_progress(&[], &[]), 0.0));
        assert!(approx(compute_construction_progress(&[], &[(1, 50)]), 0.0));
    }

    #[test]
    fn single_resource_zero_percent() {
        let reqs = [(1, 100)];
        assert!(approx(compute_construction_progress(&reqs, &[]), 0.0));
    }

    #[test]
    fn single_resource_fifty_percent() {
        let reqs = [(1, 100)];
        let contribs = [(1, 50)];
        assert!(approx(
            compute_construction_progress(&reqs, &contribs),
            50.0
        ));
    }

    #[test]
    fn single_resource_one_hundred_percent() {
        let reqs = [(1, 100)];
        let contribs = [(1, 100)];
        assert!(approx(
            compute_construction_progress(&reqs, &contribs),
            100.0
        ));
    }

    #[test]
    fn over_contribution_capped_at_one_hundred() {
        let reqs = [(1, 100)];
        let contribs = [(1, 500)];
        assert!(approx(
            compute_construction_progress(&reqs, &contribs),
            100.0
        ));
    }

    #[test]
    fn multi_resource_partial_average() {
        let reqs = [(1, 100), (2, 200)];
        let contribs = [(1, 100), (2, 50)]; // 100% + 25% → avg 62.5%
        assert!(approx(
            compute_construction_progress(&reqs, &contribs),
            62.5
        ));
    }

    #[test]
    fn multi_resource_over_one_cannot_mask_under_another() {
        let reqs = [(1, 100), (2, 200)];
        let contribs = [(1, 1000), (2, 0)]; // 100% (capped) + 0% → 50%
        assert!(approx(
            compute_construction_progress(&reqs, &contribs),
            50.0
        ));
    }

    #[test]
    fn aggregated_contributions_are_summed_by_caller() {
        // The engine treats `contributions` as already-aggregated, but a
        // duplicate-id slice should still behave like the sum.
        let reqs = [(1, 100)];
        let contribs = [(1, 25), (1, 25), (1, 50)];
        assert!(approx(
            compute_construction_progress(&reqs, &contribs),
            100.0
        ));
    }

    #[test]
    fn unrequired_contributions_are_ignored() {
        let reqs = [(1, 100)];
        let contribs = [(1, 50), (99, 10_000)];
        assert!(approx(
            compute_construction_progress(&reqs, &contribs),
            50.0
        ));
    }

    #[test]
    fn zero_required_counts_complete() {
        let reqs = [(1, 100), (2, 0)];
        let contribs = [(1, 100)];
        assert!(approx(
            compute_construction_progress(&reqs, &contribs),
            100.0
        ));
    }
}
